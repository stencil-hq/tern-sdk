--!strict
-- The `explorer` block: a JSON file as a collapsible tree. The file path is
-- the block's first launch argument ("Open with › JSON Explorer", the
-- window's route, or `cx:new_block("jsonx.explorer", { path })`).
--
-- Keys: ↑/↓ (or k/j) move, → (or l) expands or steps in, ← (or h) collapses
-- or steps out, Enter/Space toggles, Home/End and PageUp/PageDown jump,
-- y copies the jq path, c collapses everything, r reloads, o opens the file
-- as text.

local ui = tern.ui

-- Files above this are refused: parsing runs in Luau, inside one 2-second
-- host call.
local MAX_BYTES = 4 * 1024 * 1024
-- Containers deeper than this are refused (the parser recurses).
local MAX_DEPTH = 512
-- Children an expanded container shows before a "… N more" row.
local CHILD_ROWS = 1000
-- Rows the tree shows in total; expanding past it shows a notice instead.
local MAX_ROWS = 20000
-- Characters of a string value shown in its row.
local PREVIEW = 200

-- A parsed JSON value. `tern.json.decode` can't be used for a viewer: Lua
-- tables forget member order and the decoder drops `null` members, so the
-- block parses the text itself and keeps numbers as written.
type Value = {
	kind: string, -- "object" | "array" | "string" | "number" | "boolean" | "null"
	text: string?, -- scalars: the string's value, or the literal as written
	keys: { string }?, -- objects: member names in document order
	items: { Value }?, -- containers: member values or elements in order
}

-- One visible line of the tree. `path` is its jq path and the list item key.
type Row = {
	path: string,
	label: string, -- breadcrumb segment: a member name or `[i]`
	name: string?, -- object members: the name
	depth: number,
	parent: string?,
	value: Value?, -- nil for a "… N more" row
	more: number?, -- "… N more" rows: how many children are hidden
}

-- Why there is no tree: `nofile`, `large`, `read` or `parse`.
type Problem = { kind: string, title: string, text: string, diag: Diag? }

type State = {
	cx: BlockCx,
	path: string?,
	name: string,
	root: Value?,
	bytes: number,
	dupes: number,
	loading: boolean,
	problem: Problem?,
	expanded: { [string]: boolean },
	limits: { [string]: number },
	rows: { Row },
	index: { [string]: number },
	capped: boolean,
	cursor: string,
}

type ParseError = { msg: string, pos: number }

local ESCAPES: { [string]: string } = {
	['"'] = '"',
	["\\"] = "\\",
	["/"] = "/",
	b = "\b",
	f = "\f",
	n = "\n",
	r = "\r",
	t = "\t",
}

-- Literals are immutable, so every occurrence shares one table.
local TRUE: Value = table.freeze({ kind = "boolean", text = "true" })
local FALSE: Value = table.freeze({ kind = "boolean", text = "false" })
local NULL: Value = table.freeze({ kind = "null", text = "null" })

