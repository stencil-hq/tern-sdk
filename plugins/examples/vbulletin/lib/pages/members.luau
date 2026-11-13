-- Members: the Members List (memberlist.php) with its letter filter and
-- sortable columns, View Site Leaders, and a member's profile (member.php).

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")
local BB = require("../bbcode")

local M = {}

local PER = 30
local LETTERS = "#ABCDEFGHIJKLMNOPQRSTUVWXYZ"

local function staff(u: DB.User): boolean
	return u.group == "admin" or u.group == "smod" or u.group == "mod"
end

local function sorted(state: any, letter: string): { DB.User }
	local out = {}
	for _, u in DB.users do
		local first = string.upper(string.sub(u.name, 1, 1))
		local ok = letter == "" or (letter == "staff" and staff(u))
			or (letter == "#" and not string.match(first, "%a"))
			or first == letter
		if ok then
			table.insert(out, u)
		end
	end
	local by, asc = state.msort.by, state.msort.asc
	table.sort(out, function(a, b)
		local ka: any, kb: any
		if by == "joined" then
			ka, kb = a.joined, b.joined
		elseif by == "posts" then
			ka, kb = a.posts, b.posts
		elseif by == "last" then
			ka, kb = a.last, b.last
		else
			ka, kb = string.lower(a.name), string.lower(b.name)
		end
		if ka == kb then
			return string.lower(a.name) < string.lower(b.name)
		end
		if asc then
			return ka < kb
		end
		return ka > kb
	end)
	return out
end

local function head(state: any, label: string, by: string, class: string): Node
	local on = state.msort.by == by
	return H.td("thead " .. class, {
		H.a(label, "msort=" .. by),
		on and H.t(" ") or false,
		on and H.img(if state.msort.asc then "buttons/sortasc" else "buttons/sortdesc", "msort=" .. by) or false,
	})
end

