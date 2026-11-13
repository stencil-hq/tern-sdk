-- Text fields whose text the block owns (Tern draws `input` and `editor`
-- nodes; every key reaches the block). A field is `{ t, c }`: its text and
-- the caret as a byte offset on a character boundary.

local Field = {}

export type F = { t: string, c: number }

function Field.new(text: string?): F
	local t = text or ""
	return { t = t, c = #t }
end

local function cont(b: number?): boolean
	return b ~= nil and b >= 0x80 and b < 0xC0
end

-- The offset of the character before `c`.
local function prev(t: string, c: number): number
	if c <= 0 then
		return 0
	end
	local i = c - 1
	while i > 0 and cont(string.byte(t, i + 1)) do
		i -= 1
	end
	return i
end

-- The offset after the character at `c`.
local function next(t: string, c: number): number
	if c >= #t then
		return #t
	end
	local i = c + 1
	while i < #t and cont(string.byte(t, i + 1)) do
		i += 1
	end
	return i
end

local function space(t: string, c: number): boolean
	local ch = string.sub(t, c + 1, c + 1)
	return ch == "" or string.match(ch, "[%s%p]") ~= nil
end

local function word_left(t: string, c: number): number
	local i = c
	while i > 0 and space(t, prev(t, i)) do
		i = prev(t, i)
	end
	while i > 0 and not space(t, prev(t, i)) do
		i = prev(t, i)
	end
	return i
end

local function word_right(t: string, c: number): number
	local i = c
	while i < #t and space(t, i) do
		i = next(t, i)
	end
	while i < #t and not space(t, i) do
		i = next(t, i)
	end
	return i
end

local function line_start(t: string, c: number): number
	local i = c
	while i > 0 and string.sub(t, i, i) ~= "\n" do
		i -= 1
	end
	return i
end

local function line_end(t: string, c: number): number
	local nl = string.find(t, "\n", c + 1, true)
	return if nl then nl - 1 else #t
end

-- Characters between offsets `a` and `b`.
local function chars(t: string, a: number, b: number): number
	return utf8.len(string.sub(t, a + 1, b)) or (b - a)
end

-- The offset `n` characters after `a`, at most `limit`.
local function advance(t: string, a: number, n: number, limit: number): number
	local i = a
	for _ = 1, n do
		if i >= limit then
			break
		end
		i = next(t, i)
	end
	return math.min(i, limit)
end

local function splice(f: F, from: number, to: number, text: string)
	f.t = string.sub(f.t, 1, from) .. text .. string.sub(f.t, to + 1)
	f.c = from + #text
end

-- Inserts `text` at the caret.
function Field.insert(f: F, text: string)
	splice(f, f.c, f.c, text)
end

-- Wraps the caret in `open` and `close` (a BBCode tag pair), caret between.
function Field.wrap(f: F, open: string, close: string)
	splice(f, f.c, f.c, open .. close)
	f.c -= #close
end

export type Outcome = "edit" | "submit" | nil

-- Applies `key` to `f`: "edit" when it changed the text or caret, "submit"
-- for Enter in a one-line field (⌘/Ctrl+Enter in a multi-line one), nil for
-- keys a field doesn't take.
function Field.key(f: F, key: Key, multiline: boolean): Outcome
	local name, t, c = key.name, f.t, f.c
	if name == "paste" then
		local text = key.text or ""
		if not multiline then
			text = string.gsub(text, "[\r\n]+", " ")
		end
		Field.insert(f, text)
	elseif name == "enter" then
		if multiline and not (key.meta or key.ctrl) then
			Field.insert(f, "\n")
		else
			return "submit"
		end
	elseif name == "backspace" then
		if key.meta or (key.ctrl and key.name == "u") then
			splice(f, line_start(t, c), c, "")
		elseif key.alt or key.ctrl then
			splice(f, word_left(t, c), c, "")
		else
			splice(f, prev(t, c), c, "")
		end
	elseif name == "delete" then
		if key.alt then
			splice(f, c, word_right(t, c), "")
		else
			splice(f, c, next(t, c), "")
		end
	elseif name == "left" then
		f.c = if key.meta then line_start(t, c) elseif key.alt then word_left(t, c) else prev(t, c)
	elseif name == "right" then
		f.c = if key.meta then line_end(t, c) elseif key.alt then word_right(t, c) else next(t, c)
	elseif name == "home" or (key.ctrl and name == "a") then
		f.c = line_start(t, c)
	elseif name == "end" or (key.ctrl and name == "e") then
		f.c = line_end(t, c)
	elseif key.ctrl and name == "u" then
		splice(f, line_start(t, c), c, "")
	elseif key.ctrl and name == "k" then
		splice(f, c, line_end(t, c), "")
	elseif key.ctrl and name == "w" then
		splice(f, word_left(t, c), c, "")
	elseif name == "up" and multiline then
		local ls = line_start(t, c)
		if ls == 0 then
			f.c = 0
		else
			local col = chars(t, ls, c)
			local ps = line_start(t, ls - 1)
			f.c = advance(t, ps, col, ls - 1)
		end
	elseif name == "down" and multiline then
		local le = line_end(t, c)
		if le >= #t then
			f.c = #t
		else
			local col = chars(t, line_start(t, c), c)
			f.c = advance(t, le + 1, col, line_end(t, le + 1))
		end
	elseif key.text and key.text ~= "" and not key.ctrl and not key.meta then
		Field.insert(f, key.text)
	else
		return nil
	end
	return "edit"
end

-- The caret as UTF-16 code units, as `input` and `editor` take it.
function Field.utf16(f: F): number
	local n = 0
	for _, cp in utf8.codes(string.sub(f.t, 1, f.c)) do
		n += if cp > 0xFFFF then 2 else 1
	end
	return n
end

-- The node for field `name`: an `input`, or an `editor` when `multiline`.
-- Its key is `f_<name>`, which `Field.name_of` reads back from an event id.
function Field.node(name: string, f: F, props: { [string]: any }?): Node
	local p: { [string]: any } = props or {}
	p.key = "f_" .. name
	p.text = f.t
	p.cursor = Field.utf16(f)
	p.role = "vb.field"
	local multiline = p.multiline
	p.multiline = nil
	return { k = if multiline then "editor" else "input", p = p }
end

-- The field name in node id `id` (`main.vb.….f_kw` → "kw").
function Field.name_of(id: string): string?
	return string.match(id, "%.f_([%w_]+)$")
end

return Field
