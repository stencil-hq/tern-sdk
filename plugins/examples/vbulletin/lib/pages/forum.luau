-- A forum (FORUMDISPLAY): its sub-forums, the Post New Thread button and
-- page navigation, the thread list with sortable columns, Display Options,
-- Posting Rules, the icon legend and Forum Jump. A category shows its
-- forums only.

local H = require("../h")
local DB = require("../db")
local P = require("../parts")
local BB = require("../bbcode")

local PER = 20

local SORTS = {
	{ value = "title", label = "Thread Title" },
	{ value = "last", label = "Last Post Time" },
	{ value = "replies", label = "Number of Replies" },
	{ value = "views", label = "Number of Views" },
	{ value = "starter", label = "Thread Starter" },
	{ value = "rating", label = "Thread Rating" },
}

local function crumbs(f: DB.Forum): { { any } }
	local out = {}
	for _, a in DB.trail(f) do
		table.insert(out, { a.title, H.go("forum", a.id) })
	end
	return out
end

-- A sortable column head: a link, with the ▲/▼ arrow on the sorted one.
local function sorthead(state: any, label: string, by: string): { any }
	local s = state.tsort
	local on = s.by == by
	return {
		H.a(label, "sort=" .. by),
		on and H.t(" ") or false,
		on and H.img(if s.asc then "buttons/sortasc" else "buttons/sortdesc", "sort=" .. by, "Reverse Sort Order") or false,
	}
end

local function controls(state: any, f: DB.Forum, pg: number, pages: number, total: number): Node
	return H.div("controls", {
		if f.closed
			then H.div("smallfont", { H.el("span", "vbbtn closed", { H.t("Forum Closed") }) })
			else H.wrap(H.go("newthread", f.id), { H.el("span", "vbbtn", { H.t("+ Post New Thread") }) }),
		P.pagenav(pg, pages, function(n)
			return H.go("forum", f.id, n)
		end, total, PER),
	})
end

local function display_options(state: any, from: number, to: number, total: number): Node
	local s = state.tsort
	return H.tborder({
		H.row({ H.td("thead", { H.el("span", "tcat-right", { P.collapser("dispopts") }), H.t("Display Options") }) }),
		not DB.collapsed("dispopts") and H.row({
			H.td("alt1 dispopts", {
				H.div("smallfont", { H.t(string.format("Showing threads %d to %d of %d", from, to, total)) }),
				H.div("dispopts-row smallfont", {
					H.div("opt", {
						H.div(nil, { H.t("Sorted By") }),
						P.select("sortby", SORTS, s.by, "sortby", state.menu == "sel:sortby", "w-sort"),
					}),
					H.div("opt", {
						H.div(nil, { H.t("Sort Order") }),
						P.select("order", {
							{ value = "asc", label = "Ascending" },
							{ value = "desc", label = "Descending" },
						}, if s.asc then "asc" else "desc", "order", state.menu == "sel:order", "w-order"),
					}),
					H.div("opt", {
						H.div(nil, { H.t("From The") }),
						P.select("from", { { value = "all", label = "Beginning" } }, "all", "from", state.menu == "sel:from", "w-order"),
					}),
					H.div("opt bottom", { H.button("Show Threads", "reload") }),
				}),
			}),
		}) or false,
	}, "dispbox", { key = "dispopts" })
end

