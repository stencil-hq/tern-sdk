--!strict
-- Reads `.tern-env` files: one `KEY=VALUE` per line, `#` comments, an
-- optional `export ` prefix, and values that are bare, 'single-quoted'
-- (literal) or "double-quoted" (escapes `\n \t \" \\ \$`). Bare and
-- double-quoted values expand `${NAME}`; single-quoted ones never do.
--
-- Parsing keeps a value as parts (literal text and variable names), so
-- expanding it later, against whatever came before, is a concatenation.

local envfile = {}

-- Literal `text`, or the name (`var`) of a variable to substitute.
export type Part = { text: string?, var: string? }

export type Entry = {
	key: string,
	parts: { Part },
	line: number,
}

local ESCAPES: { [string]: string } = { n = "\n", t = "\t", ['"'] = '"', ["\\"] = "\\", ["$"] = "$" }

-- Appends `text` to `parts`, splitting out `${NAME}` references.
local function expanding(parts: { Part }, text: string)
	local at = 1
	while true do
		local from, to, name = string.find(text, "%${([%a_][%w_]*)}", at)
		if from == nil then
			break
		end
		if from > at then
			table.insert(parts, { text = string.sub(text, at, from - 1) })
		end
		table.insert(parts, { var = name :: string })
		at = (to :: number) + 1
	end
	if at <= #text then
		table.insert(parts, { text = string.sub(text, at) })
	end
end

-- The parts of a double-quoted value starting after its opening quote, and
-- what follows the closing quote; nil when the quote never closes.
local function quoted(value: string): ({ Part }?, string)
	local parts: { Part } = {}
	local run = {} -- characters since the last escape, still to be expanded
	local i = 1
	while i <= #value do
		local c = string.sub(value, i, i)
		if c == '"' then
			expanding(parts, table.concat(run))
			return parts, string.sub(value, i + 1)
		elseif c == "\\" and i < #value then
			local escaped = ESCAPES[string.sub(value, i + 1, i + 1)]
			if escaped then
				expanding(parts, table.concat(run))
				table.clear(run)
				-- An escaped character is literal: `\$` never starts `${`.
				table.insert(parts, { text = escaped })
				i += 2
				continue
			end
		end
		table.insert(run, c)
		i += 1
	end
	return nil, ""
end

-- Whether what follows a closing quote is only blanks and a comment.
local function trailing_ok(rest: string): boolean
	return string.match(rest, "^%s*$") ~= nil or string.match(rest, "^%s+#") ~= nil
end

-- The entries of `text` in file order, and a message for each line that
-- doesn't read (`"line 4: …"`); those lines are skipped.
function envfile.parse(text: string): ({ Entry }, { string })
	local entries: { Entry } = {}
	local problems: { string } = {}
	local number = 0
	for raw in string.gmatch(text .. "\n", "([^\n]*)\n") do
		number += 1
		local line = string.match(raw, "^%s*(.-)%s*$") :: string
		line = string.gsub(line, "^export%s+", "")
		if line == "" or string.sub(line, 1, 1) == "#" then
			continue
		end
		local key, value = string.match(line, "^([%a_][%w_]*)%s*=%s*(.*)$")
		if key == nil or value == nil then
			table.insert(problems, string.format("line %d: expected KEY=VALUE", number))
			continue
		end
		local parts: { Part } = {}
		local first = string.sub(value, 1, 1)
		if first == "'" then
			local body, rest = string.match(value, "^'([^']*)'(.*)$")
			if body == nil or not trailing_ok(rest :: string) then
				table.insert(problems, string.format("line %d: unterminated single quote", number))
				continue
			end
			table.insert(parts, { text = body })
		elseif first == '"' then
			local got, rest = quoted(string.sub(value, 2))
			if got == nil or not trailing_ok(rest) then
				table.insert(problems, string.format("line %d: unterminated double quote", number))
				continue
			end
			parts = got
		else
			-- A bare value ends at a ` #` comment.
			local bare = string.gsub(value, "%s+#.*$", "")
			expanding(parts, bare)
		end
		table.insert(entries, { key = key, parts = parts, line = number })
	end
	return entries, problems
end

-- `parts` with each variable replaced by `lookup(name)` (empty when nil).
function envfile.expand(parts: { Part }, lookup: (string) -> string?): string
	local out = table.create(#parts)
	for _, part in parts do
		if part.var then
			table.insert(out, lookup(part.var) or "")
		else
			table.insert(out, part.text or "")
		end
	end
	return table.concat(out)
end

return envfile
