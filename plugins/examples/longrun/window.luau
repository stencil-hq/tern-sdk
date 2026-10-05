--!strict
-- longrun, window half: the "Show slow commands" command and its key, and a
-- status line segment with the focused pane's running command and how long
-- it has been running, ticking once a second.

local common = require("./common")

local BLOCK = "longrun.slow"
local ACTION = "plugin.longrun.show"
-- The segment appears once a command has run this long, so quick commands
-- don't flash it.
local SHOW_AFTER_MS = 1000

-- A command running in one of this window's panes.
type Run = {
	program: string,
	started: number, -- tern.now() when it started
}

-- Commands running now, by pane. Only commands this window half saw start:
-- one already running when the plugin loaded has no entry.
local running: { [number]: Run } = {}
-- The focused pane, refreshed from the current session on events and ticks.
local focused: number? = nil
-- The pending tick, while one is scheduled.
local tick: TimerHandle? = nil

local function show(cx: WindowCx)
	-- Focus a Slow Commands block this window already shows, else open one.
	for _, pane in cx.session:panes() do
		if string.find(pane.title, "^Slow Commands") then
			cx.layout:focus(pane.pane)
			return
		end
	end
	if cx:new_block(BLOCK, nil, "tab") == nil then
		cx:toast("error", "Slow Commands is unavailable", "The longrun host half isn't loaded on this host.")
	end
end

tern.command({
	id = "show",
	title = "Show slow commands",
	icon = "clock",
	run = show,
})

tern.bind(if tern.runtime.os == "macos" then "cmd+alt+shift+l" else "ctrl+alt+shift+l", ACTION)

-- A reload need not emit window_start or focus. Every event and timer gets
-- a fresh context; never retain one. Snapshots also detect shell exits that
-- supplied no command_finished prompt mark.
local function reconcile(cx: WindowCx)
	focused = cx.session:focused()
	local busy: { [number]: boolean } = {}
	for _, pane in cx.session:panes() do
		busy[pane.pane] = pane.busy
	end
	for pane in running do
		if not busy[pane] then
			running[pane] = nil
		end
	end
end

-- Exactly one timer while the focused pane has an observed running command.
local function schedule()
	if focused == nil or running[focused :: number] == nil then
		if tick ~= nil then
			tick:cancel()
			tick = nil
		end
		return
	end
	if tick ~= nil then
		return
	end
	tick = tern.timer(1000, function(cx: WindowCx?)
		tick = nil
		reconcile(cx :: WindowCx)
		-- Refresh even when reconciliation removed the run, to erase its
		-- last cached segment before stopping the clock.
		tern.chrome.refresh()
		schedule()
	end)
end

tern.on("window_start", function(cx: WindowCx)
	reconcile(cx)
	schedule()
end)

tern.on("focus", function(_ev: FocusEvent, cx: WindowCx)
	reconcile(cx)
	schedule()
end)

tern.on("command_started", function(ev: CommandStartedEvent, cx: WindowCx)
	reconcile(cx)
	if not common.interactive(ev.line) then
		running[ev.pane] = { program = common.program(ev.line) or ev.line, started = tern.now() }
	end
	-- Drop any cached answer from an earlier run with the same input.
	tern.chrome.refresh()
	schedule()
end)

tern.on("command_finished", function(ev: CommandFinishedEvent, cx: WindowCx)
	running[ev.pane] = nil
	reconcile(cx)
	tern.chrome.refresh()
	schedule()
end)

tern.on("pane_closed", function(ev: PaneEvent, cx: WindowCx)
	running[ev.pane] = nil
	reconcile(cx)
	tern.chrome.refresh()
	schedule()
end)

-- Whole seconds under a minute, then as `common.duration` writes them.
local function elapsed(ms: number): string
	if ms < 60000 then
		return string.format("%ds", ms // 1000)
	end
	return common.duration(ms)
end

-- The formatter's answer depends on the clock, which is not part of its
-- input ({pane, cwd, program, title, busy}), so Tern would keep showing its first
-- answer: the timer above calls `tern.chrome.refresh()` to drop it.
tern.chrome.status(function(pane: PaneInfo): { StatusSegment }?
	-- busy changes the formatter input even if program/title happen to stay
	-- identical (for example a command that runs the login shell itself).
	if not pane.busy then
		running[pane.pane] = nil
		schedule()
		return nil
	end
	local run = running[pane.pane]
	if run == nil then
		return nil
	end
	local ms = tern.now() - run.started
	if ms < SHOW_AFTER_MS then
		return nil
	end
	return {
		{
			text = string.format("%s · %s", run.program, elapsed(ms)),
			icon = "clock",
			tone = "muted",
			command = ACTION,
		},
	}
end)
