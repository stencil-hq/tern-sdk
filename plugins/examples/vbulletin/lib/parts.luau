-- Template pieces several pages share, each named after the vBulletin 3.8
-- template it draws: forumhome_forumbit, threadbit, pagenav, the popup
-- menus, forumjump, the posting rules box.

local H = require("./h")
local DB = require("./db")
local Fmt = require("./fmt")
local Field = require("./field")

local P = {}

-- A member's name linking to their profile, in their usergroup's markup.
function P.user(u: DB.User, class: string?): Node
	local c = "username g-" .. u.group
	return H.a(u.name, H.go("member", u.id), if class then c .. " " .. class else c)
end

-- The day and time of `t`: "Today <time>03:14 PM</time>".
function P.when(t: number): { Node }
	return { H.t(Fmt.day(t) .. " "), H.t(Fmt.time(t), "time") }
end

-- The pagenav box: "Page 2 of 5  « First < 1 2 [3] 4 5 > Last »".
-- `route(n)` is the go= action of page `n`.
function P.pagenav(pg: number, pages: number, route: (number) -> string, total: number, per: number): Node | false
	if pages <= 1 then
		return false
	end
	local function range(n: number): string
		local from = (n - 1) * per + 1
		return string.format("%d to %d of %d", from, math.min(n * per, total), total)
	end
	local cells: { any } = {
		H.td("vbmenu_control pn-label", { H.t(string.format("Page %d of %d", pg, pages)) }),
	}
	if pg > 3 then
		table.insert(cells, H.td("alt1", { H.a("« First", route(1), "smallfont", { title = "First Page - Results " .. range(1) }) }))
	end
	if pg > 1 then
		table.insert(cells, H.td("alt1", { H.a("<", route(pg - 1), "smallfont", { title = "Prev Page - Results " .. range(pg - 1) }) }))
	end
	for n = math.max(1, pg - 2), math.min(pages, pg + 2) do
		if n == pg then
			table.insert(cells, H.td("alt2", { H.b(tostring(n), "smallfont") }, { title = "Showing results " .. range(n) }))
		else
			table.insert(cells, H.td("alt1", { H.a(tostring(n), route(n), "smallfont", { title = "Show results " .. range(n) }) }))
		end
	end
	if pg < pages then
		table.insert(cells, H.td("alt1", { H.a(">", route(pg + 1), "smallfont", { title = "Next Page - Results " .. range(pg + 1) }) }))
	end
	if pg + 2 < pages then
		table.insert(cells, H.td("alt1", { H.a("Last »", route(pages), "smallfont", { title = "Last Page - Results " .. range(pages) }) }))
	end
	return H.div("pagenav", { H.tborder({ H.row(cells) }, "pn") })
end

-- A navbar or toolbar menu control: `label` with the ▾ arrow, opening the
-- popup `id` holding `items` while `open`.
function P.menu(id: string, label: string, open: boolean, items: { any }, class: string?): Node
	return H.td("vbmenu_control menu" .. (if class then " " .. class else ""), {
		H.wrap("menu=" .. id, { H.t(label), H.t(" "), H.img("buttons/menu_open") }, "menu_label"),
		open and H.div("vbmenu_popup", { H.div("vbmenu_table", items) }, { actions = { click = "noop" } }) or false,
	}, { key = id })
end

-- A popup menu's section title.
function P.menu_head(text: string): Node
	return H.div("thead menu_head", { H.t(text) })
end

-- A popup menu row running `act`.
function P.menu_option(text: string, act: string): Node
	return H.div("vbmenu_option", { H.t(text) }, { actions = { click = act } })
end

export type Option = { value: string, label: string, indent: number? }

-- A drop-down list in the look of a 2008 `<select>`: the chosen option and
-- an arrow; open, the options under it. Picking one runs `act=<value>`.
function P.select(id: string, options: { Option }, value: string, act: string, open: boolean, class: string?): Node
	local current = options[1]
	for _, o in options do
		if o.value == value then
			current = o
		end
	end
	local rows = {}
	if open then
		for _, o in options do
			local label = string.rep("    ", o.indent or 0) .. o.label
			table.insert(rows, H.div(if o.value == value then "sel_opt on" else "sel_opt", { H.t(label) }, {
				actions = { click = act .. "=" .. o.value },
				key = "o" .. o.value,
			}))
		end
	end
	return H.div("select" .. (if class then " " .. class else ""), {
		H.div("sel_box", {
			H.t(if current then current.label else "", "sel_text"),
			H.div("sel_btn", { H.img("buttons/menu_open") }),
		}, { actions = { click = "menu=sel:" .. id } }),
		open and H.div("sel_list", rows, { actions = { click = "noop" } }) or false,
	}, { key = "sel_" .. id })
