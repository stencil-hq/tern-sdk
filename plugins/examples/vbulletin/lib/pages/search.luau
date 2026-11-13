-- Searching (search.php): the Search Forums form with its options, and the
-- results as threads or as posts, which New Posts and Today's Posts share.

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")
local BB = require("../bbcode")
local E = require("../editor")

local S = {}

S.PER = 25

local function fieldset(legend: string, c: { any }, key: string): Node
	return H.div("fieldset", H.cat({ H.div("legend", { H.t(legend) }) }, c), { key = key })
end

function S.form(state: any): any
	state.where = "Searching Forums"
	local form = state.form
	local menu = state.menu
	local forums = P.forum_options({ value = "all", label = "Search All Open Forums" })
	return {
		crumbs = {},
		title = "Search Forums",
		body = {
			E.errors(state.errors),
			H.tborder({
				H.row({ H.td("tcat", { H.t("Search Forums") }) }),
				H.row({
					H.td("panelsurround c", {
						H.div("panel", {
							H.div("cols2", {
								fieldset("Search by Keyword", {
									H.div("fs-row", {
										H.div(nil, { H.t("Keyword(s):") }),
										P.input(state, "kw", { aria = "Keyword(s)" }),
									}),
									H.div("fs-row", {
										P.select("titles", {
											{ value = "0", label = "Search Entire Posts" },
											{ value = "1", label = "Search Titles Only" },
										}, form.titles, "set=titles", menu == "sel:titles", "w-250"),
									}),
								}, "kwset"),
								fieldset("Search by User Name", {
									H.div("fs-row", {
										H.div(nil, { H.t("User Name:") }),
										P.input(state, "su", { aria = "User Name" }),
									}),
									H.div("fs-row", {
										P.select("starter", {
											{ value = "0", label = "Find Posts by User" },
											{ value = "1", label = "Find Threads Started by User" },
										}, form.starter, "set=starter", menu == "sel:starter", "w-250"),
										P.check("checkbox", "exact", "1", form.exact == true, "Exact name"),
									}),
								}, "userset"),
							}),
						}),
						H.div("buttons", {
							H.button("Search Now", "submit=search"),
							H.t(" "),
							H.button("Reset Fields", "reset=search"),
						}),
					}),
				}),
			}, nil, { key = "searchbox" }),
			H.div("gap"),
			H.tborder({
				H.row({ H.td("tcat", { H.el("span", "tcat-right", { P.collapser("searchopts") }), H.t("Search Options") }) }),
				not DB.collapsed("searchopts") and H.row({
					H.td("panelsurround c", {
						H.div("panel", {
							H.div("cols2", {
								H.div("col", {
									fieldset("Find Posts from", {
										P.select("since", {
											{ value = "0", label = "Any Date" },
											{ value = "1", label = "Yesterday" },
											{ value = "7", label = "A Week Ago" },
											{ value = "30", label = "A Month Ago" },
											{ value = "365", label = "A Year Ago" },
										}, form.since, "set=since", menu == "sel:since", "w-150"),
										H.t(" and Newer", "smallfont"),
									}, "sinceset"),
									fieldset("Show Results as", {
										P.check("radio", "showposts", "0", form.showposts ~= "1", "Threads"),
										P.check("radio", "showposts", "1", form.showposts == "1", "Posts"),
									}, "showset"),
								}),
								fieldset("Search in Forum(s)", {
									P.select("forum", forums, form.forum, "set=forum", menu == "sel:forum", "w-250"),
									H.div("fs-row", {
										P.check("checkbox", "child", "1", true, "Search child forums as well"),
									}),
								}, "forumset"),
							}),
						}),
						H.div("buttons", {
							H.button("Search Now", "submit=search"),
							H.t(" "),
							H.button("Reset Fields", "reset=search"),
						}),
					}),
				}) or false,
			}, nil, { key = "optsbox" }),
		},
	}
end

