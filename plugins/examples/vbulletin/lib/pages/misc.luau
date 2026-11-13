-- The smaller pages: vBulletin messages and redirects, Who's Online, the
-- FAQ, the Calendar, the User CP and Private Messages.

local H = require("../h")
local DB = require("../db")
local Fmt = require("../fmt")
local P = require("../parts")
local BB = require("../bbcode")

local M = {}

-- The standard error box ("vBulletin Message").
function M.message(text: string): Node
	return H.tborder({
		H.row({ H.td("tcat", { H.t("vBulletin Message") }) }),
		H.row({
			H.td("panelsurround c", {
				H.div("panel", { H.div("panel-inner", { H.t(text) }) }),
				H.div("buttons", { H.button("Go Back", "back") }),
			}),
		}),
	}, "msgbox", { key = "message" })
end

-- The redirect page shown after posting or logging out; the host moves on
-- by itself after a moment.
function M.redirect(state: any): any
	local r = state.redirect or { text = "", go = H.go("home") }
	return {
		crumbs = {},
		title = "Redirecting...",
		bare = true,
		body = {
			H.div("redirect", {
				H.tborder({
					H.row({ H.td("tcat", { H.t(DB.title) }) }),
					H.row({
						H.td("panelsurround c", {
							H.div("panel", {
								H.div("panel-inner c", {
									H.b(r.text),
									H.div("smallfont redirect-click", {
										H.t("(", "smallfont"),
										H.a("Click here if you do not want to wait any longer", r.go),
										H.t(")"),
									}),
								}),
							}),
						}),
					}),
				}),
			}, { key = "redirect" }),
		},
	}
end

