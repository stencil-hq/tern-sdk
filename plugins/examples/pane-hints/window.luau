--!strict
-- Temporary large numbers over existing panes, with a native number prompt.
-- No global digit bindings: numbers and Escape belong to the prompt only.

local function collect(tree: TabTree, ids: { number })
	if tree.pane then
		table.insert(ids, tree.pane)
	else
		for index = 1, 2 do
			local child = tree[index]
			if child then
				collect(child, ids)
			end
		end
	end
end

-- Tern 0.9.4 retains all window panes in session:panes() order, including
-- hidden tabs and parked panes. Display numbers still follow the split tree.
-- The selectors are private chrome structure: verify them after Tern upgrades.
local active = 0
local showing = false

local function clear()
	active += 1
	showing = false
	tern.css("hints", "")
end

local function show(cx: WindowCx)
	clear()
	local token = active
	local focused = cx.session:focused()
	local tab = if focused then cx.session:tab_of(focused) else nil
	local layout = if tab then cx.session:layout(tab) else nil
	if not tab or not layout then
		cx:toast("info", "No active tab to select a pane from")
		return
	end

	local ids: { number } = {}
	collect(layout.root, ids)
	for _, float in layout.floats do
		table.insert(ids, float.pane)
	end

	local positions: { [number]: number } = {}
	for position, pane in cx.session:panes() do
		positions[pane.pane] = position
	end
	local panes: { [number]: number } = {}
	local rules = { [[
#panel > .tn-stage > section.tn-pane::after {
 position: absolute; inset: 0;
 display: flex; align-items: center; justify-content: center;
 font-size: 120px; font-weight: 800;
 color: #ffffff; background: rgba(0, 0, 0, 0.35);
 z-index: 999; pointer-events: none;
}
#pick .tn-pk {
 position: absolute !important; left: 50% !important; top: auto !important; bottom: 24px !important;
 transform: translateX(-50%) !important; width: 440px !important; height: 96px !important; min-height: 0 !important; max-height: 96px !important;
}
]] }
	for index, id in ids do
		local pane = cx.session:pane(id)
		if pane and pane.tab == tab and not pane.parked then
			-- Zoom keeps hidden leaves in the DOM; keep their selector positions,
			-- but only offer the focused tiled pane and floating panes.
			if not layout.zoomed or id == layout.focus or pane.floating then
				panes[index] = id
				-- The renderer needs :first-of-type for the first generated glyph.
				local domIndex = positions[id]
				local position = if domIndex == 1 then ":first-of-type" else string.format(":nth-of-type(%d)", domIndex)
				table.insert(rules, string.format(
					"#panel > .tn-stage > section.tn-pane%s::after { content: '%d'; }",
					position, index
				))
			end
		end
	end
	if next(panes) == nil then
		cx:toast("info", "No visible panes in this tab")
		return
	end

	showing = true
	tern.css("hints", table.concat(rules, "\n"))
	cx:prompt({
		placeholder = "Pane number → Enter · Esc to cancel",
		action = "focus pane",
	}):next(function(value: string?, err: string?, nextCx: WindowCx?)
		if token ~= active then
			return -- Layout/focus changed or another invocation replaced this one.
		end
		clear()
		if not nextCx then
			return
		end
		if err then
			nextCx:toast("error", "Could not select a pane", err)
			return
		end
		if value == nil then
			return
		end
		local number = if string.match(value, "^%d+$") then tonumber(value) else nil
		local id = if number then panes[number] else nil
		if not id then
			nextCx:toast("info", "Enter one of the displayed pane numbers")
			return
		end
		local pane = nextCx.session:pane(id)
		local current = nextCx.session:focused()
		if not pane or pane.parked or pane.tab ~= tab then
			nextCx:toast("info", "That pane is no longer in this tab")
			return
		end
		if not current or nextCx.session:tab_of(current) ~= tab then
			return
		end
		local now = nextCx.session:layout(tab)
		local currentIds: { number } = {}
		if now then
			collect(now.root, currentIds)
			for _, float in now.floats do
				table.insert(currentIds, float.pane)
			end
		end
		if not now or now.zoomed ~= layout.zoomed or #currentIds ~= #ids then
			return
		end
		for index, oldId in ids do
			if currentIds[index] ~= oldId then
				nextCx:toast("info", "Pane layout changed; show numbers again")
				return
			end
		end
		nextCx.layout:focus(id)
	end)
end

-- A stale nth-of-type hint could point at a different pane after a mutation.
-- Remove hints rather than renumbering while the user is typing a choice.
for _, event in { "pane_created", "pane_closed", "tab_created", "tab_closed", "focus" } do
	tern.on(event, function()
		if showing then
			clear()
		end
	end)
end

tern.command({
	id = "show",
	title = "Show pane numbers",
	keys = { "ctrl+cmd+g" },
	run = show,
})
