-- BBCode to view nodes, as vBulletin 3.8's parser renders a post: [b] [i]
-- [u] [s] [color] [size] [url] [quote] [code] [list] [highlight], smilies,
-- and search words highlighted. Unknown or unbalanced tags stay text.

local H = require("./h")

local BB = {}

-- Smilie codes and their images, longest codes first so `:cool:` wins
-- over `:c…`.
local SMILIES = {
	{ ":confused:", "confused" },
	{ ":rolleyes:", "rolleyes" },
	{ ":redface:", "redface" },
	{ ":cool:", "cool" },
	{ ":mad:", "mad" },
	{ ":eek:", "eek" },
	{ ":)", "smile" },
	{ ":(", "frown" },
	{ ":D", "biggrin" },
	{ ";)", "wink" },
	{ ":p", "tongue" },
	{ ":P", "tongue" },
	{ ":o", "eek" },
	{ ":O", "eek" },
}

-- Smilie names and codes for the smilies box beside the editor.
BB.smilies = SMILIES

-- Colors [color=…] takes (vBulletin's editor palette), by lowercase name.
local COLORS = {
	black = "black", red = "red", darkred = "darkred", orange = "orange", gold = "gold",
	yellow = "gold", green = "green", darkgreen = "darkgreen", olive = "olive", teal = "teal",
	blue = "blue", darkblue = "darkblue", navy = "navy", indigo = "indigo", purple = "purple",
	gray = "gray", grey = "gray", silver = "silver", white = "white", brown = "brown",
	sienna = "brown", deeppink = "pink", pink = "pink",
}

local BLOCK = { quote = true, code = true, list = true }
local KNOWN = {
	b = true, i = true, u = true, s = true, color = true, size = true, url = true,
	quote = true, code = true, list = true, highlight = true, font = true, left = true,
	center = true, right = true, indent = true,
}

type Tag = { name: string, arg: string?, c: { any } }

-- Parses `src` into a tree of tags and text strings.
local function parse(src: string): { any }
	local root: Tag = { name = "", c = {} }
	local stack: { Tag } = { root }
	local i = 1
	local n = #src
	local function text(s: string)
		if s == "" then
			return
		end
		local top = stack[#stack]
		local last = top.c[#top.c]
		if type(last) == "string" then
			top.c[#top.c] = last .. s
		else
			table.insert(top.c, s)
		end
	end
	while i <= n do
		local open = string.find(src, "[", i, true)
		if not open then
			text(string.sub(src, i))
			break
		end
		text(string.sub(src, i, open - 1))
		local close = string.find(src, "]", open + 1, true)
		if not close then
			text(string.sub(src, open))
			break
		end
		local inner = string.sub(src, open + 1, close - 1)
		local top = stack[#stack]
		-- Inside [code] only [/code] closes; everything else is text.
		if top.name == "code" and string.lower(inner) ~= "/code" then
			text(string.sub(src, open, close))
		elseif string.sub(inner, 1, 1) == "/" then
			local name = string.lower(string.sub(inner, 2))
			local at = 0
			for j = #stack, 2, -1 do
				if stack[j].name == name then
					at = j
					break
				end
			end
			if at > 0 then
				while #stack >= at do
					table.remove(stack)
				end
			else
				text(string.sub(src, open, close))
			end
		elseif inner == "*" and top.name == "list" then
			table.insert(top.c, { name = "*", c = {} })
		else
			local name, arg = string.match(inner, "^(%a+)=(.*)$")
			name = name or string.match(inner, "^(%a+)$")
			name = name and string.lower(name)
			if name and KNOWN[name] then
				if arg then
					arg = string.gsub(arg, "^[\"']", "")
					arg = string.gsub(arg, "[\"']$", "")
				end
				local tag: Tag = { name = name, arg = arg, c = {} }
				table.insert(top.c, tag)
				table.insert(stack, tag)
			else
				text(string.sub(src, open, close))
			end
		end
		i = close + 1
	end
	return root.c
end

-- List items: the children of a [list] split at its [*] markers.
local function items(c: { any }): { { any } }
	local out = {}
	for _, v in c do
		if type(v) == "table" and v.name == "*" then
			table.insert(out, v.c)
		elseif #out > 0 then
			table.insert(out[#out], v)
		end
	end
	return out
end

export type Opts = {
	-- Words to mark as a search hit.
	hl: { string }?,
	-- Run when a quote's "View Post" arrow is clicked: the quoted post.
	find_post: ((name: string) -> number?)?,
}

-- Text `s` as spans: smilies as images, `hl` words marked.
local function inline(s: string, out: { Node }, opts: Opts)
	local i = 1
	local n = #s
	local buf = i
	local function flush(to: number)
		if to >= buf then
			local chunk = string.sub(s, buf, to)
			if opts.hl and #opts.hl > 0 then
				local lower = string.lower(chunk)
				local at = 1
				while at <= #chunk do
					local best, best_e = nil :: number?, 0
					for _, w in opts.hl do
						local a, e = string.find(lower, w, at, true)
						if a and (not best or a < best) then
							best, best_e = a, e :: number
						end
					end
					if not best then
						table.insert(out, H.t(string.sub(chunk, at)))
						break
					end
					if best > at then
						table.insert(out, H.t(string.sub(chunk, at, best - 1)))
					end
					table.insert(out, H.t(string.sub(chunk, best, best_e), "highlight"))
					at = best_e + 1
				end
			else
				table.insert(out, H.t(chunk))
			end
		end
	end
	while i <= n do
		local hit: { string }? = nil
		local c = string.sub(s, i, i)
		if c == ":" or c == ";" then
			for _, sm in SMILIES do
				local code = sm[1]
				if string.sub(s, i, i + #code - 1) == code then
					-- `:p` and friends only stand alone, so `host:port` keeps its text.
					local after = string.sub(s, i + #code, i + #code)
					local before = if i > 1 then string.sub(s, i - 1, i - 1) else " "
					local closed = string.sub(code, -1) == ":"
					if closed or ((after == "" or not string.match(after, "[%w]")) and not string.match(before, "[%w/]")) then
						hit = sm
						break
					end
				end
			end
		end
		if hit then
			flush(i - 1)
			table.insert(out, H.el("span", "img smilie i-smilies-" .. hit[2], nil, { title = hit[1] }))
			i += #hit[1]
			buf = i
		else
			i += 1
		end
	end
	flush(n)
end

-- The text of children `c`, tags dropped (a [url]'s target, code bodies).
local function plain(c: { any }): string
	local parts = {}
	for _, v in c do
		if type(v) == "string" then
			table.insert(parts, v)
		else
			table.insert(parts, plain(v.c))
		end
	end
	return table.concat(parts)
end

-- Drops one newline at the start and end of `c`'s text: the line breaks
-- around a block tag aren't blank lines.
local function trim_block(c: { any })
	local first = c[1]
	if type(first) == "string" then
		c[1] = (string.gsub(first, "^\r?\n", ""))
	end
	local last = c[#c]
	if type(last) == "string" then
		c[#c] = (string.gsub(last, "\r?\n$", ""))
	end
end

local render: (c: { any }, opts: Opts) -> { Node }

local function node(v: any, opts: Opts, out: { Node })
	if type(v) == "string" then
		inline(v, out, opts)
		return
	end
	local name, arg = v.name, v.arg
	if name == "b" then
		table.insert(out, H.el("strong", nil, render(v.c, opts)))
	elseif name == "i" then
		table.insert(out, H.el("em", nil, render(v.c, opts)))
	elseif name == "u" then
		table.insert(out, H.el("span", "u", render(v.c, opts)))
	elseif name == "s" then
		table.insert(out, H.el("del", nil, render(v.c, opts)))
	elseif name == "highlight" then
		table.insert(out, H.el("span", "highlight", render(v.c, opts)))
	elseif name == "color" then
		local key = COLORS[string.lower(arg or "")]
		table.insert(out, H.el("span", key and "c-" .. key or nil, render(v.c, opts)))
	elseif name == "size" then
		local size = tonumber(arg or "") or 2
		size = math.clamp(math.floor(size), 1, 7)
		table.insert(out, H.el("span", "sz-" .. size, render(v.c, opts)))
	elseif name == "font" then
		table.insert(out, H.el("span", nil, render(v.c, opts)))
	elseif name == "left" or name == "center" or name == "right" or name == "indent" then
		trim_block(v.c)
		table.insert(out, H.div("al-" .. name, render(v.c, opts)))
	elseif name == "url" then
		local href = arg or plain(v.c)
		if not string.match(href, "^%a[%w+.-]*://") then
			href = "http://" .. href
		end
		local label = if arg then render(v.c, opts) else { H.t(plain(v.c)) }
		table.insert(out, H.el("span", "a url", label, { href = href, title = href, actions = { click = "open=" .. href } }))
	elseif name == "quote" then
		trim_block(v.c)
		local who = arg and string.match(arg, "^([^;]+)")
		local post = arg and tonumber(string.match(arg, ";(%d+)$") or "")
		if who and not post and opts.find_post then
			post = opts.find_post(who)
		end
		local head: { any } = {}
		if who then
			head = {
				H.t("Originally Posted by "),
				H.b(who),
				post and H.t(" ") or false,
				post and H.img("buttons/viewpost", "go=post/" .. post, "View Post") or false,
			}
		end
		table.insert(out, H.div("bbcode_container", {
			H.div("smallfont label", { H.t("Quote:") }),
			H.div("alt2 bbcode_quote", {
				who and H.div("quote_by", head) or false,
				H.div(if who then "quote_text" else "quote_text plain", render(v.c, opts)),
			}),
		}))
	elseif name == "code" then
		trim_block(v.c)
		table.insert(out, H.div("bbcode_container", {
			H.div("smallfont label", { H.t("Code:") }),
			H.el("pre", "alt2 bbcode_code", { H.t(plain(v.c)) }),
		}))
	elseif name == "list" then
		local ordered = arg ~= nil
		local lis = {}
		for _, item in items(v.c) do
			trim_block(item)
			table.insert(lis, H.el("li", nil, render(item, opts)))
		end
		table.insert(out, H.el(if ordered then "ol" else "ul", "bblist", lis))
	else
		inline(plain(v.c), out, opts)
	end
end

render = function(c: { any }, opts: Opts): { Node }
	local out: { Node } = {}
	for i, v in c do
		-- The newline after a block tag ends the block's line, not a blank one.
		if type(v) == "string" and i > 1 then
			local prev = c[i - 1]
			if type(prev) == "table" and BLOCK[prev.name] then
				v = string.gsub(v, "^\r?\n", "")
			end
		end
		node(v, opts, out)
	end
	return out
end

-- Nodes for BBCode `src`, to go in a `white-space: pre-wrap` container.
function BB.render(src: string, opts: Opts?): { Node }
	return render(parse(src), opts or {})
end

-- `src` without tags, quotes and code: a plain preview of a post.
function BB.strip(src: string): string
	local parts = {}
	local function walk(c: { any })
		for _, v in c do
			if type(v) == "string" then
				table.insert(parts, v)
			elseif v.name ~= "quote" and v.name ~= "code" then
				walk(v.c)
			end
		end
	end
	walk(parse(src))
	local text = table.concat(parts)
	text = string.gsub(text, "%s+", " ")
	return (string.gsub(text, "^ ", ""))
end

return BB