local function threads(state: any, f: DB.Forum, list: { DB.Thread }, pg: number): Node
	local wide = state.wide
	local rows: { any } = {
		H.row({
			H.td("tcat", {
				H.el("span", "tcat-right smallfont", {
					H.b("Threads in Forum"),
					H.t(" : " .. f.title),
				}),
				H.t(" "),
			}),
			P.menu("forumtools", "Forum Tools", state.menu == "forumtools", {
				P.menu_option("Mark This Forum Read", "markread"),
				P.menu_option("View Parent Forum", if f.parent then H.go("forum", f.parent) else H.go("home")),
			}, "tool right"),
			P.menu("forumsearch", "Search this Forum", state.menu == "forumsearch", {
				P.menu_head("Search this Forum"),
				H.div("vbmenu_option form", {
					H.div("qs", {
						P.input(state, "fs", { aria = "Search this Forum" }),
						H.button("Go", "submit=fs"),
					}),
				}, { key = "fsform" }),
				P.menu_option("Advanced Search", H.go("search")),
			}, "tool right"),
		}, "tcatrow"),
		H.row({
			H.td("thead w-status2", { H.t(" ") }),
			H.td("thead w-thread", {
				H.el("span", "tb-right", sorthead(state, "Rating", "rating")),
				H.el("span", nil, H.cat(sorthead(state, "Thread", "title"), H.cat({ H.t(" / ") }, sorthead(state, "Thread Starter", "starter")))),
			}),
			H.td("thead w-tlast c", sorthead(state, "Last Post", "last")),
			wide and H.td("thead w-count c", sorthead(state, "Replies", "replies")) or false,
			wide and H.td("thead w-count c", sorthead(state, "Views", "views")) or false,
		}, "headrow"),
	}
	local from = (pg - 1) * PER + 1
	local shown_sticky = false
	local opts = { wide = wide, strip = BB.strip, per = 10 }
	for i = from, math.min(#list, pg * PER) do
		local t = list[i]
		if t.sticky then
			shown_sticky = true
		elseif shown_sticky then
			shown_sticky = false
			table.insert(rows, H.row({ H.td("thead sep", { H.t("Normal Threads") }) }, nil, { key = "normal" }))
		end
		table.insert(rows, P.threadbit(t, opts))
	end
	if #list == 0 then
		table.insert(rows, H.row({
			H.td("alt1 empty", {
				H.div("smallfont", { H.t("There are no threads in this forum.") }),
			}),
		}))
	end
	return H.tborder(rows, "threadslist", { key = "threads" })
end

return function(state: any): any
	local id = state.route[2]
	local f = DB.forum[id or 0]
	if not f then
		return {
			crumbs = {},
			title = "Message",
			body = { require("./misc").message("Invalid Forum specified. If you followed a valid link, please notify the administrator.") },
		}
	end
	state.where = "Viewing Forum " .. f.title
	local body: { any } = {}
	if #f.children > 0 then
		local rows: { any } = {
			H.row({ H.td("tcat", { H.t(if f.kind == "category" then f.title else "Sub-Forums : " .. f.title) }) }),
			P.forumhead(state.wide),
		}
		local where = P.locations(state)
		for _, c in f.children do
			table.insert(rows, P.forumbit(c, where, state.wide))
		end
		table.insert(body, H.tborder(rows, "forumlist", { key = "subs" }))
		table.insert(body, H.div("gap"))
	end
	if f.kind == "category" then
		table.insert(body, H.div("bottombar", { H.div("grow"), P.forumjump(state, f.id) }, { key = "bottom" }))
		return { crumbs = crumbs(f), title = f.title, body = body }
	end
	local list = DB.forum_threads(f, state.tsort.by, state.tsort.asc)
	local pages = math.max(1, math.ceil(#list / PER))
	local pg = math.clamp(tonumber(state.route[3]) or 1, 1, pages)
	table.insert(body, controls(state, f, pg, pages, #list))
	table.insert(body, threads(state, f, list, pg))
	table.insert(body, controls(state, f, pg, pages, #list))
	table.insert(body, H.div("gap"))
	table.insert(body, display_options(state, math.min((pg - 1) * PER + 1, #list), math.min(pg * PER, #list), #list))
	table.insert(body, H.div("gap"))
	table.insert(body, H.div("bottombar", {
		P.rules(f.closed),
		H.div("grow"),
		H.div("bottomright", { P.thread_legend(), P.forumjump(state, f.id) }),
	}, { key = "bottom" }))
	return { crumbs = crumbs(f), title = f.title, body = body }
end