-- Parses JSON text into a `Value`; raises a `ParseError` at the first fault.
-- Also returns how many duplicate member names it folded (the last wins,
-- as in jq).
local function parse(src: string): (Value, number)
	local pos = 1
	local dupes = 0

	local function fail(msg: string, at: number?)
		local err: ParseError = { msg = msg, pos = at or pos }
		error(err, 0)
	end

	local function skip()
		pos = string.find(src, "[^ \t\r\n]", pos) or #src + 1
	end

	local function expect(byte: number, what: string)
		skip()
		if string.byte(src, pos) ~= byte then
			fail(if pos > #src then "unexpected end of file, expected " .. what else "expected " .. what)
		end
		pos += 1
	end

	local function str(): string
		local open = pos
		pos += 1
		local parts: { string } = {}
		local seg = pos
		while true do
			local at = string.find(src, '["\\%c]', pos)
			if at == nil then
				fail("unterminated string", open)
				return ""
			end
			local b = string.byte(src, at)
			if b == 34 then -- the closing quote
				table.insert(parts, string.sub(src, seg, at - 1))
				pos = at + 1
				return table.concat(parts)
			elseif b == 92 then -- a backslash escape
				table.insert(parts, string.sub(src, seg, at - 1))
				local e = string.sub(src, at + 1, at + 1)
				if e == "u" then
					local hex = string.match(src, "^%x%x%x%x", at + 2)
					if hex == nil then
						fail("\\u needs four hex digits", at)
						return ""
					end
					local code = tonumber(hex, 16) :: number
					local after = at + 6
					if code >= 0xD800 and code <= 0xDBFF then
						local lo = string.match(src, "^\\u(%x%x%x%x)", after)
						local low = if lo then tonumber(lo, 16) :: number else 0
						if low >= 0xDC00 and low <= 0xDFFF then
							code = 0x10000 + (code - 0xD800) * 0x400 + (low - 0xDC00)
							after += 6
						else
							code = 0xFFFD -- a lone high surrogate
						end
					elseif code >= 0xDC00 and code <= 0xDFFF then
						code = 0xFFFD -- a lone low surrogate
					end
					table.insert(parts, utf8.char(code))
					pos = after
				else
					local ch = ESCAPES[e]
					if ch == nil then
						fail("invalid escape \\" .. e, at)
					end
					table.insert(parts, ch)
					pos = at + 2
				end
				seg = pos
			elseif b == 127 then
				pos = at + 1 -- DEL may appear unescaped
			else
				fail("control character in a string; it must be escaped", at)
			end
		end
	end

	local function number(): Value
		local start = pos
		if string.byte(src, pos) == 45 then -- -
			pos += 1
		end
		local _, int = string.find(src, "^0", pos)
		if int == nil then
			_, int = string.find(src, "^[1-9]%d*", pos)
		end
		if int == nil then
			fail("invalid number", start)
			return NULL
		end
		pos = int + 1
		if string.byte(src, pos) == 46 then -- .
			local _, frac = string.find(src, "^%.%d+", pos)
			if frac == nil then
				fail("a digit must follow the decimal point")
				return NULL
			end
			pos = frac + 1
		end
		local e = string.byte(src, pos)
		if e == 101 or e == 69 then -- e E
			local _, exp = string.find(src, "^[eE][%+%-]?%d+", pos)
			if exp == nil then
				fail("invalid exponent")
				return NULL
			end
			pos = exp + 1
		end
		return { kind = "number", text = string.sub(src, start, pos - 1) }
	end

	local function literal(word: string, v: Value): Value
		if string.sub(src, pos, pos + #word - 1) ~= word then
			fail("unexpected `" .. string.sub(src, pos, pos) .. "`")
		end
		pos += #word
		return v
	end

	local value: (depth: number) -> Value

	local function container(depth: number, object: boolean): Value
		if depth > MAX_DEPTH then
			fail("nested deeper than " .. MAX_DEPTH .. " levels")
		end
		local close = if object then 125 else 93 -- } ]
		pos += 1
		local items: { Value } = {}
		local keys: { string } = {}
		local seen: { [string]: number } = {}
		skip()
		if string.byte(src, pos) == close then
			pos += 1
		else
			while true do
				if object then
					skip()
					if string.byte(src, pos) ~= 34 then
						fail("expected a member name in double quotes")
					end
					local key = str()
					expect(58, "`:` after the member name")
					local v = value(depth + 1)
					local at = seen[key]
					if at then
						items[at] = v
						dupes += 1
					else
						table.insert(keys, key)
						table.insert(items, v)
						seen[key] = #items
					end
				else
					table.insert(items, value(depth + 1))
				end
				skip()
				local b = string.byte(src, pos)
				pos += 1
				if b == close then
					break
				elseif b ~= 44 then -- ,
					pos -= 1
					fail(if object then "expected `,` or `}`" else "expected `,` or `]`")
				end
			end
		end
		if object then
			return { kind = "object", keys = keys, items = items }
		end
		return { kind = "array", items = items }
	end

	value = function(depth: number): Value
		skip()
		local b = string.byte(src, pos)
		if b == 123 then -- {
			return container(depth, true)
		elseif b == 91 then -- [
			return container(depth, false)
		elseif b == 34 then -- "
			return { kind = "string", text = str() }
		elseif b == 116 then
			return literal("true", TRUE)
		elseif b == 102 then
			return literal("false", FALSE)
		elseif b == 110 then
			return literal("null", NULL)
		elseif b == 45 or (b ~= nil and b >= 48 and b <= 57) then
			return number()
		elseif b == nil then
			fail("unexpected end of file, expected a value")
		else
			fail("unexpected `" .. string.sub(src, pos, pos) .. "`")
		end
		return NULL
	end

	if string.sub(src, 1, 3) == "\239\187\191" then
		pos = 4 -- a UTF-8 byte order mark
	end
	skip()
	if pos > #src then
		fail("the file is empty")
	end
	local root = value(0)
	skip()
	if pos <= #src then
		fail("unexpected text after the JSON value")
	end
	return root, dupes
end

local function basename(path: string): string
	return string.match(path, "([^/]+)/*$") or path
end

local function plural(n: number, word: string): string
	return n .. " " .. word .. (if n == 1 then "" else "s")
end

local function bytes(n: number): string
	if n < 1024 then
		return plural(n, "byte")
	elseif n < 1024 * 1024 then
		return string.format("%.1f KiB", n / 1024)
	end
	return string.format("%.1f MiB", n / (1024 * 1024))
end

-- The first `max` characters of `s`, and whether it was longer.
local function clip(s: string, max: number): (string, boolean)
	if #s <= max then
		return s, false
	end
	local cut = utf8.offset(s, max + 1)
	if cut == nil then
		return s, false
	end
	return string.sub(s, 1, cut - 1), true
end

-- A diagnostic for a parse error at byte `pos`: line, column and an excerpt
-- of that line around the column.
local function locate(src: string, pos: number, msg: string, path: string): Diag
	local newlines, start = 0, 1
	while true do
		local nl = string.find(src, "\n", start, true)
		if nl == nil or nl >= pos then
			break
		end
		newlines += 1
		start = nl + 1
	end
	local text = string.match(src, "^[^\n]*", start) or ""
	local col = pos - start + 1
	if #text > 160 then -- minified JSON: one very long line
		local from = math.max(1, col - 60)
		text = (if from > 1 then "…" else "") .. string.sub(text, from, from + 119) .. "…"
	end
	return {
		severity = "error",
		message = msg,
		path = path,
		line = newlines + 1,
		col = col,
		snippet = { (newlines + 1) :: any, text },
	}
end

-- The jq path of a child of `parent`: member `name`, or element `i` (0-based).
local function child_path(parent: string, name: string?, i: number): string
	local seg = if name == nil
		then "[" .. i .. "]"
		elseif string.find(name, "^[%a_][%w_]*$") then "." .. name
		else "[" .. tern.json.encode(name) .. "]"
	if parent == "." then
		return if string.sub(seg, 1, 1) == "." then seg else "." .. seg
	end
	return parent .. seg
end

local function is_container(v: Value?): boolean
	return v ~= nil and (v.kind == "object" or v.kind == "array")
end

-- Rebuilds the visible rows from the tree, the expanded set and the limits.
local function relayout(state: State)
	local rows: { Row } = {}
	local index: { [string]: number } = {}
	state.capped = false
	local function add(row: Row)
		table.insert(rows, row)
		index[row.path] = #rows
	end
	local function walk(v: Value, row: Row)
		if #rows >= MAX_ROWS then
			state.capped = true
			return
		end
		add(row)
		if not (is_container(v) and state.expanded[row.path]) then
			return
		end
		local items = v.items or {}
		local keys = v.keys
		local limit = state.limits[row.path] or CHILD_ROWS
		for i = 1, math.min(#items, limit) do
			local name = if keys then keys[i] else nil
			local path = child_path(row.path, name, i - 1)
			walk(items[i], {
				path = path,
				label = name or ("[" .. (i - 1) .. "]"),
				name = name,
				depth = row.depth + 1,
				parent = row.path,
				value = items[i],
			})
			if state.capped then
				return
			end
		end
		if #items > limit then
			add({
				path = row.path .. "#more",
				label = "…",
				depth = row.depth + 1,
				parent = row.path,
				more = #items - limit,
			})
		end
	end
	local root = state.root
	if root then
		walk(root, { path = ".", label = state.name, depth = 0, value = root })
	end
	state.rows = rows
	state.index = index
	if index[state.cursor] == nil then
		state.cursor = "."
	end
end

-- Reads and parses the file now; on success the tree replaces any problem.
local function read(state: State)
	local path = state.path :: string
	local ok, src = pcall(tern.fs.read, path)
	if not ok then
		state.problem = {
			kind = "read",
			title = "Could not read " .. state.name,
			text = tostring(src),
		}
		return
	end
	if #src > MAX_BYTES then -- it grew since `wc`, or `wc` was not there
		state.bytes = #src
		state.problem = {
			kind = "large",
			title = state.name .. " is too large to explore",
			text = state.name .. " is " .. bytes(#src) .. "; JSON Explorer reads files up to " .. bytes(MAX_BYTES) .. ".",
		}
		return
	end
	if utf8.len(src) == nil then
		state.problem = {
			kind = "read",
			title = state.name .. " is not UTF-8 text",
			text = "JSON files are UTF-8; this one has bytes that are not.",
		}
		return
	end
	local parsed, root, dupes = pcall(parse, src)
	if not parsed then
		local err = root :: any
		if type(err) == "table" then
			state.problem = {
				kind = "parse",
				title = state.name .. " is not valid JSON",
				text = err.msg,
				diag = locate(src, err.pos, err.msg, path),
			}
		else
			state.problem = { kind = "read", title = "Could not parse " .. state.name, text = tostring(err) }
		end
		return
	end
	state.root = root
	state.dupes = dupes
	state.bytes = #src
	state.problem = nil
	relayout(state)
end

-- Loads the file in the background: `wc -c` measures it first, so a huge
-- file is refused before it is read into memory.
local function load(state: State)
	if state.path == nil or state.loading then
		return
	end
	state.loading = true
	local argv = { "wc", "-c", "--", state.path :: string }
	local started = pcall(function()
		tern.process.run(argv, { timeout_ms = 5000 }, function(result: ProcessResult, _)
			state.loading = false
			local size = tonumber(string.match(result.stdout, "^%s*(%d+)"))
			if result.status == 0 and size and size > MAX_BYTES then
				state.bytes = size
				state.problem = {
					kind = "large",
					title = state.name .. " is too large to explore",
					text = state.name .. " is " .. bytes(size) .. "; JSON Explorer reads files up to " .. bytes(MAX_BYTES) .. ".",
				}
			else
				-- wc failing (no such file, a directory) is reported by the read.
				read(state)
			end
			state.cx:render()
		end)
	end)
	if not started then
		-- No `wc` here (or no processes at all): read directly.
		state.loading = false
		read(state)
	end
end

local function button(label: string, act: string, tone: string): Node
	local node = ui.badge(label, tone)
	local p: { [string]: any } = node.p or {}
	p.actions = { click = act }
	node.p = p
	return node
end

local function problem_view(state: State, problem: Problem): Node
	local children: { Node } = {}
	local diag = problem.diag
	if diag then
		table.insert(children, ui.diagnostic(diag, state.cx.cwd))
	elseif problem.kind == "nofile" then
		table.insert(children, ui.md(problem.text))
	else
		table.insert(children, ui.text({ ui.span(problem.text) }))
	end
	if state.path then
		local buttons: { Node } = { button("Open as text", "open", "accent") }
		table.insert(buttons, button("Reload", "reload", "neutral"))
		table.insert(children, ui.row(buttons))
	end
	local card = ui.card({ ui.span(problem.title, "strong") }, children)
	local p: { [string]: any } = card.p or {}
	p.tone = if problem.kind == "large" or problem.kind == "nofile" then "warning" else "error"
	card.p = p
	return card
end

-- The value part of a scalar row.
local function scalar(v: Value): Span
	local text = v.text or ""
	if v.kind == "string" then
		local shown, cut = clip(text, PREVIEW)
		local quoted = tern.json.encode(shown)
		if cut then
			quoted = string.sub(quoted, 1, -2) .. "…"
		end
		return ui.span(quoted, "success")
	elseif v.kind == "number" then
		return ui.span(text, "num")
	elseif v.kind == "boolean" then
		return ui.span(text, "accent")
	end
	return ui.span(text, "muted")
end

local function name_span(name: string): Span
	if name == "" or string.find(name, "%c") then
		return ui.span(tern.json.encode(name), "info")
	end
	return ui.span(name, "info")
end

local function item(state: State, row: Row): Item
	local indent = string.rep("  ", row.depth)
	local hidden = row.more
	if hidden then
		return {
			label = { ui.span(indent .. "  … " .. hidden .. " more", "muted") },
			detail = { ui.span("Enter shows " .. math.min(hidden, CHILD_ROWS) .. " more", "dim") },
		}
	end
	local v = row.value :: Value
	local glyph = if not is_container(v) then "  " elseif state.expanded[row.path] then "▾ " else "▸ "
	local label: { Span | string } = { ui.span(indent .. glyph, "muted") }
	if row.depth == 0 then
		table.insert(label, ui.span(state.name, "strong"))
	elseif row.name then
		table.insert(label, name_span(row.name))
	else
		table.insert(label, ui.span(row.label, "muted num"))
	end
	if not is_container(v) then
		table.insert(label, ui.span(": ", "muted"))
		table.insert(label, scalar(v))
		return { label = label }
	end
	local n = #(v.items or {})
	local count = if v.kind == "object" then plural(n, "key") else plural(n, "item")
	return { label = label, value = { ui.span(count, "muted num") } }
end

-- The tree as a list keyed `tree`: its id is `main.tree` and each row's
-- `main.tree.<jq path>`, the ids `selected` and the list's events use.
local LIST_ID = "main.tree."

-- How many rows the list shows before it scrolls itself. A list with a
-- `max` height keeps its selected row in view; without one, `main` scrolls
-- and the cursor can leave the screen. List rows are about a third taller
-- than the pane's cells, and the dock takes about five cells.
local function list_lines(cx: BlockCx): number
	return math.max(4, math.floor((cx.rows - 5) * 0.72))
end

local function tree_view(state: State, cx: BlockCx): Node
	local items: { Item } = {}
	for _, row in state.rows do
		table.insert(items, item(state, row))
	end
	local list = ui.list(items)
	local kids = list.c or {}
	for i, row in state.rows do
		local p: { [string]: any } = kids[i].p or {}
		p.key = row.path
		kids[i].p = p
	end
	local p: { [string]: any } = list.p or {}
	p.key = "tree"
	p.selected = LIST_ID .. state.cursor
	p.max = { lines = list_lines(cx) }
	list.p = p
	return list
end

-- The dock: the breadcrumb, the jq path and size, notices, the keys.
local function dock_view(state: State): Node
	local chain: { Row } = {}
	local at: string? = state.cursor
	while at do
		local row = state.rows[state.index[at] or 0]
		if row == nil then
			break
		end
		table.insert(chain, 1, row)
		at = row.parent
	end
	local crumbs: { Span | string } = {}
	for i, row in chain do
		if i > 1 then
			table.insert(crumbs, ui.span("  ›  ", "dim"))
		end
		table.insert(crumbs, ui.span(row.label, if i == #chain then "strong" else "muted"))
	end

	local cursor = state.rows[state.index[state.cursor] or 0]
	local info: { Span | string } = { ui.span(if cursor and cursor.more then "" else state.cursor, "mono") }
	local v = cursor and cursor.value
	if v then
		local what = if v.kind == "object"
			then " · " .. plural(#(v.keys or {}), "key")
			elseif v.kind == "array" then " · " .. plural(#(v.items or {}), "item")
			elseif v.kind == "string" then " · " .. plural(utf8.len(v.text or "") or #(v.text or ""), "character")
			else ""
		table.insert(info, ui.span("  " .. v.kind .. what, "muted"))
	end
	table.insert(info, ui.span("  " .. bytes(state.bytes), "dim"))

	local lines: { Node } = { ui.text(crumbs), ui.text(info) }
	if state.capped then
		table.insert(
			lines,
			ui.text({ ui.span("Showing the first " .. MAX_ROWS .. " rows: collapse something to see the rest.", "warning") })
		)
	end
	if state.dupes > 0 then
		local what = plural(state.dupes, "duplicate member name")
		table.insert(lines, ui.text({ ui.span(what .. ": the last value of each is shown, as jq does.", "warning") }))
	end
	table.insert(
		lines,
		ui.text({ ui.span("↑↓ move  ←→ fold  ⏎ toggle  y copy path  c collapse all  r reload  o open as text", "dim") })
	)
	return ui.col({ ui.lines(lines) })
end

local function move(state: State, to: number)
	local row = state.rows[math.clamp(to, 1, math.max(1, #state.rows))]
	if row then
		state.cursor = row.path
	end
end

-- Enter on a row: toggles a container, or shows more of a long one.
local function toggle(state: State, row: Row)
	if row.more and row.parent then
		-- The cursor lands on the first child the "more" row revealed.
		local at = state.index[row.path] or 1
		state.limits[row.parent] = (state.limits[row.parent] or CHILD_ROWS) + CHILD_ROWS
		relayout(state)
		move(state, at)
	elseif is_container(row.value) then
		if state.expanded[row.path] then
			state.expanded[row.path] = nil
		else
			state.expanded[row.path] = true
		end
		relayout(state)
	end
end

local function open_as_text(state: State, cx: BlockCx)
	if state.path then
		-- A block's open goes through link routes and then opens as a
		-- link would (origin "link"), which this plugin's route leaves alone.
		cx:open(state.path :: string)
	end
end

tern.block.define("explorer", {
	init = function(cx: BlockCx, args: { string }, saved: any): State
		local path: string? = args[1]
		if path and string.sub(path, 1, 1) ~= "/" and cx.cwd then
			path = cx.cwd .. "/" .. path
		end
		local state: State = {
			cx = cx,
			path = path,
			name = if path then basename(path) else "JSON Explorer",
			root = nil,
			bytes = 0,
			dupes = 0,
			loading = false,
			problem = nil,
			expanded = { ["."] = true },
			limits = {},
			rows = {},
			index = {},
			capped = false,
			cursor = ".",
		}
		if type(saved) == "table" and type(saved.expanded) == "table" then
			state.expanded = {}
			for _, p in saved.expanded do
				if type(p) == "string" then
					state.expanded[p] = true
				end
			end
			if type(saved.cursor) == "string" then
				state.cursor = saved.cursor
			end
		end
		if path == nil then
			state.problem = {
				kind = "nofile",
				title = "No file to explore",
				text = "Open a `.json` file from the Files pane, or choose **Open with › JSON Explorer** in a file's menu.",
			}
		else
			load(state)
		end
		return state
	end,

	title = function(state: State): string?
		return state.name
	end,

	view = function(state: State, cx: BlockCx): BlockView
		-- A region draws its root node's children, not the root itself (a
		-- list there would lose its selection and events), so each view
		-- sits inside a column.
		if state.root and not state.problem then
			return { main = ui.col({ tree_view(state, cx) }), dock = dock_view(state) }
		elseif state.problem then
			return { main = ui.col({ problem_view(state, state.problem) }) }
		end
		return { main = ui.col({ ui.progress(nil, "Reading " .. state.name .. "…") }) }
	end,

	key = function(state: State, key: Key, cx: BlockCx): boolean?
		if key.ctrl or key.alt or key.meta then
			return false
		end
		local k = key.text or key.name
		if k == "r" then
			load(state)
			return true
		elseif k == "o" then
			open_as_text(state, cx)
			return true
		end
		if state.root == nil or state.problem then
			return false
		end
		local i = state.index[state.cursor] or 1
		local row = state.rows[i]
		local page = math.max(1, cx.rows - 6)
		if key.name == "up" or k == "k" then
			move(state, i - 1)
		elseif key.name == "down" or k == "j" then
			move(state, i + 1)
		elseif key.name == "page_up" then
			move(state, i - page)
		elseif key.name == "page_down" then
			move(state, i + page)
		elseif key.name == "home" or k == "g" then
			move(state, 1)
		elseif key.name == "end" or k == "G" then
			move(state, #state.rows)
		elseif key.name == "right" or k == "l" then
			local v = row.value
			if v and is_container(v) and not state.expanded[row.path] then
				toggle(state, row)
			elseif v and is_container(v) and #(v.items or {}) > 0 then
				move(state, i + 1) -- step onto the first child
			end
		elseif key.name == "left" or k == "h" then
			if is_container(row.value) and state.expanded[row.path] then
				toggle(state, row)
			elseif row.parent then
				state.cursor = row.parent
			end
		elseif key.name == "enter" or key.name == "space" then
			toggle(state, row)
		elseif k == "y" then
			if row.more == nil then
				cx:copy(row.path)
				cx:toast("success", "Copied jq path", row.path)
			end
		elseif k == "c" then
			state.expanded = { ["."] = true }
			state.cursor = "."
			relayout(state)
		else
			return false
		end
		return true
	end,

	event = function(state: State, ev: UiEvent, cx: BlockCx)
		if ev.ev == "action" and ev.act == "open" then
			open_as_text(state, cx)
		elseif ev.ev == "action" and ev.act == "reload" then
			load(state)
		elseif (ev.ev == "select" or ev.ev == "activate") and type(ev.item) == "string" then
			local path = string.sub(ev.item, #LIST_ID + 1)
			local row = state.rows[state.index[path] or 0]
			if row then
				state.cursor = path
				if ev.ev == "activate" then
					toggle(state, row)
				end
			end
		end
	end,

	save = function(state: State): any
		local open: { string } = {}
		for path in state.expanded do
			table.insert(open, path)
		end
		table.sort(open) -- stable output, so an unchanged set isn't sent again
		return { expanded = tern.json.array(open), cursor = state.cursor }
	end,
})
