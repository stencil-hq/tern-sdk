--!strict
-- longrun, host half: a `command_finished` hook that toasts when a command
-- took longer than the threshold and keeps the last 50 such commands in
-- `tern.kv`, and the "Slow Commands" block that lists them beside the
-- commands running right now.

local common = require("./common")
local ui = tern.ui

local DEFAULT_THRESHOLD_MS = 15000
local STEP_MS = 5000
local KEEP = 50
local HOME = tern.getenv("HOME")

-- A finished slow command, as stored under kv `history` (newest first).
type Record = {
	line: string,
	cwd: string?,
	took_ms: number,
	status: number,
	at: number, -- os.time() when it finished
}

-- A command a shell is running now.
type Run = {
	line: string,
	cwd: string?,
	started: number, -- tern.now() when it started
}

-- An open Slow Commands block.
type Open = {
	cx: BlockCx,
	since: number,
}

type State = {
	selected: number,
}

local function threshold(): number
	local value = tern.kv.get("threshold_ms")
	return if type(value) == "number" and value > 0 then value else DEFAULT_THRESHOLD_MS
end

-- The history, read once; every change writes it back.
local slow: { Record } = tern.kv.get("history") or {}
-- Commands running now, by pane.
local running: { [number]: Run } = {}
-- Open blocks, by pane.
local blocks: { [number]: Open } = {}
-- The pending re-render tick, while one is scheduled.
local tick: TimerHandle? = nil

local function cwd_of(pane: number): string?
	for _, info in tern.pane.list() do
		if info.pane == pane then
			return info.cwd
		end
	end
	return nil
end

-- Drops blocks whose pane is gone. A block registers in `init`, which can
-- run before the daemon lists its pane, so young entries are kept.
local function prune()
	local live: { [number]: boolean } = {}
	for _, info in tern.pane.list() do
		live[info.pane] = true
	end
	local now = tern.now()
	for pane, open in blocks do
		if not live[pane] and now - open.since > 2000 then
			blocks[pane] = nil
		end
	end
end

-- Re-renders every open block.
local function refresh()
	prune()
	for _, open in blocks do
		open.cx:render()
	end
end

-- Ticks once a second while a block is open and a command runs, so the
-- elapsed times move; stops by itself when either is no longer true.
local function schedule()
	if next(running) == nil or next(blocks) == nil then
		if tick ~= nil then
			tick:cancel()
			tick = nil
		end
		return
	end
	if tick ~= nil then
		return
	end
	tick = tern.timer(1000, function()
		tick = nil
		refresh()
		schedule()
	end)
end

local function remember(record: Record)
	table.insert(slow, 1, record)
	while #slow > KEEP do
		table.remove(slow)
	end
	tern.kv.set("history", slow)
end

tern.on("command_started", function(ev: CommandStartedEvent, _cx: EffectCx)
	running[ev.pane] = { line = ev.line, cwd = ev.cwd, started = tern.now() }
	schedule()
end)

tern.on("command_finished", function(ev: CommandFinishedEvent, cx: EffectCx)
	local run = running[ev.pane]
	running[ev.pane] = nil
	if ev.took_ms >= threshold() and not common.interactive(ev.line) then
		local cwd = if run then run.cwd else cwd_of(ev.pane)
		remember({ line = ev.line, cwd = cwd, took_ms = ev.took_ms, status = ev.status, at = os.time() })
		local line = common.clip(ev.line, 60)
		local took = common.duration(ev.took_ms)
		local where = if cwd then common.tilde(cwd, HOME) else nil
		if ev.status == 0 then
			cx:toast("success", string.format("%s finished in %s", line, took), where)
		else
			local sub = string.format("exit %d", ev.status) .. (if where then " · " .. where else "")
			cx:toast("error", string.format("%s failed after %s", line, took), sub)
		end
	end
	refresh()
	schedule()
end)

tern.on("pane_exited", function(ev: PaneExitedEvent, _cx: EffectCx)
	running[ev.pane] = nil
	refresh()
	schedule()
end)

-- Block views.

local function badge(text: string, tone: string, action: string): Node
	local node = ui.badge(text, tone)
	local props: { [string]: any } = node.p or {}
	props.actions = { click = action }
	node.p = props
	return node
end

local function when(at: number): string
	if os.date("%Y-%m-%d", at) == os.date("%Y-%m-%d") then
		return os.date("%H:%M", at)
	end
	return os.date("%b %d %H:%M", at)
end

