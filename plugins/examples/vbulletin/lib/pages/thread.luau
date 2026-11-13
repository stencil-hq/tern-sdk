-- A thread (SHOWTHREAD): the reply button and page navigation, the thread
-- toolbar (Thread Tools, Search this Thread, Rate Thread, Display Modes),
-- ten postbits a page, Quick Reply, previous and next thread, who is
-- viewing, Similar Threads, Posting Rules and Forum Jump.

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")
local BB = require("../bbcode")
local E = require("../editor")

local PER = 10

local T = {}
T.PER = PER

local RATINGS = { "Excellent", "Good", "Average", "Bad", "Terrible" }

-- The id of the latest post by `name` in `t` before post `n`, for a
-- quote's View Post arrow.
local function finder(t: DB.Thread, n: number): (string) -> number?
	return function(name: string): number?
		local want = string.lower(name)
		for i = n - 1, 1, -1 do
			local p = t.posts[i]
			if p and string.lower(p.user.name) == want then
				return p.id
			end
		end
		return nil
	end
end

local function postbit(state: any, p: DB.Post, hl: { string }?): Node
	local u = p.user
	local t = p.thread
	local first = p == t.first
	local title = p.title or (if first then t.title else nil)
	local icon = p.icon or (if first then t.icon else nil)
	local online = u.online
	local info: { any } = {
		H.div(nil, { P.user(u, "bigusername") }),
		H.div("smallfont", { H.t(u.title) }),
		u.avatar and H.div("smallfont avatar-wrap", {
			H.img("avatars/" .. u.avatar, H.go("member", u.id), u.name .. "'s Avatar"),
		}) or false,
		H.div("smallfont userstats", {
			H.div(nil, { H.t("Join Date: " .. Fmt.month(u.joined)) }),
			u.location and H.div(nil, { H.t("Location: " .. u.location) }) or false,
			H.div(nil, { H.t("Posts: " .. Fmt.num(u.posts)) }),
		}),
	}
	local message: { any } = {
		(title or icon) and H.div("smallfont posttitle", {
			icon and H.img("icons/" .. icon) or false,
			icon and H.t(" ") or false,
			title and H.b(title) or false,
		}) or false,
		(title or icon) and H.el("hr", "posthr") or false,
		H.div("post_message", BB.render(p.body, { hl = hl, find_post = finder(t, p.n) })),
		u.sig and H.div("signature", H.cat({ H.div(nil, { H.t("__________________") }) }, BB.render(u.sig))) or false,
	}
	local locked = t.locked or (DB.forum[t.forum] and DB.forum[t.forum].closed)
	return H.div("postwrap", {
		H.tborder({
			H.row({
				H.td("thead postdate grow", {
					H.img(if DB.post_new(p) then "statusicon/post_new" else "statusicon/post_old", nil, if DB.post_new(p) then "Unread" else "Old"),
					H.t(" " .. Fmt.stamp(p.time)),
				}),
				H.td("thead postno r", { H.t("#"), H.a(tostring(p.n), H.go("post", p.id), "b", { title = "Link to this post" }) }),
			}),
			H.row({
				H.td("alt2 w-userinfo", info),
				H.td("alt1 grow postbody", message),
			}),
			H.row({
				H.td("alt2 w-userinfo", {
					H.img(if online then "statusicon/user_online" else "statusicon/user_offline", nil, u.name .. (if online then " is online now" else " is offline")),
				}),
				H.td("alt1 grow r postbtns", {
					not locked and H.wrap(H.go("reply", t.id, p.id), { H.el("span", "pbtn", { H.t("Quote") }) }, nil, { title = "Reply With Quote" }) or false,
					not locked and H.wrap("qr=" .. p.id, { H.el("span", "pbtn", { H.t("Quick Reply") }) }, nil, { title = "Quick reply to this message" }) or false,
				}),
			}),
		}, "post"),
	}, { key = "p" .. p.id })
end

