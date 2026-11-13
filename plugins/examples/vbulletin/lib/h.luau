-- Node builders for the board's views. Pages are plain `el` trees styled by
-- vbulletin.css with vBulletin 3.8's class names (`tborder`, `tcat`, `alt1`,
-- …), so a template reads like the PHP template it copies.
--
-- Child lists may hold `false` for a part that is left out; the builders
-- drop them, so `cond and node or false` composes without holes.

local H = {}

-- Children `c` without the `false` placeholders.
local function compact(c: { any }): { any }
	local out = {}
	for _, v in c do
		if v then
			table.insert(out, v)
		end
	end
	return out
end

-- An `el` of `tag` with classes `class`, children `c` and props `p`.
function H.el(tag: string, class: string?, c: { any }?, p: { [string]: any }?): Node
	local props: { [string]: any } = p or {}
	props.tag = tag
	if class and class ~= "" then
		props.class = class
	end
	local node: Node = { k = "el", p = props }
	if c then
		node.c = compact(c)
	end
	return node
end

-- A `div` (block).
function H.div(class: string?, c: { any }?, p: { [string]: any }?): Node
	return H.el("div", class, c, p)
end

-- A `span` of plain `text`.
function H.t(text: string, class: string?, p: { [string]: any }?): Node
	local props: { [string]: any } = p or {}
	props.text = text
	return H.el("span", class, nil, props)
end

-- `text` in bold.
function H.b(text: string, class: string?): Node
	return H.el("strong", class, nil, { text = text })
end

-- A link: `text` that runs action `act` on click (`go=forum/3` navigates).
function H.a(text: string, act: string, class: string?, p: { [string]: any }?): Node
	local props: { [string]: any } = p or {}
	props.text = text
	props.actions = { click = act }
	return H.el("span", if class then "a " .. class else "a", nil, props)
end

-- A link around children `c`.
function H.wrap(act: string, c: { any }, class: string?, p: { [string]: any }?): Node
	local props: { [string]: any } = p or {}
	props.actions = { click = act }
	return H.el("span", if class then "a " .. class else "a", c, props)
end

-- The image `name` under img/ (`statusicon/forum_new`), drawn by its CSS
-- class; with `act` it is a link.
function H.img(name: string, act: string?, title: string?): Node
	local props: { [string]: any } = {}
	if act then
		props.actions = { click = act }
	end
	if title then
		props.title = title
	end
	local class = "img i-" .. string.gsub(name, "/", "-")
	return H.el("span", if act then class .. " a" else class, nil, props)
end

-- A form button (`<input type="submit" class="button">`).
function H.button(text: string, act: string, p: { [string]: any }?): Node
	local props: { [string]: any } = p or {}
	props.text = text
	props.actions = { click = act }
	return H.el("button", "button", nil, props)
end

-- A row of the cellspacing=1 grids: cells side by side, the tborder's
-- background showing through the 1px gaps.
function H.row(c: { any }, class: string?, p: { [string]: any }?): Node
	return H.div(if class then "tr " .. class else "tr", c, p)
end

-- A cell of a `row` with classes `class` (`alt1`, `thead w-lastpost`).
function H.td(class: string, c: { any }?, p: { [string]: any }?): Node
	return H.div("td " .. class, c, p)
end

-- A `tborder` grid: a navy frame whose rows sit 1px apart.
function H.tborder(c: { any }, class: string?, p: { [string]: any }?): Node
	return H.div(if class then "tborder " .. class else "tborder", c, p)
end

-- `items` joined by `sep` nodes (a fresh span each time).
function H.join(items: { Node }, sep: string): { Node }
	local out = {}
	for i, n in items do
		if i > 1 then
			table.insert(out, H.t(sep))
		end
		table.insert(out, n)
	end
	return out
end

-- `a` followed by the items of `b`.
function H.cat(a: { any }, b: { any }): { any }
	local out = table.clone(a)
	for _, v in b do
		table.insert(out, v)
	end
	return out
end

-- Route `r` as the value of a `go=` action.
function H.go(...: any): string
	local parts = { ... }
	for i, v in parts do
		parts[i] = tostring(v)
	end
	return "go=" .. table.concat(parts, "/")
end

return H