function M.list(state: any): any
	state.where = "Viewing Members List"
	local letter = tostring(state.route[2] or "")
	local users = sorted(state, letter)
	local pages = math.max(1, math.ceil(#users / PER))
	local pg = math.clamp(tonumber(state.route[3]) or 1, 1, pages)
	local cells: { any } = {}
	for i = 1, #LETTERS do
		local l = string.sub(LETTERS, i, i)
		table.insert(cells, H.td(if l == letter then "alt2 ltr" else "alt1 ltr", {
			H.a(l, H.go("members", l), "smallfont"),
		}, { key = "l" .. i }))
	end
	table.insert(cells, H.td(if letter == "" then "alt2 ltr" else "alt1 ltr", { H.a("All", H.go("members"), "smallfont") }, { key = "all" }))
	local rows: { any } = {
		H.row({
			H.td("tcat grow", {
				H.el("span", "tcat-right smallfont", {
					H.t(string.format("Showing results %d to %d of %d", math.min((pg - 1) * PER + 1, #users), math.min(#users, pg * PER), #users)),
				}),
				H.t(if letter == "staff" then "Forum Leaders" else "Members List"),
			}),
		}, "tcatrow"),
		H.row({
			head(state, "User Name", "name", "grow"),
			head(state, "Join Date", "joined", "w-ml c"),
			head(state, "Posts", "posts", "w-ml c"),
			state.wide and head(state, "Last Visit", "last", "w-mlast c") or false,
		}, "headrow"),
	}
	for i = (pg - 1) * PER + 1, math.min(#users, pg * PER) do
		local u = users[i]
		table.insert(rows, H.row({
			H.td("alt1 grow", {
				H.div(nil, { P.user(u) }),
				H.div("smallfont shade", { H.t(u.title) }),
			}),
			H.td("alt2 w-ml c", { H.t(Fmt.date(u.joined), "smallfont") }),
			H.td("alt1 w-ml c", { H.a(Fmt.num(u.posts), "find=posts/" .. u.id) }),
			state.wide and H.td("alt2 w-mlast c", {
				H.el("span", "smallfont", { H.t(Fmt.day(u.last) .. " "), H.t(Fmt.time(u.last), "time") }),
			}) or false,
		}, nil, { key = "u" .. u.id }))
	end
	return {
		crumbs = {},
		title = if letter == "staff" then "View Site Leaders" else "Members List",
		body = {
			H.div("controls", {
				H.tborder({ H.row(cells) }, "letters"),
				P.pagenav(pg, pages, function(n)
					return H.go("members", letter, n)
				end, #users, PER),
			}, { key = "ctl" }),
			H.tborder(rows, "memberlist", { key = "members" }),
		},
	}
end

local function block(title: string, c: { any }, key: string): Node
	return H.tborder({
		H.row({ H.td("thead", { H.t(title) }) }),
		H.row({ H.td("alt1 block_row", c) }),
	}, "content_block", { key = key })
end

local function item(label: string, value: { any }): Node
	return H.div("li", H.cat({ H.t(label .. ": ", "shade") }, value))
end

function M.profile(state: any): any
	local u = DB.user[state.route[2] or 0]
	if not u then
		return {
			crumbs = {},
			title = "Message",
			body = { require("./misc").message("This user has not registered and therefore does not have a profile to view.") },
		}
	end
	state.where = "Viewing Profile " .. u.name
	local days = math.max(1, (os.time() - u.joined) / 86400)
	local mine = DB.search_posts({ words = {}, users = { u }, starter = false, titles = false })
	local recent = mine[1]
	local main: { any } = {
		H.div("userinfo", {
			u.avatar and H.div("bigavatar", { H.img("avatars/" .. u.avatar) }) or false,
			H.div("username_box", {
				H.div("h1", {
					H.t(u.name, "username g-" .. u.group),
					H.t(" "),
					H.img(if u.online then "statusicon/user_online" else "statusicon/user_offline", nil, u.name .. (if u.online then " is online now" else " is offline")),
				}),
				H.div("h2", { H.t(u.title) }),
				H.div("smallfont", H.cat({ H.t("Last Activity: ") }, P.when(u.last))),
			}),
			H.div("grow"),
			H.div("profile-links smallfont", {
				H.div(nil, { H.a("Send a private message to " .. u.name, H.go("pm")) }),
				H.div(nil, { H.a("Find all posts by " .. u.name, "find=posts/" .. u.id) }),
				H.div(nil, { H.a("Find all threads started by " .. u.name, "find=threads/" .. u.id) }),
			}),
		}, { key = "head" }),
		H.div("gap"),
		block("About Me", {
			H.div("smallfont list", {
				u.bio and item("Biography", { H.t(u.bio) }) or false,
				u.location and item("Location", { H.t(u.location) }) or false,
				u.interests and item("Interests", { H.t(u.interests) }) or false,
				u.occupation and item("Occupation", { H.t(u.occupation) }) or false,
				not (u.bio or u.location or u.interests or u.occupation) and H.div(nil, { H.t(u.name .. " hasn't written anything about themselves yet.", "shade") }) or false,
			}),
			u.sig and H.div("smallfont sigbox", H.cat({ H.div("shade", { H.t("Signature") }) }, { H.div("post_message", BB.render(u.sig)) })) or false,
		}, "about"),
		H.div("gap"),
		block("Statistics", {
			H.div("cols2", {
				H.div("fieldset", {
					H.div("legend", { H.t("Total Posts") }),
					H.div("smallfont list", {
						item("Total Posts", { H.t(Fmt.num(u.posts)) }),
						item("Posts Per Day", { H.t(string.format("%.2f", u.posts / days)) }),
						H.div("li", { H.a("Find all posts by " .. u.name, "find=posts/" .. u.id) }),
						H.div("li", { H.a("Find all threads started by " .. u.name, "find=threads/" .. u.id) }),
					}),
				}, { key = "posts" }),
				H.div("fieldset", {
					H.div("legend", { H.t("General Information") }),
					H.div("smallfont list", {
						recent and item("Last Post", { H.a(recent.thread.title, H.go("post", recent.id)), H.t(" " .. Fmt.stamp(recent.time), "shade") }) or false,
						item("Last Activity", P.when(u.last)),
						item("Join Date", { H.t(Fmt.date(u.joined)) }),
						item("Referrals", { H.t("0") }),
					}),
				}, { key = "info" }),
			}),
		}, "stats"),
	}
	return {
		crumbs = { { "Members List", H.go("members") } },
		title = "View Profile: " .. u.name,
		body = { H.div("profile", main, { key = "profile" }) },
	}
end

return M
