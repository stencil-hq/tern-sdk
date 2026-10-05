--!strict
-- The window half: a palette command and a key bind that open the queue, or
-- focus the one already open in this window.

local KIND = "review-queue.queue"

local function open_queue(cx: WindowCx)
	-- One queue per window is enough: each open block polls GitHub.
	for _, pane in cx.session:panes() do
		if string.sub(pane.title, 1, #"Review Queue") == "Review Queue" then
			cx.layout:focus(pane.pane)
			return
		end
	end
	if cx:new_block(KIND, nil, "beside") == nil then
		cx:toast("error", "Review Queue is not loaded on this host", "Install the plugin where your panes run.")
	end
end

tern.command({
	id = "open",
	title = "Open review queue",
	icon = "merge",
	run = open_queue,
})

tern.bind("alt+shift+cmd+r", "plugin.review-queue.open")