local function running_view(): Node?
	local now = tern.now()
	local runs: { Run } = {}
	for _, run in running do
		-- Under a second is noise; interactive programs aren't waits.
		if now - run.started >= 1000 and not common.interactive(run.line) then
			table.insert(runs, run)
		end
	end
	if #runs == 0 then
		return nil
	end
	table.sort(runs, function(a: Run, b: Run)
		return a.started < b.started
	end)
	local rows = {}
	for _, run in runs do
		table.insert(rows, {
			{ ui.span(run.line, "mono") },
			{ ui.span(if run.cwd then common.tilde(run.cwd, HOME) else "", "muted") },
			{ ui.span(common.duration(now - run.started), "num accent") },
		})
	end
	local cols: { Col } = {
		{ id = "command", head = "Command", grow = 1 },
		{ id = "cwd", head = "Directory", truncate = "start", priority = -1 },
		{ id = "elapsed", head = "Running", align = "end" },
	}
	return ui.section("Running now", { ui.table(cols, rows) })
end

local function history_view(state: State): Node
	if #slow == 0 then
		return ui.text({
			ui.span(string.format("No command has taken %s or longer yet.", common.duration(threshold())), "muted"),
		})
	end
	local rows = {}
	for i, record in slow do
		local mark = if i == state.selected then ui.span("›", "accent strong") else ""
		local status = if record.status == 0
			then ui.span("ok", "success")
			else ui.span(string.format("exit %d", record.status), "error")
		table.insert(rows, {
			{ mark },
			{ ui.span(record.line, if i == state.selected then "mono strong" else "mono") },
			{ ui.span(if record.cwd then common.tilde(record.cwd, HOME) else "", "muted") },
			{ ui.span(common.duration(record.took_ms), "num") },
			{ status },
			{ ui.span(when(record.at), "muted num") },
		})
	end
	local cols: { Col } = {
		{ id = "mark", head = "" },
		{ id = "command", head = "Command", grow = 1 },
		{ id = "cwd", head = "Directory", truncate = "start", priority = -1 },
		{ id = "took", head = "Took", align = "end" },
		{ id = "status", head = "Status" },
		{ id = "when", head = "When", align = "end", priority = -2 },
	}
	return ui.table(cols, rows)
end

local function dock(): Node
	return ui.row({
		ui.text({
			ui.span("↑↓", "strong"),
			ui.span(" select  ", "muted"),
			ui.span("⏎", "strong"),
			ui.span(" copy  ", "muted"),
			ui.span("c", "strong"),
			ui.span(" clear  ", "muted"),
			ui.span("+ −", "strong"),
			ui.span(string.format(" threshold %s", common.duration(threshold())), "muted"),
		}),
		badge("Copy", "accent", "copy"),
		badge("Clear", "muted", "clear"),
	})
end

local function copy(state: State, cx: BlockCx)
	local record = slow[state.selected]
	if record then
		cx:copy(record.line)
		cx:toast("info", "Copied", common.clip(record.line, 60))
	end
end

local function clear(state: State)
	table.clear(slow)
	tern.kv.set("history", nil)
	state.selected = 1
	refresh()
end

local function nudge(delta: number)
	local value = math.max(STEP_MS, threshold() + delta)
	tern.kv.set("threshold_ms", value)
	refresh()
end

tern.block.define("slow", {
	init = function(cx: BlockCx, _args: { string }, _saved: any): State
		blocks[cx.pane] = { cx = cx, since = tern.now() }
		schedule()
		return { selected = 1 }
	end,
	title = function(_state: State): string?
		return if #slow > 0 then string.format("Slow Commands (%d)", #slow) else "Slow Commands"
	end,
	view = function(state: State, _cx: BlockCx): BlockView
		state.selected = math.clamp(state.selected, 1, math.max(1, #slow))
		local parts: { Node } = {}
		local now = running_view()
		if now then
			table.insert(parts, now)
		end
		table.insert(parts, history_view(state))
		return { main = ui.col(parts), dock = dock() }
	end,
	key = function(state: State, key: Key, cx: BlockCx): boolean?
		local press = key.text or key.name
		if key.name == "up" or press == "k" then
			state.selected = math.max(1, state.selected - 1)
		elseif key.name == "down" or press == "j" then
			state.selected = math.min(math.max(1, #slow), state.selected + 1)
		elseif key.name == "enter" then
			copy(state, cx)
		elseif press == "c" then
			clear(state)
		elseif press == "+" or press == "=" then
			nudge(STEP_MS)
		elseif press == "-" then
			nudge(-STEP_MS)
		else
			return false
		end
		return true
	end,
	event = function(state: State, ev: UiEvent, cx: BlockCx)
		if ev.ev ~= "action" then
			return
		end
		if ev.act == "copy" then
			copy(state, cx)
		elseif ev.act == "clear" then
			clear(state)
		end
	end,
})