local function toolbar(state: any, t: DB.Thread): Node
	local menu = state.menu
	local vote = DB.my_vote(t)
	local rate: { any }
	if vote then
		rate = { H.div("vbmenu_option", { H.t("You have already rated this thread (" .. RATINGS[6 - vote] .. ").") }) }
	else
		rate = { P.menu_head("Rate This Thread") }
		for i, label in RATINGS do
			table.insert(rate, H.div("vbmenu_option", {
				H.img("rating/rating_" .. (6 - i)),
				H.t(" " .. label),
			}, { actions = { click = "vote=" .. (6 - i) }, key = "r" .. i }))
		end
	end
	return H.tborder({
		H.row({
			H.td("tcat grow", { H.t(" ") }),
			P.menu("threadtools", "Thread Tools", menu == "threadtools", {
				P.menu_option("Show Printable Version", H.go("print", t.id)),
				P.menu_option(if DB.subscribed(t) then "Unsubscribe from this Thread" else "Subscribe to this Thread", "subscribe"),
			}, "tool right"),
			P.menu("threadsearch", "Search this Thread", menu == "threadsearch", {
				P.menu_head("Search this Thread"),
				H.div("vbmenu_option form", {
					H.div("qs", {
						P.input(state, "ts", { aria = "Search this Thread" }),
						H.button("Go", "submit=ts"),
					}),
				}, { key = "tsform" }),
				P.menu_option("Advanced Search", H.go("search")),
			}, "tool right"),
			P.menu("threadrating", "Rate Thread", menu == "threadrating", rate, "tool right"),
			P.menu("displaymodes", "Display Modes", menu == "displaymodes", {
				H.div("vbmenu_option on", { H.b("Linear Mode") }),
				H.div("vbmenu_option dim", { H.t("Switch to Hybrid Mode") }),
				H.div("vbmenu_option dim", { H.t("Switch to Threaded Mode") }),
			}, "tool right"),
		}, "tcatrow"),
	}, "threadtoolbar", { key = "toolbar" })
end