function M.online(state: any): any
	state.where = "Viewing Who's Online"
	local locs = P.locations(state)
	local online = DB.online()
	local rows: { any } = {
		H.row({
			H.td("tcat grow", {
				H.el("span", "tcat-right smallfont", {
					H.t(string.format("%d Members and %d Guests", #online, DB.guests())),
				}),
				H.t("Who's Online"),
			}),
		}, "tcatrow"),
		H.row({
			H.td("thead w-who", { H.t("User Name") }),
			H.td("thead w-whotime c", { H.t("Last Activity") }),
			H.td("thead grow", { H.t("Location") }),
		}, "headrow"),
	}
	for i, u in online do
		local loc = locs[u.id] or { text = "Viewing Forum Home" }
		local where: { any } = { H.t(loc.text) }
		if loc.thread then
			where = { H.t(loc.text .. " "), H.a(loc.thread.title, H.go("thread", loc.thread.id)) }
		elseif loc.forum and loc.forum_title then
			where = { H.t(loc.text .. " "), H.a(loc.forum_title, H.go("forum", loc.forum)) }
		end
		local a, b = if i % 2 == 1 then "alt1" else "alt2", if i % 2 == 1 then "alt2" else "alt1"
		table.insert(rows, H.row({
			H.td(a .. " w-who", { P.user(u) }),
			H.td(b .. " w-whotime c", { H.t(Fmt.time(u.last), "smallfont time") }),
			H.td(a .. " grow", { H.div("smallfont", where) }),
		}, nil, { key = "u" .. u.id }))
	end
	for g = 1, DB.guests() do
		local a = if (#online + g) % 2 == 1 then "alt1" else "alt2"
		table.insert(rows, H.row({
			H.td(a .. " w-who", { H.t("Guest", "shade") }),
			H.td(a .. " w-whotime c", { H.t(Fmt.time(os.time() - g * 97), "smallfont time") }),
			H.td(a .. " grow", { H.div("smallfont", { H.t(if g % 3 == 0 then "Viewing Forum Home" elseif g % 3 == 1 then "Viewing Thread" else "Searching Forums") }) }),
		}, nil, { key = "g" .. g }))
	end
	return {
		crumbs = {},
		title = "Who's Online",
		body = { H.tborder(rows, "online", { key = "online" }) },
	}
end

local FAQ = {
	{ "User Maintenance", "Why should I register?", "You need to register to post. Registration is free and lets you use features such as subscriptions, private messaging and the user control panel." },
	{ "User Maintenance", "What is the User Control Panel?", "Your User CP lists the threads you subscribed to. Use Thread Tools › Subscribe to this Thread on any thread to add it." },
	{ "Reading and Posting Messages", "What is BB code?", "BB code is a set of tags based on HTML that lets you format messages: [b]bold[/b], [i]italic[/i], [u]underline[/u], [color=red]color[/color], [size=4]size[/size], [url]links[/url], [quote]quotes[/quote], [code]code[/code] and [list][*]lists[/list]." },
	{ "Reading and Posting Messages", "What are smilies?", "Smilies are small graphical images that convey an emotion or feeling. Type :) :D ;) :p :( :o :cool: :confused: :mad: :rolleyes: or :redface: and they become pictures. Click one beside the editor to insert it." },
	{ "Reading and Posting Messages", "How do I search?", "Use Search in the navigation bar for a quick search, or Advanced Search to search titles only, find posts or threads by a user, or limit the search to one forum." },
	{ "General Forum Usage", "How do I find new posts?", "New Posts lists every thread with posts since your last visit. Quick Links › Today's Posts lists the threads with posts in the last 24 hours." },
	{ "General Forum Usage", "Can I use the keyboard?", "Yes. Tab moves between fields, Enter submits a search, ⌘Enter posts a reply, Esc leaves a field, j and k move between posts, and Backspace goes back a page." },
}

function M.faq(state: any): any
	state.where = "Viewing FAQ"
	local open = state.route[2]
	local rows: { any } = { H.row({ H.td("tcat", { H.t("vBulletin FAQ") }) }) }
	local group = nil
	for i, q in FAQ do
		if q[1] ~= group then
			group = q[1]
			table.insert(rows, H.row({ H.td("thead", { H.t(group) }) }, nil, { key = "g" .. i }))
		end
		local shown = open == i
		table.insert(rows, H.row({
			H.td("alt1 faq", {
				H.div(nil, { H.a(q[2], if shown then H.go("faq") else H.go("faq", i), "b") }),
				shown and H.div("post_message faq-answer", { H.t(q[3]) }) or false,
			}),
		}, nil, { key = "q" .. i }))
	end
	return {
		crumbs = {},
		title = "FAQ",
		body = { H.tborder(rows, "faqlist", { key = "faq" }) },
	}
end

local DAYS = { "Sunday", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday" }

function M.calendar(state: any): any
	state.where = "Viewing Calendar"
	local now = os.date("*t")
	local y = tonumber(state.route[2]) or now.year
	local m = tonumber(state.route[3]) or now.month
	local first = os.time({ year = y, month = m, day = 1, hour = 12 } :: any)
	local start = os.date("*t", first).wday
	local days = os.date("*t", os.time({ year = y, month = m + 1, day = 0, hour = 12 } :: any)).day
	local py, pm = if m == 1 then y - 1 else y, if m == 1 then 12 else m - 1
	local ny, nm = if m == 12 then y + 1 else y, if m == 12 then 1 else m + 1
	local cells: { any } = {}
	for _, d in DAYS do
		table.insert(cells, H.div("thead c calhead", { H.t(d) }, { key = d }))
	end
	for i = 1, start - 1 do
		table.insert(cells, H.div("alt2 calday blank", nil, { key = "b" .. i }))
	end
	for d = 1, days do
		local today = y == now.year and m == now.month and d == now.day
		local bdays = DB.birthdays(m, d)
		local items: { any } = { H.div("calnum", { H.t(tostring(d)) }) }
		for _, u in bdays do
			table.insert(items, H.div("smallfont calevent", { H.img("misc/birthday"), H.t(" "), P.user(u), H.t("'s Birthday") }))
		end
		table.insert(cells, H.div(if today then "alt1 calday today" else "alt1 calday", items, { key = "d" .. d }))
	end
	local trailing = (7 - ((start - 1 + days) % 7)) % 7
	for i = 1, trailing do
		table.insert(cells, H.div("alt2 calday blank", nil, { key = "a" .. i }))
	end
	local title = os.date("%B %Y", first)
	return {
		crumbs = {},
		title = "Default Calendar",
		body = {
			H.tborder({
				H.row({
					H.td("tcat grow c calnav", {
						H.a("«", H.go("calendar", py, pm), "b"),
						H.t("  " .. title .. "  "),
						H.a("»", H.go("calendar", ny, nm), "b"),
					}),
				}),
				H.div("calgrid", cells),
			}, "calendar", { key = "calendar" }),
		},
	}
end

function M.usercp(state: any): any
	state.where = "Viewing User Control Panel"
	local subs = DB.subscriptions()
	local rows: { any } = {
		H.row({ H.td("tcat", { H.t("Subscribed Threads with New Posts") }) }),
	}
	if #subs == 0 then
		table.insert(rows, H.row({
			H.td("alt1", {
				H.div("smallfont", { H.t("There are no threads in your subscriptions. Use Thread Tools › Subscribe to this Thread to add one.") }),
			}),
		}))
	else
		table.insert(rows, H.row({
			H.td("thead w-status2", { H.t(" ") }),
			H.td("thead w-thread", { H.t("Thread / Thread Starter") }),
			H.td("thead w-tlast c", { H.t("Last Post") }),
			state.wide and H.td("thead w-count c", { H.t("Replies") }) or false,
			state.wide and H.td("thead w-count c", { H.t("Views") }) or false,
		}, "headrow"))
		local opts = { wide = state.wide, strip = BB.strip, per = 10 }
		for _, t in subs do
			table.insert(rows, P.threadbit(t, opts))
		end
	end
	return {
		crumbs = {},
		title = "User Control Panel",
		body = {
			H.div("usercp", {
				H.div("ucp-nav", {
					H.tborder({
						H.row({ H.td("tcat", { H.t("My Account") }) }),
						H.row({ H.td("alt2 ucp-link", { H.a("Your Profile", H.go("member", DB.me.id), "smallfont") }) }),
						H.row({ H.td("thead", { H.t("Networking") }) }),
						H.row({ H.td("alt2 ucp-link", { H.a("Who's Online", H.go("online"), "smallfont") }) }),
						H.row({ H.td("thead", { H.t("Subscribed Threads") }) }),
						H.row({ H.td("alt2 ucp-link", { H.a("List Subscriptions", H.go("usercp"), "smallfont") }) }),
						H.row({ H.td("thead", { H.t("Private Messages") }) }),
						H.row({ H.td("alt2 ucp-link", { H.a("List Messages", H.go("pm"), "smallfont") }) }),
					}),
				}),
				H.div("ucp-main", { H.tborder(rows, "threadslist") }),
			}, { key = "usercp" }),
		},
	}
end

function M.pm(state: any): any
	state.where = "Viewing Private Messages"
	return {
		crumbs = { { "User Control Panel", H.go("usercp") } },
		title = "Private Messages",
		body = {
			H.tborder({
				H.row({
					H.td("tcat grow", {
						H.el("span", "tcat-right smallfont", { H.t("0 messages. 500 remaining.") }),
						H.t("Private Messages in Folder: Inbox"),
					}),
				}, "tcatrow"),
				H.row({ H.td("thead", { H.t("Messages") }) }),
				H.row({
					H.td("alt1", { H.div("smallfont", { H.t("There are no messages in this folder for the specified time period.") }) }),
				}),
			}, nil, { key = "pm" }),
		},
	}
end

return M
