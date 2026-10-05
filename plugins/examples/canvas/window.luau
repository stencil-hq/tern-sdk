--!strict
local tern = require("tern")

local function dashboard(status: string)
	return tern.ui.col({
		tern.ui.node("text", { key = "status", text = status }),
		tern.ui.node("badge", {
			key = "refresh", text = "Refresh", actions = { click = "refresh" },
		}),
	})
end

tern.command({
	id = "open",
	title = "Open canvas dashboard",
	icon = "sparkle",
	run = function(cx)
		-- Find a surviving canvas after a reload instead of opening another.
		for _, canvas in cx.canvas:list() do
			if canvas.owner == tern.plugin.id and canvas.title == "Dashboard" then
				cx.layout:focus(canvas.pane)
				return
			end
		end
		local pane, err = cx.canvas:open({
			title = "Dashboard",
			about = "Refresh replaces the timestamp; it runs no programs.",
			view = dashboard("Not refreshed yet"),
			how = "beside",
			focus = false,
		})
		if not pane then
			cx:toast("error", "Dashboard not opened", err)
		end
	end,
})

tern.on("canvas_action", function(ev, cx)
	if ev.act ~= "refresh" then
		return false
	end
	cx.canvas:set(ev.pane, { set = {
		status = tern.ui.node("text", {
			key = "status", text = string.format("Refreshed at %.0f ms", tern.now()),
		}),
	} })
	return true
end)
