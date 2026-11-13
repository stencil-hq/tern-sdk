-- The board: seed.luau's users, forums and threads, plus what this machine
-- added (your posts and threads, read marks, views, votes), kept in
-- `tern.kv` under "board" so every block of the plugin sees one board.
--
-- Seed times are minutes before the board's epoch, the moment the board was
-- first opened on this machine, so a new board looks lively and an old one
-- ages like a real forum.

local seed = require("../seed")

local KV = "board"
-- vBulletin's hot-thread thresholds (views raised for a board whose seed
-- threads carry years of views).
local HOT_REPLIES = 15
local HOT_VIEWS = 1500

local DB = {}

export type User = {
	id: number,
	name: string,
	group: string,
	title: string,
	joined: number,
	posts: number,
	location: string?,
	occupation: string?,
	interests: string?,
	bio: string?,
	sig: string?,
	avatar: string?,
	online: boolean,
	last: number,
	birthday: string?,
	me: boolean,
}

export type Forum = {
	id: number,
	parent: number?,
	title: string,
	desc: string?,
	kind: string,
	link: string?,
	closed: boolean,
	children: { Forum },
	threads: { Thread },
	-- Including sub-forums:
	nthreads: number,
	nposts: number,
	last: Post?,
}

export type Post = {
	id: number,
	thread: Thread,
	user: User,
	time: number,
	title: string?,
	icon: string?,
	body: string,
	n: number,
	mine: boolean,
}

export type Thread = {
	id: number,
	forum: number,
	title: string,
	icon: string?,
	sticky: boolean,
	locked: boolean,
	views: number,
	rating: number?,
	votes: number,
	posts: { Post },
	first: Post,
	last: Post,
	starter: User,
}

-- What this machine added to the seed, as stored in `tern.kv`.
type Overlay = {
	rev: number,
	epoch: number,
	lastvisit: number,
	readall: number,
	read: { [string]: number },
	views: { [string]: number },
	votes: { [string]: number },
	subs: { [string]: boolean },
	collapsed: { [string]: boolean },
	posts: { any },
	threads: { any },
}

DB.title = seed.title
DB.users = {} :: { User }
DB.user = {} :: { [number]: User }
DB.forums = {} :: { Forum }
DB.forum = {} :: { [number]: Forum }
DB.threads = {} :: { Thread }
DB.thread = {} :: { [number]: Thread }
DB.post = {} :: { [number]: Post }
DB.me = (nil :: any) :: User

local o: Overlay = (nil :: any) :: Overlay
local built_rev = -1

-- Your forum name: the login name of the host's user.
local function my_name(): string
	local name = tern.getenv("USER") or tern.getenv("USERNAME")
	if not name or name == "" then
		return "you"
	end
	return name
end

local function read_overlay(): Overlay
	local v = tern.kv.get(KV)
	local now = os.time()
	if type(v) ~= "table" or type(v.epoch) ~= "number" then
		v = { rev = 1, epoch = now, lastvisit = now - 3 * 3600 }
		v.readall = 0
	end
	v.rev = v.rev or 1
	v.readall = v.readall or 0
	v.lastvisit = v.lastvisit or v.epoch
	for _, k in { "read", "views", "votes", "subs", "collapsed" } do
		if type(v[k]) ~= "table" then
			v[k] = {}
		end
	end
	for _, k in { "posts", "threads" } do
		if type(v[k]) ~= "table" then
			v[k] = {}
		end
	end
	return v :: Overlay
end

local function write_overlay()
	o.rev += 1
	local copy = table.clone(o) :: any
	copy.posts = tern.json.array(o.posts)
	copy.threads = tern.json.array(o.threads)
	tern.kv.set(KV, copy)
end

-- The user title vBulletin's default ladder gives `u`.
local function title_of(u: User, custom: string?): string
	if u.group == "banned" then
		return "Banned"
	elseif custom then
		return custom
	elseif u.group == "admin" then
		return "Administrator"
	elseif u.group == "smod" then
		return "Super Moderator"
	elseif u.group == "mod" then
		return "Moderator"
	elseif u.posts < 30 then
		return "Junior Member"
	elseif u.posts < 100 then
		return "Member"
	end
	return "Senior Member"
