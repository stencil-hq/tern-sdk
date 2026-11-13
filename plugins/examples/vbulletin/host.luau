-- Block `vbulletin.forum`: a vBulletin 3.8 message board, drawn in its
-- default style. Pages: the forum home, forums, threads with Quick Reply,
-- Search and its results, New Posts, Today's Posts, the Members List and
-- profiles, Who's Online, the FAQ, the Calendar, the User CP, and the Post
-- New Thread and Reply pages. What you post stays on this machine (see
-- lib/db.luau).
--
-- Keys: Tab moves between fields, Enter submits a search, ⌘Enter posts,
-- Esc leaves a field or closes a menu, j/k move between posts, n/p turn
-- pages, r replies, / searches, h goes home, Backspace goes back.

local H = require("./lib/h")
local DB = require("./lib/db")
local Field = require("./lib/field")
local Chrome = require("./lib/chrome")
local E = require("./lib/editor")
local Home = require("./lib/pages/home")
local Forum = require("./lib/pages/forum")
local Thread = require("./lib/pages/thread")
local Search = require("./lib/pages/search")
local Members = require("./lib/pages/members")
local Misc = require("./lib/pages/misc")
local Post = require("./lib/pages/post")

local WIDE_COLS = 100
local REDIRECT_MS = 1800
local MIN_MESSAGE = 10

local PAGES: { [string]: (any) -> any } = {
	home = Home,
	forum = Forum,
	thread = Thread.view,
	print = Thread.print,
	search = Search.form,
	results = Search.results,
	members = Members.list,
	member = Members.profile,
	online = Misc.online,
	faq = Misc.faq,
	calendar = Misc.calendar,
	usercp = Misc.usercp,
	pm = Misc.pm,
	newthread = Post.newthread,
	reply = Post.reply,
	redirect = Misc.redirect,
}

-- Multi-line fields; the rest are one line.
local MULTILINE = { qr = true, nt_body = true, rp_body = true }

-- The form each field submits with Enter (⌘Enter in a multi-line one).
local FORM_OF = {
	qs = "qs", fs = "fs", ts = "ts", kw = "search", su = "search",
	qr = "qr", nt_title = "newthread", nt_body = "newthread", rp_title = "reply", rp_body = "reply",
}

type Route = { any }

-- "thread/12/2" → { "thread", 12, 2 }: numbers where a part is all digits.
local function parse(s: string): Route
	local out: Route = {}
	for part in string.gmatch(s .. "/", "([^/]*)/") do
		table.insert(out, if string.match(part, "^%d+$") then tonumber(part) else part)
	end
	if out[1] == "" or not out[1] then
		out[1] = "home"
	end
	return out
end

local function fields(): { [string]: Field.F }
	local f = {}
	for name in FORM_OF do
		f[name] = Field.new()
	end
	return f
end

local function default_form(): { [string]: any }
	return {
		titles = "0",
		starter = "0",
		exact = true,
		showposts = "0",
		forum = "all",
		since = "0",
		icon = "",
		qrquote = false,
		qsposts = "0",
	}
end

-- Whether pane `pane` still runs this block.
local function running(pane: number): boolean
	for _, p in tern.pane.list() do
		if p.pane == pane then
			return true
		end
	end
	return false
end

-- The fields Tab cycles through on the current page and menu.
local function page_fields(state: any): { string }
	if state.menu == "navbar_search" then
		return { "qs" }
	elseif state.menu == "forumsearch" then
		return { "fs" }
	elseif state.menu == "threadsearch" then
		return { "ts" }
	end
	local name = state.route[1]
	if name == "search" then
		return { "kw", "su" }
	elseif name == "thread" then
		return { "qr" }
	elseif name == "newthread" then
		return { "nt_title", "nt_body" }
	elseif name == "reply" then
		return { "rp_title", "rp_body" }
	end
	return {}
end

local function focus(state: any, name: string?)
	state.focus = name
	state.refocus = true
end

local navigate: (state: any, route: Route, push: boolean?) -> ()

-- Shows vBulletin's "Thank you for posting!" page, then `target`.
local function redirect(state: any, text: string, target: string)
	local r = { text = text, go = target }
	state.redirect = r
	state.route = { "redirect" }
	state.menu = nil
	focus(state, nil)
	state.reveal = "vb"
	local cx = state.cx
	tern.timer(REDIRECT_MS, function()
		if not running(cx.pane) or state.redirect ~= r then
			return
		end
		state.redirect = nil
		navigate(state, parse(string.sub(target, 4)), false)
		cx:render()
	end)
end