end

-- A text box (`<input class="bginput">`) for field `name` of `state`.
function P.input(state: any, name: string, props: { [string]: any }?): Node
	return Field.node(name, state.f[name], props)
end

-- A checkbox or radio with its label; a click sends a `change` event named
-- `name`.
function P.check(kind: string, name: string, value: string, checked: boolean, label: string): Node
	return H.el("label", "check", {
		H.el("input", nil, nil, { type = kind, name = name, value = value, checked = checked }),
		H.t(label),
	}, { key = "c_" .. name .. "_" .. value })
end

-- The collapse button in a tcat or thead (collapse_tcat.gif).
function P.collapser(key: string): Node
	local shut = DB.collapsed(key)
	return H.img(if shut then "buttons/collapse_tcat_collapsed" else "buttons/collapse_tcat", "collapse=" .. key, if shut then "Expand" else "Collapse")
end

-- The status icon of a forum.
local function forum_icon(f: DB.Forum): string
	if f.kind == "link" then
		return "statusicon/forum_link"
	end
	local new = DB.forum_unread(f)
	if f.closed then
		return if new then "statusicon/forum_new_lock" else "statusicon/forum_old_lock"
	end
	return if new then "statusicon/forum_new" else "statusicon/forum_old"
end

-- Users viewing forum `f` (the online users' locations).
function P.viewing(f: DB.Forum, where: { [number]: any }): number
	local n = 0
	for _, loc in where do
		if loc.forum == f.id then
			n += 1
		end
	end
	return n
end

-- The Last Post cell of a forum or search row.
local function lastpost(p: DB.Post?, wide: boolean): Node
	if not p then
		return H.td("alt2 w-lastpost", { H.div("smallfont lp-none", { H.t("Never") }) })
	end
	local t = p.thread
	local new = DB.unread(t)
	local title = Fmt.clip(t.title, if wide then 25 else 20)
	return H.td("alt2 w-lastpost", {
		H.div("smallfont lp", {
			H.div("lp-title", {
				t.icon and H.img("icons/" .. t.icon) or false,
				t.icon and H.t(" ") or false,
				H.wrap(if new then H.go("newpost", t.id) else H.go("thread", t.id), { H.b(title) }, nil, {
					title = if new then "Go to first unread post in thread '" .. t.title .. "'" else t.title,
				}),
			}),
			H.div(nil, { H.t("by "), P.user(p.user) }),
			H.div("lp-when", H.cat(P.when(p.time), {
				H.t(" "),
				H.img("buttons/lastpost", H.go("post", p.id), "Go to last post"),
			})),
		}),
	})
end

-- One forum row on the forum home or a forum's sub-forum list
-- (forumhome_forumbit_level2_post).
function P.forumbit(f: DB.Forum, where: { [number]: any }, wide: boolean): Node
	local subs = {}
	for i, c in f.children do
		if i > 1 then
			table.insert(subs, H.t(", "))
		end
		table.insert(subs, H.img(if DB.forum_unread(c) then "statusicon/subforum_new" else "statusicon/subforum_old"))
		table.insert(subs, H.t(" "))
		table.insert(subs, H.a(c.title, H.go("forum", c.id)))
	end
	local viewing = P.viewing(f, where)
	local title_act = if f.kind == "link" then "open=" .. (f.link or "") else H.go("forum", f.id)
	local info = H.td("alt1Active w-forum", {
		H.div("fb", {
			H.div("fb-icon", { H.img(forum_icon(f), title_act) }),
			H.div("fb-text", {
				H.div(nil, {
					H.wrap(title_act, { H.b(f.title) }),
					viewing > 0 and H.t(" (" .. viewing .. " Viewing)", "smallfont viewing") or false,
				}),
				f.desc and H.div("smallfont", { H.t(f.desc) }) or false,
				#subs > 0 and H.div("smallfont subforums", H.cat({ H.b("Sub-Forums"), H.t(": ") }, subs)) or false,
			}),
		}),
	}, nil)
	if f.kind == "link" then
		return H.row({
			info,
			H.td("alt2 w-lastpost", { H.div("smallfont lp-none", { H.t("") }) }),
			wide and H.td("alt1 w-count", { H.t("-") }) or false,
			wide and H.td("alt2 w-count", { H.t("-") }) or false,
		}, nil, { key = "f" .. f.id })
	end
	return H.row({
		info,
		lastpost(f.last, wide),
		wide and H.td("alt1 w-count", { H.t(Fmt.num(f.nthreads)) }) or false,
		wide and H.td("alt2 w-count", { H.t(Fmt.num(f.nposts)) }) or false,
	}, nil, { key = "f" .. f.id })
end

-- The header row of a forum list.
function P.forumhead(wide: boolean): Node
	return H.row({
		H.td("thead w-icon", { H.t(" ") }),
		H.td("thead w-forum2", { H.t("Forum") }),
		H.td("thead w-lastpost c", { H.t("Last Post") }),
		wide and H.td("thead w-count c", { H.t("Threads") }) or false,
		wide and H.td("thead w-count c", { H.t("Posts") }) or false,
	}, "headrow")
end

-- The status icon of thread `t`.
local function thread_icon(t: DB.Thread): string
	local new = DB.unread(t)
	if t.locked then
		return if new then "statusicon/thread_lock_new" else "statusicon/thread_lock"
	elseif DB.hot(t) then
		return if new then "statusicon/thread_hot_new" else "statusicon/thread_hot"
	end
	return if new then "statusicon/thread_new" else "statusicon/thread"
end

-- Links to the pages of a thread longer than a page: "(1 2 3 ... Last Page)".
local function multipage(t: DB.Thread, per: number): Node | false
	local pages = math.ceil(#t.posts / per)
	if pages <= 1 then
		return false
	end
	local links: { any } = { H.t(" ("), H.img("misc/multipage") }
	for n = 1, math.min(pages, 4) do
		table.insert(links, H.t(" "))
		table.insert(links, H.a(tostring(n), H.go("thread", t.id, n)))
	end
	if pages > 4 then
		table.insert(links, H.t(" ... "))
		table.insert(links, H.a("Last Page", H.go("thread", t.id, pages)))
	end
	table.insert(links, H.t(")"))
	return H.el("span", "smallfont multipage", links)
end

-- The tooltip of a thread title: the start of its first post.
local function preview(t: DB.Thread, strip: (string) -> string): string
	return Fmt.clip(strip(t.first.body), 250)
end

export type ThreadbitOpts = {
	wide: boolean,
	forum_col: boolean?,
	hl: string?,
	strip: (string) -> string,
	per: number,
}

-- One thread row (threadbit): status icon, post icon, title and starter,
-- last post, replies, views, and the forum in search results.
function P.threadbit(t: DB.Thread, o: ThreadbitOpts): Node
	local new = DB.unread(t)
	local f = DB.forum[t.forum]
	local rating = t.rating and t.votes >= 3 and math.clamp(math.floor(t.rating + 0.5), 1, 5)
	local go_thread = if o.hl then H.go("thread", t.id, 1, o.hl) else H.go("thread", t.id)
	local title_cell = H.td("alt1 w-thread", {
		H.div("tb-title", {
			rating and H.el("span", "tb-right", {
				H.img("rating/rating_" .. rating, nil, string.format("Thread Rating: %d votes, %.2f average.", t.votes, t.rating or 0)),
			}) or false,
			new and H.img("buttons/firstnew", H.go("newpost", t.id), "Go to first new post") or false,
			new and H.t(" ") or false,
			t.sticky and H.t("Sticky: ") or false,
			H.a(t.title, go_thread, if new then "tb-link unread" else "tb-link"),
			multipage(t, o.per),
		}),
		H.div("smallfont tb-starter", { P.user(t.starter, "starter") }),
	}, { title = preview(t, o.strip) })
	return H.row({
		H.td("alt1 w-status c", { H.img(thread_icon(t)) }),
		H.td("alt2 w-picon c", { if t.icon then H.img("icons/" .. t.icon) else H.t(" ") }),
		title_cell,
		H.td("alt2 w-tlast", {
			H.div("smallfont r", {
				H.div(nil, P.when(t.last.time)),
				H.div(nil, {
					H.t("by "),
					P.user(t.last.user),
					H.t(" "),
					H.img("buttons/lastpost", H.go("post", t.last.id), "Go to last post"),
				}),
			}),
		}, { title = string.format("Replies: %s, Views: %s", Fmt.num(#t.posts - 1), Fmt.num(t.views)) }),
		o.wide and H.td("alt1 w-count c", { H.a(Fmt.num(#t.posts - 1), H.go("thread", t.id)) }) or false,
		o.wide and H.td("alt2 w-count c", { H.t(Fmt.num(t.views)) }) or false,
		o.forum_col and f and H.td("alt1 w-tforum", { H.a(f.title, H.go("forum", f.id), "smallfont") }) or false,
	}, nil, { key = "t" .. t.id })
end

-- The thread icon legend (forumdisplay's bottom right).
function P.thread_legend(): Node
	local function item(icon: string, text: string): Node
		return H.div("legend_item", { H.img(icon), H.t(" " .. text) })
	end
	return H.div("smallfont iconlegend", {
		item("statusicon/thread_new", "New posts"),
		item("statusicon/thread_hot_new", "Hot thread with new posts"),
		item("statusicon/thread", "No new posts"),
		item("statusicon/thread_hot", "Hot thread with no new posts"),
		item("statusicon/thread_lock", "Thread is closed"),
		item("statusicon/thread_lock_new", "Closed thread with new posts"),
	})
end

-- The Posting Rules box.
function P.rules(closed: boolean): Node
	local function line(may: boolean, what: string): Node
		return H.div(nil, { H.t("You "), H.b(if may then "may" else "may not"), H.t(" " .. what) })
	end
	return H.tborder({
		H.row({ H.td("thead", { H.t("Posting Rules") }) }),
		H.row({
			H.td("alt1", {
				H.div("smallfont rules", {
					line(not closed, "post new threads"),
					line(not closed, "post replies"),
					line(false, "post attachments"),
					line(true, "edit your posts"),
					H.el("hr", nil),
					H.div(nil, { H.a("BB code", H.go("faq", 3)), H.t(" is "), H.b("On") }),
					H.div(nil, { H.a("Smilies", H.go("faq", 4)), H.t(" are "), H.b("On") }),
					H.div(nil, { H.t("[IMG] code is "), H.b("Off") }),
					H.div(nil, { H.t("HTML code is "), H.b("Off") }),
					H.el("hr", nil),
					H.div(nil, { H.a("Forum Rules", H.go("thread", 1)) }),
				}),
			}),
		}),
	}, "rulesbox", { key = "rules" })
end

-- Forum options for a select: every forum, indented by depth.
function P.forum_options(first: Option?): { Option }
	local out = {}
	if first then
		table.insert(out, first)
	end
	local function walk(f: DB.Forum, depth: number)
		if f.kind ~= "link" then
			table.insert(out, { value = tostring(f.id), label = f.title, indent = depth })
		end
		for _, c in f.children do
			walk(c, depth + 1)
		end
	end
	for _, f in DB.forums do
		if not f.parent then
			walk(f, 0)
		end
	end
	return out
end

-- Forum Jump: a select of every forum that opens the one picked.
function P.forumjump(state: any, current: number?): Node
	local options = P.forum_options({ value = "home", label = "Forum Home" })
	return H.div("smallfont forumjump", {
		H.div("fj-label", { H.b("Forum Jump") }),
		H.div("fj-row", {
			P.select("jump", options, tostring(current or "home"), "jump", state.menu == "sel:jump", "w-jump"),
			H.button("Go", "jump=" .. tostring(current or "home")),
		}),
	})
end

-- The "you are here" lines users' locations show (Who's Online and
-- Viewing counts): each online member at a page, deterministically.
function P.locations(state: any): { [number]: any }
	local out = {}
	local recent = {}
	for _, t in DB.threads do
		table.insert(recent, t)
	end
	table.sort(recent, function(a, b)
		return a.last.time > b.last.time
	end)
	local i = 0
	for _, u in DB.online() do
		if u.me then
			out[u.id] = { text = state.where or "Viewing Forum Home" }
		else
			i += 1
			local t = recent[((i * 3) % math.max(#recent, 1)) + 1]
			local kind = i % 4
			if kind == 1 and t then
				out[u.id] = { text = "Viewing Thread", thread = t, forum = t.forum }
			elseif kind == 2 and t then
				out[u.id] = { text = "Replying to Thread", thread = t, forum = t.forum }
			elseif kind == 3 then
				out[u.id] = { text = "Viewing Forum Home" }
			else
				local f = DB.forum[(t and t.forum) or 3]
				out[u.id] = { text = "Viewing Forum", forum = f and f.id, forum_title = f and f.title }
			end
		end
	end
	return out
end

return P
