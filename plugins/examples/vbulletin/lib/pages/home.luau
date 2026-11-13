-- The forum home (FORUMHOME): every category with its forums, What's
-- Going On? with the active users, birthdays and statistics, and the icon
-- legend.

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")

local function forums(state: any, where: { [number]: any }): Node
	local rows: { any } = { P.forumhead(state.wide) }
	for _, cat in DB.forums do
		if not cat.parent then
			local key = "cat" .. cat.id
			table.insert(rows, H.row({
				H.td("tcat", {
					H.el("span", "tcat-right", { P.collapser(key) }),
					H.a(cat.title, H.go("forum", cat.id)),
					cat.desc and H.div("smallfont", { H.t(cat.desc) }) or false,
				}),
			}, nil, { key = key }))
			if not DB.collapsed(key) then
				for _, f in cat.children do
					table.insert(rows, P.forumbit(f, where, state.wide))
				end
			end
		end
	end
	return H.tborder(rows, "forumlist", { key = "forums" })
end

local function whats_going_on(state: any): Node
	local online = DB.online()
	local guests = DB.guests()
	local names = {}
	for _, u in online do
		table.insert(names, P.user(u))
	end
	local threads, posts, members, newest = DB.totals()
	local now = os.date("*t")
	local bdays = DB.birthdays(now.month, now.day)
	local bday_names = {}
	for _, u in bdays do
		table.insert(bday_names, P.user(u))
	end
	local shut = DB.collapsed("wgo")
	local most_at = os.time() - 412 * 86400
	local rows: { any } = {
		H.row({
			H.td("tcat", {
				H.el("span", "tcat-right", { P.collapser("wgo") }),
				H.t("What's Going On?"),
			}),
		}),
	}
	if not shut then
		table.insert(rows, H.row({
			H.td("thead", {
				H.a("Currently Active Users", H.go("online")),
				H.t(string.format(": %d (%d members and %d guests)", #online + guests, #online, guests)),
			}),
		}))
		table.insert(rows, H.row({
			H.td("alt2 w-wgo c", { H.img("misc/whos_online", H.go("online"), "View Who's Online") }),
			H.td("alt1 grow", {
				H.div("smallfont", {
					H.div("nowrap", {
						H.t(string.format("Most users ever online was %d, %s at ", 87, Fmt.date(most_at))),
						H.t(Fmt.time(most_at), "time"),
						H.t("."),
					}),
					H.div("names", H.join(names, ", ")),
				}),
			}),
		}))
		if #bdays > 0 then
			table.insert(rows, H.row({ H.td("thead", { H.a("Today's Birthdays", H.go("calendar")) }) }))
			table.insert(rows, H.row({
				H.td("alt2 w-wgo c", { H.img("misc/birthday") }),
				H.td("alt1 grow", { H.div("smallfont", H.join(bday_names, ", ")) }),
			}))
		end
		table.insert(rows, H.row({ H.td("thead", { H.t(DB.title .. " Statistics") }) }))
		table.insert(rows, H.row({
			H.td("alt2 w-wgo c", { H.img("misc/stats", nil, DB.title .. " Statistics") }),
			H.td("alt1 grow", {
				H.div("smallfont", {
					H.div(nil, {
						H.t(string.format("Threads: %s, Posts: %s, Members: %s", Fmt.num(threads), Fmt.num(posts), Fmt.num(members))),
					}),
					H.div(nil, { H.t("Welcome to our newest member, "), P.user(newest) }),
				}),
			}),
		}))
	end
	return H.tborder(rows, "wgo", { key = "wgo" })
end

local function legend(): Node
	return H.div("legendbar", {
		H.div("smallfont", {
			H.img("statusicon/forum_new"),
			H.t(" New posts   "),
			H.img("statusicon/forum_old"),
			H.t(" No new posts   "),
			H.img("statusicon/forum_old_lock"),
			H.t(" A closed forum"),
		}),
		H.div("smallfont r", {
			H.a("Mark Forums Read", "markread"),
			H.t(" | "),
			H.a("View Site Leaders", H.go("members", "staff")),
		}),
	}, { key = "legend" })
end

return function(state: any): any
	local where = P.locations(state)
	return {
		crumbs = {},
		title = nil,
		body = {
			forums(state, where),
			H.div("gap"),
			whats_going_on(state),
			H.div("gap"),
			legend(),
		},
	}
end