end

local function build()
	local epoch = o.epoch
	local now = os.time()
	DB.users, DB.user = {}, {}
	DB.forums, DB.forum = {}, {}
	DB.threads, DB.thread, DB.post = {}, {}, {}

	local customs: { [number]: string? } = {}
	for _, s in seed.users do
		local u: User = {
			id = s.id,
			name = s.name,
			group = s.group,
			title = "",
			joined = epoch - s.joined * 86400,
			posts = s.posts,
			location = s.location,
			occupation = s.occupation,
			interests = s.interests,
			bio = s.bio,
			sig = s.sig,
			avatar = s.avatar,
			online = s.online,
			last = now - s.last * 60,
			birthday = s.birthday,
			me = false,
		}
		customs[u.id] = s.usertitle
		table.insert(DB.users, u)
		DB.user[u.id] = u
	end
	local me: User = {
		id = 1000,
		name = my_name(),
		group = "member",
		title = "",
		joined = epoch,
		posts = 0,
		online = true,
		last = now,
		me = true,
	}
	table.insert(DB.users, me)
	DB.user[me.id] = me
	DB.me = me

	for _, s in seed.forums do
		local f: Forum = {
			id = s.id,
			parent = s.parent,
			title = s.title,
			desc = s.desc,
			kind = s.kind,
			link = s.link,
			closed = s.closed == true,
			children = {},
			threads = {},
			nthreads = 0,
			nposts = 0,
			last = nil,
		}
		table.insert(DB.forums, f)
		DB.forum[f.id] = f
		if f.parent and DB.forum[f.parent] then
			table.insert(DB.forum[f.parent].children, f)
		end
	end

	-- Threads, then every post in time order so post ids rise with time.
	local all: { Post } = {}
	local function add_thread(s: any, created: number?): Thread
		local t: Thread = {
			id = s.id,
			forum = s.forum,
			title = s.title,
			icon = s.icon,
			sticky = s.sticky == true,
			locked = s.locked == true,
			views = (s.views or 0) + (o.views[tostring(s.id)] or 0),
			rating = s.rating,
			votes = s.votes or 0,
			posts = {},
			first = (nil :: any) :: Post,
			last = (nil :: any) :: Post,
			starter = (nil :: any) :: User,
		}
		local mine = o.votes[tostring(t.id)]
		if mine then
			local sum = (t.rating or 0) * t.votes + mine
			t.votes += 1
			t.rating = sum / t.votes
		end
		table.insert(DB.threads, t)
		DB.thread[t.id] = t
		local f = DB.forum[t.forum]
		if f then
			table.insert(f.threads, t)
		end
		return t
	end
	for _, s in seed.threads do
		local t = add_thread(s)
		for _, sp in s.posts do
			local u = DB.user[sp.user] or me
			table.insert(all, {
				id = 0,
				thread = t,
				user = u,
				time = epoch - sp.ago * 60,
				title = sp.title,
				icon = sp.icon,
				body = sp.body,
				n = 0,
				mine = false,
			})
		end
	end
	for _, s in o.threads do
		add_thread({ id = s.id, forum = s.forum, title = s.title, icon = s.icon, views = 0 })
	end
	for _, s in o.posts do
		local t = DB.thread[s.thread]
		if t then
			table.insert(all, {
				id = 0,
				thread = t,
				user = me,
				time = s.time,
				title = s.title,
				icon = s.icon,
				body = s.body,
				n = 0,
				mine = true,
			})
		end
	end
	table.sort(all, function(a, b)
		if a.time ~= b.time then
			return a.time < b.time
		end
		return a.thread.id < b.thread.id
	end)
	for i, p in all do
		p.id = i
		DB.post[i] = p
		table.insert(p.thread.posts, p)
		p.n = #p.thread.posts
	end

	-- Seeded post counts are archive posts plus the posts on the board.
	for _, p in all do
		p.user.posts += 1
	end
	for _, u in DB.users do
		u.title = title_of(u, customs[u.id])
	end

	for i = #DB.threads, 1, -1 do
		local t = DB.threads[i]
		if #t.posts == 0 then
			table.remove(DB.threads, i)
			DB.thread[t.id] = nil
			local f = DB.forum[t.forum]
			if f then
				local at = table.find(f.threads, t)
				if at then
					table.remove(f.threads, at)
				end
			end
		else
			t.first = t.posts[1]
			t.last = t.posts[#t.posts]
			t.starter = t.first.user
		end
	end

	-- Forum totals include sub-forums: walk children first.
	local function total(f: Forum)
		local nthreads, nposts, last = 0, 0, nil :: Post?
		for _, t in f.threads do
			nthreads += 1
			nposts += #t.posts
			if not last or t.last.time > last.time then
				last = t.last
			end
		end
		for _, c in f.children do
			total(c)
			nthreads += c.nthreads
			nposts += c.nposts
			if c.last and (not last or c.last.time > last.time) then
				last = c.last
			end
		end
		f.nthreads, f.nposts, f.last = nthreads, nposts, last
	end
	for _, f in DB.forums do
		if not f.parent then
			total(f)
		end
	end
	built_rev = o.rev
end

-- Loads the board, again when another block changed it.
function DB.sync()
	o = read_overlay()
	if built_rev ~= o.rev then
		build()
	end
end

-- When you last visited (the "new posts" line).
function DB.lastvisit(): number
	return o.lastvisit
end

-- The time posts are new after in thread `t`.
local function seen(t: Thread): number
	return math.max(o.lastvisit, o.readall, o.read[tostring(t.id)] or 0)
end

-- Whether thread `t` has posts you haven't read.
function DB.unread(t: Thread): boolean
	return t.last.time > seen(t)
end

-- The first post of `t` you haven't read, or nil.
function DB.first_unread(t: Thread): Post?
	local since = seen(t)
	for _, p in t.posts do
		if p.time > since then
			return p
		end
	end
	return nil
end

-- Whether post `p` is new to you.
function DB.post_new(p: Post): boolean
	return p.time > seen(p.thread)
end

-- Whether forum `f` or a sub-forum has unread threads.
function DB.forum_unread(f: Forum): boolean
	for _, t in f.threads do
		if DB.unread(t) then
			return true
		end
	end
	for _, c in f.children do
		if DB.forum_unread(c) then
			return true
		end
	end
	return false
end

-- Whether `t` counts as hot (vBulletin: 15 replies or 150 views).
function DB.hot(t: Thread): boolean
	return #t.posts - 1 >= HOT_REPLIES or t.views >= HOT_VIEWS
end


-- Category `f` is collapsed on the forum home.
function DB.collapsed(key: string): boolean
	return o.collapsed[key] == true
end

function DB.set_collapsed(key: string, on: boolean)
	o.collapsed[key] = if on then true else nil
	write_overlay()
end

-- Marks thread `t` read up to `time`, and counts a view.
function DB.visit(t: Thread, time: number)
	local key = tostring(t.id)
	o.read[key] = math.max(o.read[key] or 0, time)
	o.views[key] = (o.views[key] or 0) + 1
	write_overlay()
	t.views += 1
	built_rev = o.rev
end

-- Mark Forums Read.
function DB.mark_all_read()
	o.readall = os.time()
	write_overlay()
end

-- Your vote on thread `t`, or nil.
function DB.my_vote(t: Thread): number?
	return o.votes[tostring(t.id)]
end

-- Rates thread `t` `n` stars; vBulletin lets you vote once.
function DB.vote(t: Thread, n: number)
	o.votes[tostring(t.id)] = n
	write_overlay()
	build()
end

function DB.subscribed(t: Thread): boolean
	return o.subs[tostring(t.id)] == true
end

function DB.set_subscribed(t: Thread, on: boolean)
	o.subs[tostring(t.id)] = if on then true else nil
	write_overlay()
end

-- Threads you subscribed to, newest activity first.
function DB.subscriptions(): { Thread }
	local out = {}
	for key in o.subs do
		local t = DB.thread[tonumber(key) or 0]
		if t then
			table.insert(out, t)
		end
	end
	table.sort(out, function(a, b)
		return a.last.time > b.last.time
	end)
	return out
end

-- Adds your reply to thread `t`; returns the new post.
function DB.reply(t: Thread, body: string, title: string?, icon: string?): Post
	local now = os.time()
	table.insert(o.posts, { thread = t.id, time = now, body = body, title = title, icon = icon })
	o.read[tostring(t.id)] = now
	write_overlay()
	build()
	local th = DB.thread[t.id]
	return th.last
end

-- Starts a thread in forum `f`; returns it.
function DB.new_thread(f: Forum, title: string, body: string, icon: string?): Thread
	local id = 1
	for _, t in DB.threads do
		id = math.max(id, t.id + 1)
	end
	local now = os.time()
	table.insert(o.threads, { id = id, forum = f.id, title = title, icon = icon })
	table.insert(o.posts, { thread = id, time = now, body = body, title = title, icon = icon })
	o.read[tostring(id)] = now
	write_overlay()
	build()
	return DB.thread[id]
end


-- `f`'s ancestors from its category down, without `f`.
function DB.trail(f: Forum): { Forum }
	local out = {}
	local at = f.parent and DB.forum[f.parent]
	while at do
		table.insert(out, 1, at)
		at = at.parent and DB.forum[at.parent]
	end
	return out
end

-- Every thread of forum `f` (not sub-forums), stickies first then by `sort`.
function DB.forum_threads(f: Forum, sort: string, asc: boolean): { Thread }
	local list = table.clone(f.threads)
	local function key(t: Thread): any
		if sort == "title" then
			return string.lower(t.title)
		elseif sort == "starter" then
			return string.lower(t.starter.name)
		elseif sort == "replies" then
			return #t.posts
		elseif sort == "views" then
			return t.views
		elseif sort == "rating" then
			return t.rating or 0
		end
		return t.last.time
	end
	table.sort(list, function(a, b)
		if a.sticky ~= b.sticky then
			return a.sticky
		end
		local ka, kb = key(a), key(b)
		if ka == kb then
			return a.last.time > b.last.time
		end
		if asc then
			return ka < kb
		end
		return ka > kb
	end)
	return list
end

-- Counts on the forum home: threads, posts, members, newest member.
function DB.totals(): (number, number, number, User)
	local nposts = 0
	for _, t in DB.threads do
		nposts += #t.posts
	end
	local newest = DB.users[1]
	for _, u in DB.users do
		if u.joined > newest.joined or (u.joined == newest.joined and u.me) then
			newest = u
		end
	end
	return #DB.threads, nposts, #DB.users, newest
end

-- Users active in the last 15 minutes, you included.
function DB.online(): { User }
	local out = {}
	for _, u in DB.users do
		if u.online then
			table.insert(out, u)
		end
	end
	table.sort(out, function(a, b)
		return string.lower(a.name) < string.lower(b.name)
	end)
	return out
end

-- Guests browsing: a steady number for the board's size.
function DB.guests(): number
	return 7 + (os.date("*t").hour % 5)
end

-- Users whose birthday is `month`-`day`.
function DB.birthdays(month: number, day: number): { User }
	local key = string.format("%02d-%02d", month, day)
	local out = {}
	for _, u in DB.users do
		if u.birthday == key then
			table.insert(out, u)
		end
	end
	return out
end

-- Finds a user by name, case-insensitively: exact, else the first that
-- contains it unless `exact`.
function DB.find_user(name: string, exact: boolean): { User }
	local want = string.lower(name)
	local out = {}
	for _, u in DB.users do
		local n = string.lower(u.name)
		if n == want or (not exact and string.find(n, want, 1, true)) then
			table.insert(out, u)
		end
	end
	return out
end

-- Lowercase search words of `query`, with the ones vBulletin drops (under
-- 3 letters) in `skipped`.
function DB.words(query: string): ({ string }, { string })
	local words, skipped = {}, {}
	for w in string.gmatch(string.lower(query), "[^%s%p]+") do
		if #w < 3 then
			table.insert(skipped, w)
		elseif not table.find(words, w) then
			table.insert(words, w)
		end
	end
	return words, skipped
end

local function has_all(text: string, words: { string }): boolean
	local hay = string.lower(text)
	for _, w in words do
		if not string.find(hay, w, 1, true) then
			return false
		end
	end
	return true
end

-- Whether forum `f` is `root` or under it.
local function within(f: Forum?, root: number): boolean
	local at = f
	while at do
		if at.id == root then
			return true
		end
		at = at.parent and DB.forum[at.parent]
	end
	return false
end

export type Query = {
	words: { string },
	users: { User }?,
	starter: boolean,
	titles: boolean,
	forum: number?,
	thread: number?,
	since: number?,
	unread: boolean?,
}

-- Posts matching `q`, newest first.
function DB.search_posts(q: Query): { Post }
	local out = {}
	for _, t in DB.threads do
		local f = DB.forum[t.forum]
		if (not q.forum or within(f, q.forum)) and (not q.thread or t.id == q.thread) then
			for _, p in t.posts do
				local ok = true
				if q.users then
					local who = if q.starter then t.starter else p.user
					ok = table.find(q.users, who) ~= nil
					if ok and q.starter then
						ok = p == t.first
					end
				end
				if ok and #q.words > 0 then
					local text = if q.titles then t.title else (p.title or "") .. " " .. t.title .. " " .. p.body
					if q.titles and p ~= t.first then
						ok = false
					else
						ok = has_all(text, q.words)
					end
				end
				if ok and q.since then
					ok = p.time >= q.since
				end
				if ok and q.unread then
					ok = DB.post_new(p)
				end
				if ok then
					table.insert(out, p)
				end
			end
		end
	end
	table.sort(out, function(a, b)
		return a.time > b.time
	end)
	return out
end

-- Threads holding posts matching `q`, by last post, newest first.
function DB.search_threads(q: Query): { Thread }
	local seen_t: { [Thread]: boolean } = {}
	local out = {}
	for _, p in DB.search_posts(q) do
		if not seen_t[p.thread] then
			seen_t[p.thread] = true
			table.insert(out, p.thread)
		end
	end
	table.sort(out, function(a, b)
		return a.last.time > b.last.time
	end)
	return out
end

local STOP = {
	["the"] = true, ["and"] = true, ["for"] = true, ["with"] = true, ["your"] = true,
	["what"] = true, ["how"] = true, ["from"] = true, ["this"] = true, ["that"] = true,
	["tern"] = true, ["does"] = true, ["anyone"] = true, ["help"] = true, ["into"] = true,
}

-- Up to five threads whose titles share words with `t`'s.
function DB.similar(t: Thread): { Thread }
	local mine = {}
	for w in string.gmatch(string.lower(t.title), "[%w]+") do
		if #w >= 4 and not STOP[w] then
			mine[w] = true
		end
	end
	local scored = {}
	for _, other in DB.threads do
		if other ~= t then
			local score = 0
			for w in string.gmatch(string.lower(other.title), "[%w]+") do
				if mine[w] then
					score += 1
				end
			end
			if score > 0 then
				table.insert(scored, { t = other, score = score })
			end
		end
	end
	table.sort(scored, function(a, b)
		if a.score ~= b.score then
			return a.score > b.score
		end
		return a.t.last.time > b.t.last.time
	end)
	local out = {}
	for i = 1, math.min(5, #scored) do
		table.insert(out, scored[i].t)
	end
	return out
end

-- The thread before and after `t` in its forum by last post.
function DB.neighbours(t: Thread): (Thread?, Thread?)
	local f = DB.forum[t.forum]
	if not f then
		return nil, nil
	end
	local older, newer = nil :: Thread?, nil :: Thread?
	for _, other in f.threads do
		if other ~= t then
			if other.last.time < t.last.time and (not older or other.last.time > older.last.time) then
				older = other
			elseif other.last.time > t.last.time and (not newer or other.last.time < newer.last.time) then
				newer = other
			end
		end
	end
	return older, newer
end

return DB