local function summary(r: any, from: number, to: number): Node
	return H.row({
		H.td("tcat grow", {
			H.el("span", "tcat-right smallfont", {
				H.t("Search took "),
				H.b(string.format("%.2f", r.took)),
				H.t(" seconds."),
			}),
			H.t(string.format("Showing results %d to %d of %d", from, to, #r.items)),
		}),
	}, "tcatrow", { key = "summary" })
end

local function thread_rows(state: any, r: any, from: number, to: number): { any }
	local wide = state.wide
	local rows: { any } = {
		summary(r, from, to),
		H.row({
			H.td("thead w-status2", { H.t(" ") }),
			H.td("thead w-thread", { H.t("Thread / Thread Starter") }),
			H.td("thead w-tlast c", { H.t("Last Post") }),
			wide and H.td("thead w-count c", { H.t("Replies") }) or false,
			wide and H.td("thead w-count c", { H.t("Views") }) or false,
			H.td("thead w-tforum", { H.t("Forum") }),
		}, "headrow"),
	}
	local opts = { wide = wide, forum_col = true, hl = r.hl, strip = BB.strip, per = 10 }
	for i = from, to do
		table.insert(rows, P.threadbit(r.items[i], opts))
	end
	return rows
end

local function post_boxes(r: any, from: number, to: number): { any }
	local out: { any } = { H.tborder({ summary(r, from, to) }, nil, { key = "head" }) }
	for i = from, to do
		local p: DB.Post = r.items[i]
		local t = p.thread
		local f = DB.forum[t.forum]
		local go = if r.hl then H.go("post", p.id, r.hl) else H.go("post", p.id)
		table.insert(out, H.div("postwrap", {
			H.tborder({
				H.row({
					H.td("thead grow", {
						H.img(if DB.post_new(p) then "statusicon/post_new" else "statusicon/post_old"),
						H.t(" " .. Fmt.stamp(p.time)),
					}),
				}),
				H.row({
					H.td("alt1 grow", {
						H.div("sr-head", {
							H.div("smallfont", {
								H.el("em", nil, { H.t("Thread") }),
								H.t(": "),
								H.a(t.title, H.go("thread", t.id, 1, r.hl or ""), "b"),
							}),
							f and H.div("smallfont r", { H.el("em", nil, { H.t("Forum") }), H.t(": "), H.a(f.title, H.go("forum", f.id)) }) or false,
						}),
						H.el("hr", "posthr"),
						H.div("smallfont", {
							H.wrap(go, { H.b(if p == t.first then t.title else "Re: " .. t.title) }),
						}),
						H.div("smallfont sr-snippet", { H.wrap(go, { H.el("em", nil, { H.t(Fmt.clip(BB.strip(p.body), 220)) }) }) }),
						H.el("hr", "posthr"),
						H.div("sr-foot smallfont", {
							H.div(nil, {
								H.t(string.format("Replies: %s", Fmt.num(#t.posts - 1))),
								H.t(string.format("   Views: %s", Fmt.num(t.views))),
							}),
							H.div("r", { H.t("Posted By "), P.user(p.user, "b") }),
						}),
					}),
				}),
			}),
		}, { key = "r" .. p.id }))
	end
	return out
end

function S.results(state: any): any
	state.where = "Viewing Search Results"
	local r = state.result
	if not r then
		return S.form(state)
	end
	local total = #r.items
	local pages = math.max(1, math.ceil(total / S.PER))
	local pg = math.clamp(tonumber(state.route[2]) or 1, 1, pages)
	local from, to = (pg - 1) * S.PER + 1, math.min(total, pg * S.PER)
	local nav = P.pagenav(pg, pages, function(n)
		return H.go("results", n)
	end, total, S.PER)
	local body: { any } = {
		H.div("controls", { H.div("smallfont", { r.note and H.t(r.note) or false }), nav }, { key = "ctl_top" }),
	}
	if r.kind == "posts" then
		for _, n in post_boxes(r, from, to) do
			table.insert(body, n)
		end
	else
		table.insert(body, H.tborder(thread_rows(state, r, from, to), "threadslist", { key = "results" }))
	end
	table.insert(body, H.div("controls", { H.div("grow"), nav }, { key = "ctl_bottom" }))
	table.insert(body, H.div("gap"))
	table.insert(body, H.div("bottombar", { H.div("grow"), P.forumjump(state, nil) }, { key = "bottom" }))
	return {
		crumbs = { { "Search Forums", H.go("search") } },
		title = r.title,
		body = body,
	}
end

return S