-- Runs a search and shows its results, or the search form with vBulletin's
-- error when nothing matched.
local function search(state: any, q: DB.Query, kind: string, title: string, skipped: { string }?)
	local t0 = os.clock()
	local items: { any } = if kind == "posts" then DB.search_posts(q) else DB.search_threads(q)
	local took = os.clock() - t0
	if #items == 0 then
		navigate(state, { "search" })
		state.errors = { "Sorry - no matches. Please try some different terms." }
		return
	end
	local note = nil
	if skipped and #skipped > 0 then
		note = "The following words are either very common, too long, or too short and were not included in your search: " .. table.concat(skipped, ", ")
	end
	state.result = {
		kind = kind,
		items = items,
		took = math.max(took, 0.01),
		title = title,
		note = note,
		hl = if #q.words > 0 then table.concat(q.words, "+") else nil,
	}
	navigate(state, { "results", 1 })
end

-- A search from the words in field `field`, within `forum` or `thread`.
local function quick_search(state: any, field: string, forum: number?, thread: number?)
	local words, skipped = DB.words(state.f[field].t)
	if #words == 0 then
		navigate(state, { "search" })
		state.errors = { "Please enter one or more search terms." }
		return
	end
	local posts = thread ~= nil or (field == "qs" and state.form.qsposts == "1")
	search(state, { words = words, starter = false, titles = false, forum = forum, thread = thread }, if posts then "posts" else "threads", "Search Results", skipped)
end

local function full_search(state: any)
	local form = state.form
	local words, skipped = DB.words(state.f.kw.t)
	local name = string.match(state.f.su.t, "^%s*(.-)%s*$") or ""
	local users = nil
	if name ~= "" then
		users = DB.find_user(name, form.exact)
		if #users == 0 then
			state.errors = { "The specified user name does not exist." }
			return
		end
	end
	if #words == 0 and not users then
		state.errors = { "Please enter one or more search terms." }
		return
	end
	local since = tonumber(form.since) or 0
	search(state, {
		words = words,
		users = users,
		starter = form.starter == "1",
		titles = form.titles == "1",
		forum = tonumber(form.forum),
		since = if since > 0 then os.time() - since * 86400 else nil,
	}, if form.showposts == "1" then "posts" else "threads", "Search Results", skipped)
end

local function too_short(body: string): boolean
	local text = string.gsub(body, "%s+", "")
	return #text < MIN_MESSAGE
end

local SHORT = "The message you have entered is too short. Please lengthen your message to at least " .. MIN_MESSAGE .. " characters."

local function submit(state: any, form: string)
	state.menu = if form == "qs" or form == "fs" or form == "ts" then nil else state.menu
	if form == "qs" then
		quick_search(state, "qs")
	elseif form == "fs" then
		quick_search(state, "fs", if state.route[1] == "forum" then state.route[2] else nil)
	elseif form == "ts" then
		quick_search(state, "ts", nil, state.route[2])
	elseif form == "search" then
		full_search(state)
	elseif form == "qr" then
		local t = DB.thread[state.route[2] or 0]
		if not t then
			return
		end
		local body = state.f.qr.t
		if too_short(body) then
			state.errors = { SHORT }
			focus(state, "qr")
			return
		end
		if state.form.qrquote and state.qr_post then
			local q = DB.post[state.qr_post]
			if q then
				body = "[QUOTE=" .. q.user.name .. ";" .. q.id .. "]" .. q.body .. "[/QUOTE]\n" .. body
			end
		end
		local p = DB.reply(t, body)
		state.f.qr = Field.new()
		state.form.qrquote = false
		state.qr_post = nil
		redirect(state, "Thank you for posting! You will now be taken to your post. If you opted to post a poll, you will now be allowed to do so.", H.go("post", p.id))
	elseif form == "reply" then
		local t = DB.thread[state.route[2] or 0]
		if not t then
			return
		end
		local body = state.f.rp_body.t
		if too_short(body) then
			state.errors = { SHORT }
			return
		end
		local title = string.match(state.f.rp_title.t, "^%s*(.-)%s*$")
		local p = DB.reply(t, body, if title ~= "" then title else nil, if state.form.icon ~= "" then state.form.icon else nil)
		state.f.rp_body, state.f.rp_title = Field.new(), Field.new()
		redirect(state, "Thank you for posting! You will now be taken to your post.", H.go("post", p.id))
	elseif form == "newthread" then
		local f = DB.forum[state.route[2] or 0]
		if not f then
			return
		end
		local title = string.match(state.f.nt_title.t, "^%s*(.-)%s*$") or ""
		local errors = {}
		if title == "" then
			table.insert(errors, "The title you have entered is too short. Please enter a longer title.")
		end
		if too_short(state.f.nt_body.t) then
			table.insert(errors, SHORT)
		end
		if #errors > 0 then
			state.errors = errors
			state.reveal = "vb"
			return
		end
		local t = DB.new_thread(f, title, state.f.nt_body.t, if state.form.icon ~= "" then state.form.icon else nil)
		state.f.nt_title, state.f.nt_body = Field.new(), Field.new()
		redirect(state, "Thank you for posting! You will now be taken to your new thread.", H.go("thread", t.id))
	end
