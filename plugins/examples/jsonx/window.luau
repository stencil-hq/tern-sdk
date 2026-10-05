--!strict
-- The window half: `.json` files opened from the Files pane or the palette
-- go to the JSON Explorer block instead of the editor, unless the command
-- "Toggle JSON explorer for .json opens" turned that off.

-- The kv key of the switch; absent means on.
local ROUTE_KEY = "route_json"

-- Read on every open: `tern.kv` is one store for every window, so a toggle
-- in one window applies in the others at once.
local function enabled(): boolean
	return tern.kv.get(ROUTE_KEY) ~= false
end

tern.command({
	id = "toggle_route",
	title = "Toggle JSON explorer for .json opens",
	icon = "braces",
	run = function(cx: WindowCx)
		local on = not enabled()
		tern.kv.set(ROUTE_KEY, on)
		if on then
			cx:toast("success", ".json files open in JSON Explorer", "From the Files pane and the palette")
		else
			cx:toast("info", ".json files open in the editor", "Open with › JSON Explorer still works")
		end
	end,
})

tern.route.open(function(req: OpenRequest, _cx: WindowCx): RouteDecision?
	if not enabled() then
		return nil
	end
	-- Only a plain open a person made in the Files pane or the palette. A
	-- placement (beside, split, preview) or a line to reveal asks for the
	-- editor, and links, drops and `tern open` keep their usual behavior.
	if req.how ~= "default" or req.line ~= nil then
		return nil
	end
	if req.origin ~= "files" and req.origin ~= "palette" then
		return nil
	end
	if string.lower(string.sub(req.path, -5)) ~= ".json" then
		return nil
	end
	return { block = "jsonx.explorer", args = { req.path } }
end)
