--!strict
-- workspaces, window half: opens the tabs a project's
-- `.tern/workspace.json` describes, from the palette or at launch
-- (`TERN_WORKSPACE`), starts new tabs at the root of the workspace the
-- focused pane is in, and names that workspace in the status line.

local workspace = require("./workspace")

local OPEN = "plugin.workspaces.open"
local EDIT = "plugin.workspaces.edit"
local HOME = tern.getenv("HOME")

-- A workspace this window opened (or found restored), and its tab ids.
type Opened = {
	name: string,
	root: string,
	tabs: { [number]: boolean },
}

-- Opened workspaces by root. Only this window's VM sees it, and a plugin
-- reload starts it empty.
local opened: { [string]: Opened } = {}

local function tilde(path: string): string
	if HOME and string.sub(path, 1, #HOME + 1) == HOME .. "/" then
		return "~" .. string.sub(path, #HOME + 1)
	end
	return path
end

local function inside(path: string, root: string): boolean
	return path == root or string.sub(path, 1, #root + 1) == root .. "/" or root == "/"
end

-- The opened workspace holding `path`; the deepest root wins when
-- workspaces nest.
local function owner(path: string?): Opened?
	if path == nil then
		return nil
	end
	local best: Opened? = nil
	for root, entry in opened do
		if inside(path, root) and (best == nil or #root > #best.root) then
			best = entry
		end
	end
	return best
end

local function focused_cwd(cx: WindowCx): string?
	local id = cx.session:focused()
	for _, pane in cx.session:panes() do
		if pane.pane == id then
			return pane.cwd
		end
	end
	return nil
end

-- Builds `w`'s tabs and remembers them.
local function build(cx: WindowCx, w: workspace.Workspace)
	local entry: Opened = { name = w.name, root = w.root, tabs = {} }
	local count = 0
	-- What couldn't start: a tab whose first pane failed is missing, a
	-- split whose new side failed is left out of its tab.
	local problems: { string } = {}
	for _, spec in w.tabs do
		local tab, err = cx.layout:tab(spec)
		if tab then
			entry.tabs[tab] = true
			count += 1
		end
		if err then
			table.insert(problems, err)
		end
	end
	if count == 0 then
		local why = if #problems > 0 then table.concat(problems, "; ") else tilde(w.root)
		cx:toast("error", "Workspace " .. w.name .. " opened no tabs", why)
		return
	end
	opened[w.root] = entry
	-- Panes already inside the root get the segment too.
	tern.chrome.refresh()
	if #problems > 0 then
		cx:toast("error", "Opened " .. w.name .. " incomplete", table.concat(problems, "; "))
	else
		cx:toast("success", "Opened " .. w.name, string.format("%d tabs · %s", count, tilde(w.root)))
	end
end

-- Reads the workspace at `root`, toasting the problem when it doesn't load.
local function read(cx: WindowCx, root: string): workspace.Workspace?
	local ok, result = pcall(workspace.load, root)
	if not ok then
		cx:toast("error", "Workspace not opened", tostring(result))
		return nil
	end
	return result
end

local function open_at(cx: WindowCx, root: string)
	local current = opened[root]
	if current then
		cx:toast("info", current.name .. " is already open", tilde(root))
		return
	end
	local w = read(cx, root)
	if w then
		build(cx, w)
	end
end

tern.command({
	id = "open",
	title = "Open workspace",
	icon = "folder",
	run = function(cx: WindowCx)
		local cwd = focused_cwd(cx)
		local root = if cwd then workspace.find(cwd) else nil
		if root == nil then
			cx:toast("error", "No workspace here", "No .tern/workspace.json above " .. tilde(cwd or "this pane"))
			return
		end
		open_at(cx, root)
	end,
})

tern.command({
	id = "edit",
	title = "Edit workspace file",
	icon = "folder",
	run = function(cx: WindowCx)
		local cwd = focused_cwd(cx)
		local root = if cwd then workspace.find(cwd) else nil
		if root == nil then
			cx:toast("error", "No workspace here", "No .tern/workspace.json above " .. tilde(cwd or "this pane"))
			return
		end
		cx:open(root .. "/.tern/workspace.json", "beside")
	end,
})

tern.bind(if tern.runtime.os == "macos" then "cmd+alt+shift+o" else "ctrl+alt+shift+o", OPEN)

-- At launch: the workspace TERN_WORKSPACE names (its root, or its
-- workspace.json). A restored session already has its tabs, so tabs named
-- like the workspace's and inside its root are adopted instead of opened
-- twice.
tern.on("window_start", function(cx: WindowCx)
	local target = tern.getenv("TERN_WORKSPACE")
	if target == nil or target == "" then
		return
	end
	local first = string.sub(target, 1, 1)
	if first ~= "/" and first ~= "~" then
		cx:toast("error", "TERN_WORKSPACE must be an absolute path", target)
		return
	end
	local root = workspace.resolve("/", target)
	root = string.gsub(root, "/%.tern/workspace%.json$", "")
	local w = read(cx, root)
	if w == nil then
		return
	end
	local names: { [string]: boolean } = {}
	for _, name in w.names do
		names[name] = true
	end
	local restored: Opened = { name = w.name, root = root, tabs = {} }
	for _, tab in cx.session:tabs() do
		if tab.name and names[tab.name] and tab.cwd and inside(tab.cwd, root) then
			restored.tabs[tab.id] = true
		end
	end
	if next(restored.tabs) then
		opened[root] = restored
		tern.chrome.refresh()
	else
		build(cx, w)
	end
end)

tern.on("tab_closed", function(ev: TabEvent, _cx: WindowCx)
	for root, entry in opened do
		if entry.tabs[ev.tab] then
			entry.tabs[ev.tab] = nil
			if next(entry.tabs) == nil then
				opened[root] = nil -- the last of its tabs closed
				tern.chrome.refresh()
			end
		end
	end
end)

-- A new tab from a pane inside an opened workspace starts at its root;
-- anywhere else the built-in runs (returning false).
tern.override("new_tab", function(cx: WindowCx): boolean
	local entry = owner(focused_cwd(cx))
	if entry == nil then
		return false
	end
	return cx.layout:new_tab({ cwd = entry.root }) ~= nil
end)

-- Pure: reads only `opened`, never the disk. Tern caches the answer per
-- pane input, so every change to `opened` calls `tern.chrome.refresh()`.
tern.chrome.status(function(pane: PaneInfo): { StatusSegment }?
	local entry = owner(pane.cwd)
	if entry == nil then
		return nil
	end
	return { { text = entry.name, icon = "folder", command = EDIT } }
end)
