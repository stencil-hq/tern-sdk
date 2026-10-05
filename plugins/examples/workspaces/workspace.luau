--!strict
-- Finds and reads `.tern/workspace.json`, turning it into the tab specs
-- `cx.layout:tab` builds.
--
-- {
--   "name": "webapp",                      -- default: the root's folder name
--   "tabs": [{
--     "name": "dev",                       -- tab name
--     "cwd": "web",                        -- relative to the root (default: the root)
--     "split": "right",                    -- how `panes` divide (default "right")
--     "panes": [                           -- default: one shell
--       {"command": "npm run dev"},        -- a login shell runs it with -c
--       {"block": "longrun.slow", "args": []},
--       {"cwd": "api"},                    -- a shell, relative to the tab's cwd
--       {"split": "down", "panes": [...]}  -- a group, divided its own way
--     ]
--   }]
-- }

local workspace = {}

local FILE = ".tern/workspace.json"
local HOME = tern.getenv("HOME")
local SIDES: { [string]: boolean } = { right = true, down = true, left = true, up = true }

export type Workspace = {
	name: string,
	root: string,
	tabs: { TabSpec },
	-- The tab names, to recognize a restored session's tabs.
	names: { string },
}

-- `path` without trailing slashes.
local function trim(path: string): string
	local out = string.gsub(path, "(.)/+$", "%1")
	return out
end

-- `path` made absolute against `base`; `~` is the home directory.
function workspace.resolve(base: string, path: string): string
	if path == "~" or string.sub(path, 1, 2) == "~/" then
		if HOME == nil then
			error("HOME is not set, so " .. path .. " can't be resolved", 0)
		end
		return trim(HOME .. string.sub(path, 2))
	end
	if string.sub(path, 1, 1) == "/" then
		return trim(path)
	end
	path = string.gsub(path, "^%./", "")
	if path == "" or path == "." then
		return base
	end
	return trim(if base == "/" then "/" .. path else base .. "/" .. path)
end

-- The nearest directory at or above `dir` holding `.tern/workspace.json`.
function workspace.find(dir: string): string?
	local at: string? = trim(dir)
	while at do
		local candidate = if at == "/" then "/" .. FILE else at .. "/" .. FILE
		if tern.fs.exists(candidate) then
			return at
		end
		if at == "/" then
			return nil
		end
		local up = string.match(at, "^(.*)/[^/]*$")
		at = if up == nil or up == "" then "/" else up
	end
	return nil
end

-- Raises `where: message` (no Lua position: the toast shows it as is).
local function fail(where: string, message: string): never
	error(string.format("%s: %s", where, message), 0)
end

local function optional_string(value: any, where: string): string?
	if value ~= nil and type(value) ~= "string" then
		fail(where, "must be a string")
	end
	return value
end

local function side(value: any, where: string): Dir
	local name = optional_string(value, where) or "right"
	if not SIDES[name] then
		fail(where, "must be right, down, left or up")
	end
	return name :: Dir
end

local node: (entry: any, cwd: string, where: string) -> TabSpec

-- `list` divided `split`-wise: the first half on one side, the rest on the
-- other, recursively. With an odd count the first side holds fewer panes,
-- so the first pane (often an editor) gets the most room.
local function group(list: { any }, split: Dir, cwd: string, where: string): TabSpec
	if #list == 0 then
		return {}
	end
	local function range(from: number, to: number): TabSpec
		if from == to then
			return node(list[from], cwd, string.format("%s[%d]", where, from))
		end
		local middle = from + (to - from + 1) // 2 - 1
		return { split = split, range(from, middle), range(middle + 1, to) }
	end
	return range(1, #list)
end

-- One `panes` entry: a group (it has `panes`) or a leaf.
node = function(entry: any, cwd: string, where: string): TabSpec
	if type(entry) ~= "table" then
		fail(where, "must be an object")
	end
	local own = optional_string(entry.cwd, where .. ".cwd")
	local here = if own then workspace.resolve(cwd, own) else cwd
	if entry.panes ~= nil then
		if type(entry.panes) ~= "table" then
			fail(where .. ".panes", "must be an array")
		end
		local spec = group(entry.panes, side(entry.split, where .. ".split"), here, where .. ".panes")
		spec.cwd = spec.cwd or here
		return spec
	end
	local command = optional_string(entry.command, where .. ".command")
	local block = optional_string(entry.block, where .. ".block")
	if command and block then
		fail(where, "has both command and block")
	end
	local args: { string }? = nil
	if entry.args ~= nil then
		if block == nil then
			fail(where .. ".args", "only blocks take args")
		end
		if type(entry.args) ~= "table" then
			fail(where .. ".args", "must be an array of strings")
		end
		args = {}
		for i, arg in entry.args do
			table.insert(args :: { string }, optional_string(arg, string.format("%s.args[%d]", where, i)) :: string)
		end
	end
	return { cwd = here, launch = { command = command, block = block, args = args } }
end

-- The workspace at `root`; raises a readable message when the file is
-- missing or malformed.
function workspace.load(root: string): Workspace
	local path = root .. "/" .. FILE
	local ok, text = pcall(tern.fs.read, path)
	if not ok then
		fail(path, "can't be read")
	end
	local decoded, data = pcall(tern.json.decode, text)
	if not decoded then
		fail(path, "isn't valid JSON (" .. tostring(data) .. ")")
	end
	if type(data) ~= "table" then
		fail(path, "must hold an object")
	end
	local folder = string.match(root, "([^/]+)$") or root
	local name = optional_string(data.name, "name") or folder
	if type(data.tabs) ~= "table" or #data.tabs == 0 then
		fail("tabs", "must be a non-empty array")
	end
	local tabs: { TabSpec } = {}
	local names: { string } = {}
	for i, tab in data.tabs do
		local where = string.format("tabs[%d]", i)
		if type(tab) ~= "table" then
			fail(where, "must be an object")
		end
		local cwd = workspace.resolve(root, optional_string(tab.cwd, where .. ".cwd") or ".")
		local panes = tab.panes or {}
		if type(panes) ~= "table" then
			fail(where .. ".panes", "must be an array")
		end
		local spec = group(panes, side(tab.split, where .. ".split"), cwd, where .. ".panes")
		spec.cwd = spec.cwd or cwd
		spec.name = optional_string(tab.name, where .. ".name")
		if spec.name then
			table.insert(names, spec.name :: string)
		end
		table.insert(tabs, spec)
	end
	return { name = name, root = root, tabs = tabs, names = names }
end

return workspace