end

navigate = function(state: any, route: Route, push: boolean?)
	local name = route[1]
	-- Routes that resolve to another page.
	if name == "post" then
		local p = DB.post[route[2] or 0]
		if not p then
			return navigate(state, { "thread", 0 }, push)
		end
		navigate(state, { "thread", p.thread.id, Thread.page_of(p), route[3] }, push)
		state.reveal = "p" .. p.id
		return
	elseif name == "newpost" or name == "lastpost" then
		local t = DB.thread[route[2] or 0]
		if not t then
			return navigate(state, { "thread", 0 }, push)
		end
		local p = if name == "newpost" then DB.first_unread(t) or t.last else t.last
		return navigate(state, { "post", p.id }, push)
	elseif name == "getnew" then
		return search(state, { words = {}, starter = false, titles = false, unread = true }, "threads", "New Posts")
	elseif name == "getdaily" then
		return search(state, { words = {}, starter = false, titles = false, since = os.time() - 86400 }, "threads", "Today's Posts")
	end
	if push ~= false and state.route[1] ~= "redirect" then
		table.insert(state.hist, state.route)
		if #state.hist > 50 then
			table.remove(state.hist, 1)
		end
	end
	state.route = route
	state.menu = nil
	state.errors = nil
	state.preview = false
	state.reveal = "vb"
	state.cur = nil
	focus(state, nil)
	if name == "thread" then
		local t = DB.thread[route[2] or 0]
		if t then
			local pg = math.clamp(tonumber(route[3]) or 1, 1, math.max(1, math.ceil(#t.posts / Thread.PER)))
			local last = t.posts[math.min(#t.posts, pg * Thread.PER)]
			DB.visit(t, last.time)
		end
	elseif name == "search" then
		focus(state, "kw")
	elseif name == "newthread" then
		state.form.icon = ""
		focus(state, "nt_title")
	elseif name == "reply" then
		state.form.icon = ""
		local q = DB.post[route[3] or 0]
		if q then
			state.f.rp_body = Field.new("[QUOTE=" .. q.user.name .. ";" .. q.id .. "]" .. q.body .. "[/QUOTE]\n")
		end
		focus(state, "rp_body")
	end
end

local function back(state: any)
	local prev = table.remove(state.hist)
	if prev then
		navigate(state, prev, false)
	end
end

-- The id of the node keyed `key` under `node` (whose id is `id`).
local function find_id(node: Node, id: string, key: string): string?
	if node.p and node.p.key == key then
		return id
	end
	for i, child in node.c or {} do
		local k = (child.p and child.p.key) or (i - 1)
		local found = find_id(child, id .. "." .. tostring(k), key)
		if found then
			return found
		end
	end
	return nil
end

local function event(state: any, ev: UiEvent, cx: BlockCx)
	DB.sync()
	if ev.ev == "focus" then
		local name = Field.name_of(ev.id)
		if name then
			focus(state, name)
		end
		return
	elseif ev.ev == "change" then
		local form = state.form
		if ev.name == "exact" or ev.name == "qrquote" then
			form[ev.name] = ev.checked == true
		elseif ev.name == "showposts" or ev.name == "icon" or ev.name == "qsposts" then
			form[ev.name] = ev.value or ""
		end
		return
	elseif ev.ev ~= "action" then
		return
	end
	local act, value = ev.act, ev.value or ""
	if act == "noop" then
		return
	elseif act == "go" then
		navigate(state, parse(value))
	elseif act == "open" then
		cx:open(value)
	elseif act == "menu" then
		local was = state.menu
		state.menu = if value == "" or value == was then nil else value
		local inputs = { navbar_search = "qs", forumsearch = "fs", threadsearch = "ts" }
		if inputs[state.menu or ""] then
			focus(state, inputs[state.menu])
		elseif inputs[was or ""] and state.focus == inputs[was] then
			focus(state, nil)
		end
	elseif act == "collapse" then
		DB.set_collapsed(value, not DB.collapsed(value))
	elseif act == "set" then
		local key, v = string.match(value, "^([%w_]+)/?(.*)$")
		if key then
			state.form[key] = v
		end
		state.menu = nil
	elseif act == "sortby" then
		state.tsort.by = value
		state.menu = nil
	elseif act == "order" then
		state.tsort.asc = value == "asc"
		state.menu = nil
	elseif act == "sort" then
		if state.tsort.by == value then
			state.tsort.asc = not state.tsort.asc
		else
			state.tsort.by = value
			state.tsort.asc = value == "title" or value == "starter"
		end
	elseif act == "msort" then
		if state.msort.by == value then
			state.msort.asc = not state.msort.asc
		else
			state.msort.by = value
			state.msort.asc = value == "name"
		end
	elseif act == "jump" then
		state.menu = nil
		if value == "home" then
			navigate(state, { "home" })
		else
			navigate(state, { "forum", tonumber(value) or 0 })
		end
	elseif act == "style" then
		state.menu = nil
	elseif act == "submit" then
		submit(state, value)
	elseif act == "reset" then
		state.f.kw, state.f.su = Field.new(), Field.new()
		state.form = default_form()
		state.errors = nil
		focus(state, "kw")
	elseif act == "preview" then
		state.preview = true
		state.errors = nil
		if state.route[1] == "newthread" and too_short(state.f.nt_body.t) then
			state.errors = { SHORT }
		end
		state.reveal = "vb"
	elseif act == "bb" or act == "smilie" then
		local name, arg = string.match(value, "^([%w_]+)/(.*)$")
		local f = name and state.f[name]
		if f then
			if act == "bb" then
				local open, close = E.tool(tonumber(arg) or 0)
				if open and close then
					Field.wrap(f, open, close)
				end
			else
				Field.insert(f, (if f.c > 0 and not string.match(string.sub(f.t, f.c, f.c), "%s") then " " else "") .. arg .. " ")
			end
			focus(state, name)
		end
	elseif act == "qr" then
		state.qr_post = tonumber(value)
		state.form.qrquote = true
		DB.set_collapsed("qr", false)
		focus(state, "qr")
		state.reveal = "qr"
	elseif act == "advanced" then
		local text = state.f.qr.t
		navigate(state, { "reply", tonumber(value) or 0 })
		if text ~= "" then
			state.f.rp_body = Field.new(text)
		end
	elseif act == "vote" then
		local t = DB.thread[state.route[2] or 0]
		if t and not DB.my_vote(t) then
			DB.vote(t, tonumber(value) or 3)
			redirect(state, "Thank you for voting. You will now be taken back to the thread.", H.go("thread", t.id, state.route[3] or 1))
		end
		state.menu = nil
	elseif act == "subscribe" then
		local t = DB.thread[state.route[2] or 0]
		if t then
			local on = not DB.subscribed(t)
			DB.set_subscribed(t, on)
			redirect(state, if on
				then "You are now subscribed to this thread. You will now be taken back to the thread."
				else "Your subscription to this thread has been deleted.", H.go("thread", t.id, state.route[3] or 1))
		end
	elseif act == "markread" then
		DB.mark_all_read()
		redirect(state, "All forums have been marked as read. Now taking you to the forums home page.", H.go("home"))
	elseif act == "logout" then
		redirect(state, "All cookies cleared! (You logged straight back in: this board has only one member at this computer.)", H.go("home"))
	elseif act == "find" then
		local kind, id = string.match(value, "^(%a+)/(%d+)$")
		local u = DB.user[tonumber(id) or 0]
		if u then
			search(state, { words = {}, users = { u }, starter = kind == "threads", titles = false },
				if kind == "threads" then "threads" else "posts", "Search Results")
		end
	elseif act == "back" then
		back(state)
	elseif act == "reload" or act == "top" then
		state.reveal = "vb"
		state.menu = nil
	end
end

-- Keys outside the text fields: the board's own shortcuts.
local function shortcut(state: any, key: Key): boolean
	local name = key.name
	if key.ctrl or key.meta then
		return false
	end
	if name == "escape" then
		if state.menu then
			state.menu = nil
			return true
		end
		return false
	elseif name == "tab" then
		local list = page_fields(state)
		if #list == 0 then
			return false
		end
		focus(state, list[1])
		return true
	elseif name == "backspace" or (key.alt and name == "left") or name == "[" then
		back(state)
		return true
	elseif name == "/" then
		state.menu = "navbar_search"
		state.reveal = "vb"
		focus(state, "qs")
		return true
	elseif name == "h" then
		navigate(state, { "home" })
		return true
	end
	local r = state.route
	if r[1] == "thread" then
		local t = DB.thread[r[2] or 0]
		if not t then
			return false
		end
		local pg = tonumber(r[3]) or 1
		local pages = math.max(1, math.ceil(#t.posts / Thread.PER))
		local first = (pg - 1) * Thread.PER + 1
		local last = math.min(#t.posts, pg * Thread.PER)
		if name == "j" or name == "k" then
			local cur = state.cur or (if name == "j" then first - 1 else last + 1)
			cur = math.clamp(cur + (if name == "j" then 1 else -1), first, last)
			state.cur = cur
			state.reveal = "p" .. t.posts[cur].id
			return true
		elseif name == "n" and pg < pages then
			navigate(state, { "thread", t.id, pg + 1, r[4] })
			return true
		elseif name == "p" and pg > 1 then
			navigate(state, { "thread", t.id, pg - 1, r[4] })
			return true
		elseif name == "r" and not t.locked then
			DB.set_collapsed("qr", false)
			focus(state, "qr")
			state.reveal = "qr"
			return true
		end
	elseif r[1] == "forum" or r[1] == "results" or r[1] == "members" then
		local at = if r[1] == "results" then 2 else 3
		local pg = tonumber(r[at]) or 1
		if name == "n" or name == "p" then
			local nr = table.clone(r)
			for i = #nr + 1, at - 1 do
				nr[i] = ""
			end
			nr[at] = math.max(1, pg + (if name == "n" then 1 else -1))
			navigate(state, nr)
			return true
		end
	end
	return false
end

local function key(state: any, k: Key, _cx: BlockCx): boolean
	DB.sync()
	local name = state.focus
	if not name then
		return shortcut(state, k)
	end
	if k.name == "escape" then
		focus(state, nil)
		if state.menu then
			state.menu = nil
		end
		return true
	elseif k.name == "tab" then
		local list = page_fields(state)
		local at = table.find(list, name) or 0
		local step = if k.shift then -1 else 1
		local nxt = list[((at - 1 + step) % math.max(#list, 1)) + 1]
		focus(state, nxt)
		return true
	end
	local f = state.f[name]
	if not f then
		focus(state, nil)
		return true
	end
	local outcome = Field.key(f, k, MULTILINE[name] == true)
	if outcome == "submit" then
		submit(state, FORM_OF[name])
		return true
	end
	return outcome == "edit"
end

local function view(state: any, cx: BlockCx): BlockView
	DB.sync()
	state.wide = cx.cols >= WIDE_COLS
	state.where = nil
	local page = (PAGES[state.route[1]] or Home)(state)
	state.title = page.title
	local main: Node = { k = "col", p = { role = "vb.main" }, c = { Chrome.frame(state, page) } }
	local ops = {}
	if state.refocus then
		state.refocus = false
		local id = state.focus and find_id(main, "main", "f_" .. state.focus)
		table.insert(ops, { "focus", id or tern.json.null })
	end
	if state.reveal then
		local id = find_id(main, "main", state.reveal)
		state.reveal = nil
		if id then
			table.insert(ops, { "reveal", id, "start" })
		end
	end
	if #ops > 0 then
		-- Applied after this render's frame, which adds the nodes they name.
		cx:frame(ops)
	end
	return { main = main }
end

tern.block.define("forum", {
	font = "sans",
	init = function(cx: BlockCx, args: { string }, saved: any): any
		DB.sync()
		local state = {
			cx = cx,
			route = { "home" } :: Route,
			hist = {},
			f = fields(),
			focus = nil,
			refocus = false,
			reveal = nil,
			menu = nil,
			form = default_form(),
			tsort = { by = "last", asc = false },
			msort = { by = "name", asc = true },
			result = nil,
			errors = nil,
			preview = false,
			redirect = nil,
			qr_post = nil,
			cur = nil,
			wide = true,
			where = nil,
			title = nil,
		}
		if type(saved) == "table" then
			if type(saved.route) == "table" and PAGES[saved.route[1]] and saved.route[1] ~= "results" and saved.route[1] ~= "redirect" then
				state.route = saved.route
			end
			if type(saved.tsort) == "table" then
				state.tsort = { by = tostring(saved.tsort.by or "last"), asc = saved.tsort.asc == true }
			end
		elseif args[1] then
			state.route = parse(args[1])
		end
		return state
	end,
	title = function(state: any): string?
		if state.title then
			return state.title .. " - " .. DB.title
		end
		return DB.title
	end,
	view = view,
	key = key,
	event = event,
	save = function(state: any): any
		return { route = state.route, tsort = state.tsort }
	end,
})