local function controls(t: DB.Thread, pg: number, pages: number, locked: boolean, hl: string?, key: string): Node
	return H.div("controls", {
		if locked
			then H.el("span", "vbbtn closed", { H.t("Closed Thread") })
			else H.wrap(H.go("reply", t.id), { H.el("span", "vbbtn", { H.t("↩ Post Reply") }) }),
		P.pagenav(pg, pages, function(n)
			return H.go("thread", t.id, n, hl)
		end, #t.posts, PER),
	}, { key = key })
end

local function quick_reply(state: any, t: DB.Thread): Node
	local shut = DB.collapsed("qr")
	return H.div("qr-wrap", {
		E.errors(state.errors),
		H.tborder({
			H.row({ H.td("tcat", { H.el("span", "tcat-right", { P.collapser("qr") }), H.t("Quick Reply") }) }),
			not shut and H.row({
				H.td("panelsurround c", {
					H.div("panel", {
						H.div("panel-inner", {
							H.div("smallfont qr-who", { H.t("Logged in as "), P.user(DB.me) }),
							E.editor(state, "qr", false),
							H.div("fieldset", {
								H.div("legend", { H.t("Options") }),
								P.check("checkbox", "qrquote", "1", state.form.qrquote == true, "Quote message in reply?"),
							}),
						}),
					}),
					H.div("buttons", {
						H.button("Post Quick Reply", "submit=qr"),
						H.t(" "),
						H.button("Go Advanced", "advanced=" .. t.id),
					}),
				}),
			}) or false,
		}, "qrbox"),
	}, { key = "qr" })
end

local function viewing(state: any, t: DB.Thread): Node
	local names = { P.user(DB.me) }
	local n = 1
	for id, loc in P.locations(state) do
		if loc.thread == t and id ~= DB.me.id then
			table.insert(names, P.user(DB.user[id]))
			n += 1
		end
	end
	local guests = t.id % 3
	return H.tborder({
		H.row({
			H.td("tcat", {
				H.t(string.format("Currently Active Users Viewing This Thread: %d ", n + guests)),
				H.t(string.format("(%d members and %d guests)", n, guests), "normal"),
			}),
		}),
		H.row({ H.td("alt1", { H.div("smallfont", H.join(names, ", ")) }) }),
	}, nil, { key = "viewing" })
end

local function similar(t: DB.Thread): Node | false
	local list = DB.similar(t)
	if #list == 0 then
		return false
	end
	local rows: { any } = {
		H.row({ H.td("tcat", { H.t("Similar Threads") }) }),
		H.row({
			H.td("thead s-thread", { H.t("Thread") }),
			H.td("thead s-starter c", { H.t("Thread Starter") }),
			H.td("thead s-forum c", { H.t("Forum") }),
			H.td("thead s-replies c", { H.t("Replies") }),
			H.td("thead s-last c", { H.t("Last Post") }),
		}, "headrow"),
	}
	for _, s in list do
		local f = DB.forum[s.forum]
		table.insert(rows, H.row({
			H.td("alt1 s-thread", { H.a(s.title, H.go("thread", s.id), "smallfont") }),
			H.td("alt2 s-starter c", { H.el("span", "smallfont", { P.user(s.starter) }) }),
			H.td("alt1 s-forum c", { f and H.a(f.title, H.go("forum", f.id), "smallfont") or false }),
			H.td("alt2 s-replies c", { H.t(Fmt.num(#s.posts - 1), "smallfont") }),
			H.td("alt1 s-last r", { H.el("span", "smallfont", { H.t(Fmt.date(s.last.time) .. " "), H.t(Fmt.time(s.last.time), "time") }) }),
		}, nil, { key = "s" .. s.id }))
	end
	return H.tborder(rows, "similar", { key = "similar" })
end

-- The page `p` of thread `t` sits on.
function T.page_of(p: DB.Post): number
	return math.ceil(p.n / PER)
end

function T.view(state: any): any
	local t = DB.thread[state.route[2] or 0]
	if not t then
		return {
			crumbs = {},
			title = "Message",
			body = { require("./misc").message("No Thread specified. If you followed a valid link, please notify the administrator.") },
		}
	end
	local f = DB.forum[t.forum]
	state.where = "Viewing Thread " .. t.title
	local crumbs = {}
	if f then
		for _, a in DB.trail(f) do
			table.insert(crumbs, { a.title, H.go("forum", a.id) })
		end
		table.insert(crumbs, { f.title, H.go("forum", f.id) })
	end
	local pages = math.max(1, math.ceil(#t.posts / PER))
	local pg = math.clamp(tonumber(state.route[3]) or 1, 1, pages)
	local hl = nil
	if state.route[4] and state.route[4] ~= "" then
		hl = {}
		for w in string.gmatch(tostring(state.route[4]), "[^+]+") do
			table.insert(hl, w)
		end
	end
	local locked = t.locked or (f ~= nil and f.closed)
	local posts: { any } = {}
	for i = (pg - 1) * PER + 1, math.min(#t.posts, pg * PER) do
		table.insert(posts, postbit(state, t.posts[i], hl))
	end
	local older, newer = DB.neighbours(t)
	local body: { any } = {
		controls(t, pg, pages, locked, state.route[4], "ctl_top"),
		toolbar(state, t),
		H.div("posts", posts, { key = "posts" }),
		controls(t, pg, pages, locked, state.route[4], "ctl_bottom"),
		H.div("gap"),
		not locked and quick_reply(state, t) or false,
		H.div("smallfont c prevnext", {
			H.b("« "),
			older and H.a("Previous Thread", H.go("thread", older.id)) or H.t("Previous Thread", "shade"),
			H.t(" | "),
			newer and H.a("Next Thread", H.go("thread", newer.id)) or H.t("Next Thread", "shade"),
			H.b(" »"),
		}, { key = "prevnext" }),
		H.div("gap"),
		viewing(state, t),
		H.div("gap"),
		similar(t),
		H.div("gap"),
		H.div("bottombar", {
			P.rules(locked),
			H.div("grow"),
			P.forumjump(state, t.forum),
		}, { key = "bottom" }),
	}
	return { crumbs = crumbs, title = t.title, body = body }
end

-- The printable version: every post in plain boxes (printthread).
function T.print(state: any): any
	local t = DB.thread[state.route[2] or 0]
	if not t then
		return T.view(state)
	end
	local items: { any } = {}
	for _, p in t.posts do
		table.insert(items, H.div("fieldset printpost", {
			H.div("legend", { H.t(p.user.name) }),
			H.div("smallfont", { H.t(Fmt.stamp(p.time)) }),
			H.el("hr", "posthr"),
			H.div("post_message", BB.render(p.body)),
		}, { key = "p" .. p.id }))
	end
	return {
		crumbs = { { t.title, H.go("thread", t.id) } },
		title = "Printable Version",
		body = {
			H.div("printable", H.cat({
				H.div("print-head", {
					H.b(DB.title),
					H.t(" > "),
					H.t((DB.forum[t.forum] or { title = "" }).title),
					H.t(" > "),
					H.b(t.title),
				}),
				H.div("smallfont", { H.a("View Full Version : " .. t.title, H.go("thread", t.id)) }),
			}, items), { key = "print" }),
		},
	}
end

return T
