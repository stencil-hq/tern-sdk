--!strict
-- dirvars, host half: a `spawn` filter that gives each new shell the
-- variables of the `.tern-env` files between its directory and the home
-- directory, but only after the user approves each file's exact contents
-- in the Directory Variables block. A `cwd` hook reports approved changes.

local envfile = require("./envfile")

local FILE = ".tern-env"
-- Larger files are skipped: an env file this big is a mistake.
local MAX_BYTES = 64 * 1024
local function normalize(path: string): string
	local clean = string.gsub(string.gsub(path, "\\", "/"), "/+$", "")
	if clean == "" then
		return "/"
	end
	return if string.match(clean, "^%a:$") then clean .. "/" else clean
end

local home = tern.getenv("HOME") or tern.getenv("USERPROFILE")
local HOME = if home then normalize(home) else nil
local ui = tern.ui
local APPROVAL = "approved:"

-- A file read this far, with what it parsed to. `text` is the cache key:
-- `tern.fs` has no mtime, so every lookup reads the file and parsing runs
-- only when its content changed.
type Parsed = {
	text: string,
	entries: { envfile.Entry },
}

-- The files that applied, farthest first, and their contents.
type Applied = {
	paths: { string },
	texts: { [string]: string },
}

local parsed: { [string]: Parsed } = {}
local invalidated: { [string]: boolean } = {}
-- What each pane started with, by pane id (from the spec's `TERN_PANE`).
local started: { [number]: Applied } = {}
-- Directories each pane was already told about.
local told: { [number]: { [string]: boolean } } = {}

local function parent(dir: string): string?
	if dir == "/" or string.match(dir, "^%a:/$") or string.match(dir, "^//[^/]+/[^/]+$") then
		return nil
	end
	local up = string.match(dir, "^(.*)/[^/]*$")
	return if up == nil or up == "" then "/" else normalize(up)
end

local function join(dir: string, name: string): string
	return if string.sub(dir, -1) == "/" then dir .. name else dir .. "/" .. name
end

local function tilde(path: string): string
	if HOME and string.sub(path, 1, #HOME + 1) == HOME .. "/" then
		return "~" .. string.sub(path, #HOME + 1)
	end
	return path
end

-- The `.tern-env` paths from `cwd` up to the home directory (or the root,
-- for a directory outside home), farthest first.
local function chain(cwd: string): { string }
	local found: { string } = {}
	local dir: string? = normalize(cwd)
	while dir do
		local path = join(dir, FILE)
		if tern.fs.exists(path) then
			table.insert(found, 1, path)
		end
		if dir == HOME then
			break
		end
		dir = parent(dir)
	end
	return found
end

-- The entries of `path`, parsing only when its content changed; nil when it
-- can't be read (gone since `exists`, a directory, no permission) or is too
-- large.
local function load(path: string): Parsed?
	local ok, text = pcall(tern.fs.read, path, MAX_BYTES)
	if not ok then
		return nil
	end
	local hit = parsed[path]
	if hit and hit.text == text then
		return hit
	end
	local entries, problems = envfile.parse(text)
	for _, problem in problems do
		tern.log.warn(string.format("%s %s", path, problem))
	end
	local fresh = { text = text, entries = entries }
	parsed[path] = fresh
	return fresh
end

-- Persist the bounded bytes, not a path-only trust flag. KV failures deny.
-- Forget approval as soon as different or unreadable content is observed.
local function approved(path: string, file: Parsed?): boolean
	local ok, text = pcall(tern.kv.get, APPROVAL .. path)
	if not ok then
		invalidated[path] = true
		return false
	end
	if not invalidated[path] and file and type(text) == "string" and #text <= MAX_BYTES and text == file.text then
		return true
	end
	if text ~= nil then
		invalidated[path] = true
		pcall(tern.kv.set, APPROVAL .. path, nil)
	end
	return false
end

-- The files that apply in `cwd` and their contents.
local function applied(cwd: string): Applied
	local result: Applied = { paths = {}, texts = {} }
	for _, path in chain(cwd) do
		local file = load(path)
		local allowed = approved(path, file)
		if file and allowed then
			table.insert(result.paths, path)
			result.texts[path] = file.text
		end
	end
	return result
end

-- Applies `files` over `env` in place, farthest first, so nearer files win.
-- `${NAME}` sees the keys set so far, then `env`, then the daemon's own
-- environment (`HOME`, `PATH`, …).
local function apply(files: Applied, env: { [string]: string })
	for _, path in files.paths do
		for _, entry in parsed[path].entries do
			if string.sub(entry.key, 1, 5) == "TERN_" then
				-- The daemon's own variables (TERN_PANE, …) aren't ours to change.
				tern.log.warn(string.format("%s line %d: %s is reserved; skipped", path, entry.line, entry.key))
				continue
			end
			env[entry.key] = envfile.expand(entry.parts, function(name: string): string?
				return env[name] or tern.getenv(name)
			end)
		end
	end
	env.TERN_DIRVARS = table.concat(files.paths, ":")
end

tern.on("spawn", function(spec: SpawnSpec): SpawnSpec?
	if spec.cwd == nil then
		return nil
	end
	local files = applied(spec.cwd)
	local pane = tonumber(spec.env.TERN_PANE)
	if pane then
		started[pane] = files
	end
	if #files.paths == 0 then
		return nil
	end
	apply(files, spec.env)
	return spec
end)

-- Files that apply in `now` but that `before` didn't have or had with other
-- content.
local function news(before: Applied, now: Applied): { string }
	local out = {}
	for _, path in now.paths do
		if before.texts[path] ~= now.texts[path] then
			table.insert(out, path)
		end
	end
	return out
end

tern.on("cwd", function(ev: CwdEvent, cx: EffectCx)
	local before = started[ev.pane]
	if before == nil then
		return -- started before this plugin loaded: unknown baseline
	end
	local changed = news(before, applied(ev.path))
	if #changed == 0 then
		return
	end
	local seen = told[ev.pane] or {}
	told[ev.pane] = seen
	local key = ev.path .. "\0" .. table.concat(changed, "\0")
	if seen[key] then
		return
	end
	seen[key] = true
	if #changed == 1 then
		cx:toast("info", tilde(changed[1]) .. " applies here", "This shell didn't load it; new panes pick it up.")
	else
		local text = string.format("%d .tern-env files apply here", #changed)
		cx:toast("info", text, "This shell didn't load them; new panes pick them up.")
	end
end)

tern.on("pane_exited", function(ev: PaneExitedEvent, _cx: EffectCx)
	started[ev.pane] = nil
	told[ev.pane] = nil
end)

-- An approval always refers to the bytes last displayed, never a fresh
-- unseen file. Re-read at click time to reject edits while the block is open.
type Review = { path: string, text: string, token: string }
type State = { cwd: string?, review: { Review } }

local function button(label: string, action: string): Node
	local node = ui.badge(label, "accent")
	local props = node.p or {}
	props.actions = { click = action }
	node.p = props
	return node
end

tern.block.define("approve", {
	init = function(cx: BlockCx, args: { string }, _saved: unknown): State
		return { cwd = args[1] or cx.cwd, review = {} }
	end,
	title = function(_state: State): string?
		return "Directory Variables"
	end,
	view = function(state: State, _cx: BlockCx): BlockView
		state.review = {}
		local parts: { Node } = {
			ui.text("Review every byte before approving. These variables can execute shell startup code. Approval is for this path and exact content only."),
		}
		if state.cwd == nil or state.cwd == "" then
			table.insert(parts, ui.text("No working directory. Open this block from a project pane, or pass an absolute directory as its argument."))
		else
			for _, path in chain(state.cwd) do
				local file = load(path)
				local allowed = approved(path, file)
				table.insert(parts, ui.text(path))
				if file then
					local token = tern.json.encode({ path = path, text = file.text })
					table.insert(state.review, { path = path, text = file.text, token = token })
					-- Quote each line so controls and escape sequences cannot hide bytes.
					-- Include comments and malformed lines too; no truncated preview.
					local lines: { Node } = {}
					for line in string.gmatch(file.text .. "\n", "(.-)\n") do
						local quoted = string.format("%q", line)
						quoted = string.gsub(quoted, "[^ -~]", function(byte: string): string
							return string.format("\\x%02X", string.byte(byte))
						end)
						table.insert(lines, ui.text(quoted))
					end
					table.insert(parts, ui.lines(lines))
					table.insert(parts, button(
						if allowed then "Revoke approval" else "Approve these exact bytes",
						(if allowed then "revoke=" else "approve=") .. token
					))
				else
					table.insert(parts, ui.text("Rejected: unreadable, nonregular, symlink, or larger than 64 KiB."))
				end
			end
			if #state.review == 0 then
				table.insert(parts, ui.text("No readable .tern-env files here. Nothing will be imported."))
			end
		end
		return {
			main = ui.col(parts),
			dock = ui.row({ button("Refresh files", "refresh") }),
		}
	end,
	event = function(state: State, ev: UiEvent, cx: BlockCx)
		if ev.ev ~= "action" then
			return
		end
		if ev.act == "refresh" then
			cx:render()
			return
		end
		-- A delayed click carries the bytes that its own frame displayed,
		-- even if another render or block restart has replaced state.review.
		local review: Review? = nil
		for _, candidate in state.review do
			if candidate.token == ev.value then
				review = candidate
				break
			end
		end
		if review == nil then
			return
		end
		if ev.act == "revoke" then
			invalidated[review.path] = true
			tern.kv.set(APPROVAL .. review.path, nil)
		elseif ev.act == "approve" then
			local current = load(review.path)
			if current == nil or current.text ~= review.text then
				approved(review.path, current)
				cx:toast("warn", "File changed; not approved", "Refresh and review the new contents.")
			else
				tern.kv.set(APPROVAL .. review.path, review.text)
				invalidated[review.path] = nil
				cx:toast("info", "Exact contents approved", "Only new shells load these variables.")
			end
		end
		cx:render()
	end,
})
