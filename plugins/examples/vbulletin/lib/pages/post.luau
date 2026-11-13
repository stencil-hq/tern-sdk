-- Posting: Post New Thread (newthread.php) and Reply to Thread
-- (newreply.php), each with the title box, the editor and smilies, post
-- icons, Preview, and for replies the Thread Review.

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")
local BB = require("../bbcode")
local E = require("../editor")

local Post = {}

local ICONS = {
	"page", "arrow", "lightbulb", "exclamation", "question", "cool", "smile",
	"angry", "unhappy", "talking", "redface", "wink", "thumbsdown", "thumbsup",
}

local function icons(state: any): Node
	local cells: { any } = {}
	for _, name in ICONS do
		table.insert(cells, H.el("label", "check iconpick", {
			H.el("input", nil, nil, { type = "radio", name = "icon", value = name, checked = state.form.icon == name }),
			H.img("icons/" .. name),
		}, { key = name }))
	end
	return H.div("fieldset", {
		H.div("legend", { H.t("Post Icons") }),
		H.div("smallfont", { H.t("You may choose an icon for your message from the following list:") }),
		H.div("iconlist", cells),
		H.div("smallfont", {
			P.check("radio", "icon", "", state.form.icon == "", "No icon"),
		}),
	}, { key = "icons" })
end

local function form(state: any, opts: { head: string, title_field: string, body_field: string, submit: string, label: string }): { any }
	return {
		E.errors(state.errors),
		state.preview and E.preview(state.f[opts.title_field].t, state.f[opts.body_field].t) or false,
		H.tborder({
			H.row({ H.td("tcat", { H.t(opts.head) }) }),
			H.row({
				H.td("panelsurround c", {
					H.div("panel", {
						H.div("panel-inner", {
							H.div("smallfont qr-who", { H.t("Logged in as "), P.user(DB.me) }),
							H.div("smallfont", { H.t("Title:") }),
							P.input(state, opts.title_field, { aria = "Title" }),
							H.div("gap-s"),
							H.div("smallfont", { H.t("Message:") }),
							E.editor(state, opts.body_field, true),
							icons(state),
						}),
					}),
					H.div("buttons", {
						H.button(opts.label, opts.submit),
						H.t(" "),
						H.button("Preview Post", "preview"),
					}),
				}),
			}),
		}, nil, { key = "postform" }),
	}
end

function Post.newthread(state: any): any
	local f = DB.forum[state.route[2] or 0]
	if not f or f.kind ~= "forum" then
		return { crumbs = {}, title = "Message", body = { require("./misc").message("Invalid Forum specified. If you followed a valid link, please notify the administrator.") } }
	end
	state.where = "Posting New Thread in " .. f.title
	local crumbs = {}
	for _, a in DB.trail(f) do
		table.insert(crumbs, { a.title, H.go("forum", a.id) })
	end
	table.insert(crumbs, { f.title, H.go("forum", f.id) })
	return {
		crumbs = crumbs,
		title = "Post New Thread",
		body = form(state, {
			head = "Post New Thread",
			title_field = "nt_title",
			body_field = "nt_body",
			submit = "submit=newthread",
			label = "Submit New Thread",
		}),
	}
end

function Post.reply(state: any): any
	local t = DB.thread[state.route[2] or 0]
	if not t then
		return { crumbs = {}, title = "Message", body = { require("./misc").message("No Thread specified. If you followed a valid link, please notify the administrator.") } }
	end
	state.where = "Replying to Thread " .. t.title
	local f = DB.forum[t.forum]
	local crumbs = {}
	if f then
		for _, a in DB.trail(f) do
			table.insert(crumbs, { a.title, H.go("forum", a.id) })
		end
		table.insert(crumbs, { f.title, H.go("forum", f.id) })
	end
	table.insert(crumbs, { t.title, H.go("thread", t.id) })
	local body = form(state, {
		head = "Reply to Thread",
		title_field = "rp_title",
		body_field = "rp_body",
		submit = "submit=reply",
		label = "Submit Reply",
	})
	local review: { any } = {
		H.row({ H.td("tcat", { H.t("Thread Review (Newest First)") }) }),
		H.row({ H.td("thead w-review", { H.t("Posted By") }), H.td("thead grow", { H.t("Message") }) }, "headrow"),
	}
	for i = #t.posts, math.max(1, #t.posts - 9), -1 do
		local p = t.posts[i]
		table.insert(review, H.row({
			H.td("alt2 w-review", { P.user(p.user, "b") }),
			H.td("alt1 grow", {
				H.div("smallfont shade", { H.t(Fmt.stamp(p.time)) }),
				H.el("hr", "posthr"),
				H.div("post_message", BB.render(p.body)),
			}),
		}, nil, { key = "p" .. p.id }))
	end
	table.insert(body, H.div("gap"))
	table.insert(body, H.tborder(review, "review", { key = "review" }))
	return { crumbs = crumbs, title = "Reply to Thread", body = body }
end

return Post
