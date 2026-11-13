-- The frame around every page: the logo header, the breadcrumb navbar with
-- the welcome box, the navigation buttons bar with its Search and Quick
-- Links menus, and the footer (vBulletin 3.8's header, navbar and footer
-- templates).

local H = require("./h")
local DB = require("./db")
local Fmt = require("./fmt")
local P = require("./parts")

local C = {}

export type Page = {
	-- Breadcrumbs between the board and the page: { title, action }.
	crumbs: { { any } },
	-- The page's own title, bold on the navbar's second line; nil on the
	-- forum home.
	title: string?,
	body: { any },
	-- Drawn alone on the body, like vBulletin's redirect pages.
	bare: boolean?,
}

local function header(): Node
	return H.tborder({
		H.row({
			H.td("alt1 logo-cell", { H.img("misc/logo", H.go("home"), DB.title) }),
			H.td("header_right", { H.t(" ") }),
		}),
	}, "header", { key = "header" })
end

local function navbar(state: any, page: Page): Node
	local crumbs: { any } = {}
	if page.title then
		table.insert(crumbs, H.a(DB.title, H.go("home"), "navbar", { key = "home" }))
		for _, c in page.crumbs do
			table.insert(crumbs, H.t(" > ", "navbar"))
			table.insert(crumbs, H.a(c[1], c[2], "navbar"))
		end
	else
		table.insert(crumbs, H.el("span", "navbar", { H.b(DB.title) }))
	end
	local me = DB.me
	local last = DB.lastvisit()
	return H.tborder({
		H.row({
			H.td("alt1 navcell", {
				H.div("nav1", {
					H.img("misc/navbits_start", "back", "Go Back"),
					H.t(" "),
					H.el("span", "crumbs", crumbs),
				}),
				page.title and H.div("navbar nav2", {
					H.img("misc/navbits_finallink", "reload", "Reload this Page"),
					H.t(" "),
					H.b(page.title),
				}) or false,
			}),
			H.td("alt2 welcome", {
				H.div("smallfont", {
					H.div(nil, { H.b("Welcome, "), P.user(me, "b"), H.b(".") }),
					H.div(nil, H.cat({ H.t("You last visited: " .. Fmt.day(last) .. " at ") }, { H.t(Fmt.time(last), "time") })),
					H.div(nil, { H.a("Private Messages", H.go("pm")), H.t(": Unread 0, Total 0.") }),
				}),
			}),
		}),
	}, "navbox", { key = "navbar" })
end

local function buttons(state: any): Node
	local menu = state.menu
	local function item(label: string, act: string, key: string): Node
		return H.td("vbmenu_control", { H.a(label, act) }, { key = key })
	end
	return H.div("navbuttons", {
		H.row({
			item("User CP", H.go("usercp"), "usercp"),
			item("FAQ", H.go("faq"), "faq"),
			item("Members List", H.go("members"), "members"),
			item("Calendar", H.go("calendar"), "calendar"),
			item("New Posts", H.go("getnew"), "getnew"),
			P.menu("navbar_search", "Search", menu == "navbar_search", {
				P.menu_head("Search Forums"),
				H.div("vbmenu_option form", {
					H.div("qs", {
						P.input(state, "qs", { placeholder = "", aria = "Search Forums" }),
						H.button("Go", "submit=qs"),
					}),
					H.div("qs-show", {
						P.check("radio", "qsposts", "0", state.form.qsposts ~= "1", "Show Threads"),
						P.check("radio", "qsposts", "1", state.form.qsposts == "1", "Show Posts"),
					}),
				}, { key = "qsform" }),
				P.menu_option("Advanced Search", H.go("search")),
			}),
			P.menu("usercptools", "Quick Links", menu == "usercptools", {
				P.menu_option("Today's Posts", H.go("getdaily")),
				P.menu_option("Mark Forums Read", "markread"),
				P.menu_option("Open Buddy List", H.go("online")),
				P.menu_head("User Control Panel"),
				P.menu_option("Subscribed Threads", H.go("usercp")),
				P.menu_option("Your Profile", H.go("member", DB.me.id)),
				P.menu_head("Miscellaneous"),
				P.menu_option("Who's Online", H.go("online")),
				P.menu_option("Members List", H.go("members")),
			}, "right"),
			item("Log Out", "logout", "logout"),
		}),
	}, { key = "navbuttons" })
end

local function footer(state: any): { Node }
	return {
		H.div("page tfoot-wrap", {
			H.row({
				H.td("tfoot", {
					P.select("style", {
						{ value = "1", label = "-- vBulletin 3" },
						{ value = "2", label = "---- Tern Community Classic" },
					}, "1", "style", state.menu == "sel:style", "w-style"),
				}),
				H.td("tfoot r grow", {
					H.div("smallfont", {
						H.a("Contact Us", H.go("member", 1), "b"),
						H.t(" - ", "b"),
						H.a(DB.title, H.go("home"), "b"),
						H.t(" - ", "b"),
						H.a("Archive", H.go("forum", 22), "b"),
						H.t(" - ", "b"),
						H.a("Top", "top", "b"),
					}),
				}),
			}),
		}, { key = "tfoot" }),
		H.div("smallfont copyright", {
			H.div(nil, { H.t("Powered by vBulletin® Version 3.8.11") }),
			H.div(nil, { H.t("Copyright ©2000 - " .. os.date("%Y") .. ", vBulletin Solutions, Inc.") }),
		}, { key = "copyright" }),
	}
end

-- The whole page: `page` inside the board's frame.
function C.frame(state: any, page: Page): Node
	if page.bare then
		return H.div("vb3 bare", page.body, { key = "vb" })
	end
	local inner = H.cat({
		navbar(state, page),
		buttons(state),
		H.div("gap"),
	}, page.body)
	table.insert(inner, H.div("smallfont timeline", {
		H.t("All times are GMT" .. C.offset() .. ". The time now is "),
		H.t(Fmt.time(os.time()), "time"),
		H.t("."),
	}, { key = "timeline" }))
	local outer = H.cat({
		header(),
		H.div("page", inner, { key = "page" }),
	}, footer(state))
	return H.div("vb3" .. (if state.wide then " wide" else " narrow"), outer, {
		key = "vb",
		actions = if state.menu then { click = "menu=" } else nil,
	})
end

-- The local zone as vBulletin writes it after GMT: " +1", " -5", "".
function C.offset(): string
	local now = os.time()
	local utc = os.time(os.date("!*t", now) :: any)
	local hours = math.floor((now - utc) / 3600 + 0.5)
	if hours == 0 then
		return ""
	end
	return string.format(" %+d", hours)
end

return C
