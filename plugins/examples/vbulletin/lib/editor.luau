-- The message editor (vBulletin's vB_Editor in standard mode): a toolbar
-- of BBCode buttons over a text area, the smilies box, and the error box
-- forms show above themselves.

local H = require("./h")
local P = require("./parts")
local BB = require("./bbcode")

local E = {}

-- Toolbar buttons: label, tooltip, BBCode open and close tags.
local TOOLS = {
	{ "B", "Bold", "[B]", "[/B]", "tb-b" },
	{ "I", "Italic", "[I]", "[/I]", "tb-i" },
	{ "U", "Underline", "[U]", "[/U]", "tb-u" },
	"-",
	{ "Red", "Font Color: Red", "[COLOR=Red]", "[/COLOR]", "tb-color" },
	{ "Big", "Size 4", "[SIZE=4]", "[/SIZE]", "tb-size" },
	"-",
	{ "•", "Insert Bulleted List", "[LIST]\n[*]", "\n[/LIST]", "tb-list" },
	{ "1.", "Insert Numbered List", "[LIST=1]\n[*]", "\n[/LIST]", "tb-list" },
	"-",
	{ "Link", "Insert Link", "[URL=http://]", "[/URL]", "tb-link" },
	{ "“ ”", "Wrap [QUOTE] tags around selected text", "[QUOTE]", "[/QUOTE]", "tb-quote" },
	{ "#", "Wrap [CODE] tags around selected text", "[CODE]", "[/CODE]", "tb-code" },
}

-- The editor for field `name`: toolbar, text area, and with `smilies` the
-- clickable smilies box beside it.
function E.editor(state: any, name: string, smilies: boolean): Node
	local tools: { any } = {}
	for i, t in TOOLS do
		if t == "-" then
			table.insert(tools, H.div("tb-sep", nil, { key = "s" .. i }))
		else
			table.insert(tools, H.div("tb-btn " .. t[5], { H.t(t[1]) }, {
				title = t[2],
				actions = { click = "bb=" .. name .. "/" .. i },
				key = "b" .. i,
			}))
		end
	end
	local box: { any } = {}
	if smilies then
		local seen = {}
		local faces = {}
		for _, s in BB.smilies do
			if not seen[s[2]] then
				seen[s[2]] = true
				table.insert(faces, H.div("sm-cell", {
					H.img("smilies/" .. s[2], "smilie=" .. name .. "/" .. s[1], s[1]),
				}, { key = s[2] }))
			end
		end
		box = {
			H.tborder({
				H.row({ H.td("thead c", { H.t("Smilies") }) }),
				H.row({ H.td("alt1 sm-grid", faces) }),
			}, "smiliebox"),
		}
	end
	return H.div("vbeditor", H.cat({
		H.div("controlbar", {
			H.div("toolbar", tools),
			P.input(state, name, {
				multiline = true,
				placeholder = "",
				aria = "Message",
			}),
		}),
	}, box), { key = "ed_" .. name })
end

-- Applies toolbar button `i` to field `f` (Field.wrap at the caret).
function E.tool(i: number): (string?, string?)
	local t = TOOLS[i]
	if type(t) ~= "table" then
		return nil, nil
	end
	return t[3], t[4]
end

-- "The following errors occurred with your submission" with `errors`.
function E.errors(errors: { string }?): Node | false
	if not errors or #errors == 0 then
		return false
	end
	local items = {}
	for _, e in errors do
		table.insert(items, H.el("li", nil, { H.t(e) }))
	end
	return H.div("errors-wrap", {
		H.tborder({
			H.row({ H.td("tcat", { H.t("The following errors occurred with your submission") }) }),
			H.row({ H.td("alt1", { H.el("ol", "errors", items) }) }),
		}),
		H.div("gap"),
	}, { key = "errors" })
end

-- The Preview box: the message as it will read once posted.
function E.preview(title: string?, body: string): Node
	return H.div("preview-wrap", {
		H.tborder({
			H.row({ H.td("tcat", { H.t("Preview") }) }),
			H.row({
				H.td("alt1", {
					title and title ~= "" and H.div("smallfont", { H.b(title) }) or false,
					title and title ~= "" and H.el("hr", "posthr") or false,
					H.div("post_message", BB.render(body)),
				}),
			}),
		}),
		H.div("gap"),
	}, { key = "preview" })
end

return E
