--!strict
-- The `plan` lens: Terraform's (and OpenTofu's) human-readable plan output
-- as summary badges, diagnostics and resources grouped by action. Clicking a
-- resource copies its address.

local ui = tern.ui

-- Rows shown per group before the rest fold into "… N more".
local GROUP_ROWS = 200

-- The diagnostic box Terraform draws around errors and warnings.
local BOX_SIDE = "│"
local BOX_END = "╵"

type Resource = { addr: string, action: string, note: string? }
type Output = { sym: string, name: string }
type Diagnostic = {
	severity: string,
	title: string,
	path: string?,
	line: number?,
	snippet: { any }?,
	notes: { string },
	para: string?,
}
type State = {
	base: string?,
	tool: string,
	section: string,
	resources: { Resource },
	drift: { Resource },
	outputs: { Output },
	summary: { [string]: number }?,
	nochanges: boolean,
	refreshed: number,
	diags: { Diagnostic },
	open: Diagnostic?,
	header: boolean,
	status: number?,
}

-- Resource headers (`# <addr> <suffix>`), longest suffix first where one
-- ends another.
local ACTIONS = {
	{ " is tainted, so must be replaced", "replace" },
	{ " will be replaced, as requested", "replace" },
	{ " must be replaced", "replace" },
	{ " will be created", "create" },
	{ " will be updated in-place", "update" },
	{ " will be destroyed", "destroy" },
	{ " will be read during apply", "read" },
	{ " will be imported", "import" },
	{ " will no longer be managed by Terraform", "forget" },
	{ " will no longer be managed by OpenTofu", "forget" },
}

-- Headers of the "Objects have changed outside of Terraform" section.
local DRIFT = {
	{ " has been deleted", "deleted" },
	{ " has changed", "changed" },
}

-- Groups in the order a reviewer reads them: what is lost first.
local GROUPS = {
	{ action = "destroy", title = "Destroy", sym = "-", style = "error" },
	{ action = "replace", title = "Replace", sym = "-/+", style = "warning" },
	{ action = "update", title = "Update in place", sym = "~", style = "warning" },
	{ action = "create", title = "Create", sym = "+", style = "success" },
	{ action = "import", title = "Import", sym = "<-", style = "info" },
	{ action = "move", title = "Move", sym = "->", style = "muted" },
	{ action = "forget", title = "Forget", sym = "/", style = "muted" },
	{ action = "read", title = "Read during apply", sym = "<=", style = "info" },
}

local function starts(s: string, prefix: string): boolean
	return string.sub(s, 1, #prefix) == prefix
end

local function ends(s: string, suffix: string): boolean
	return #s >= #suffix and string.sub(s, -#suffix) == suffix
end

-- The directory relative paths in diagnostics resolve against: the shell's
-- working directory, or the `-chdir=` one.
local function base_dir(run: LensRun): string?
	for _, arg in run.args do
		local dir = string.match(arg, "^%-chdir=(.+)$")
		if dir then
			if starts(dir, "/") or run.cwd == nil then
				return dir
			end
			return run.cwd .. "/" .. dir
		end
	end
	return run.cwd
end

-- The resource a `# …` header line announces, if it is one.
local function header(body: string, section: string): Resource?
	if section == "drift" then
		for _, pair in DRIFT do
			if ends(body, pair[1]) then
				return { addr = string.sub(body, 1, #body - #pair[1]), action = pair[2] }
			end
		end
		return nil
	end
	local from, to = string.match(body, "^(.-) has moved to (.+)$")
	if from and to then
		return { addr = to, action = "move", note = "moved from " .. from }
	end
	for _, pair in ACTIONS do
		if ends(body, pair[1]) then
			local addr = string.sub(body, 1, #body - #pair[1])
			local base, deposed = string.match(addr, "^(.-) %((deposed object %w+)%)$")
			if base and deposed then
				return { addr = base, action = pair[2], note = deposed }
			end
			return { addr = addr, action = pair[2] }
		end
	end
	return nil
end

-- Ends the paragraph a diagnostic is collecting.
local function flush(d: Diagnostic)
	if d.para then
		table.insert(d.notes, d.para)
		d.para = nil
	end
end

-- One line from inside a diagnostic box.
local function absorb(d: Diagnostic, body: string)
	local path, line = string.match(body, "^%s*on (.-) line (%d+)")
	if path and line and d.path == nil then
		d.path = path
		d.line = tonumber(line)
		return
	end
	local num, src = string.match(body, "^%s*(%d+): ?(.*)$")
	if num and src and d.path and #d.notes == 0 and d.para == nil then
		local snippet = d.snippet
		if snippet then
			snippet[2] ..= "\n" .. src
		else
			d.snippet = { tonumber(num) :: any, src }
		end
		return
	end
	local text = string.match(body, "^%s*(.-)%s*$") or ""
	if text == "" then
		flush(d)
	elseif starts(text, "├") then
		return -- the rule above value annotations
	elseif starts(text, BOX_SIDE) then
		flush(d)
		table.insert(d.notes, (string.gsub(string.sub(text, #BOX_SIDE + 1), "^%s+", "")))
	else
		d.para = if d.para then d.para .. " " .. text else text
	end
end

local function close(state: State)
	local open = state.open
	if open then
		flush(open)
		state.open = nil
	end
end

-- A clickable resource row: its symbol, address and note.
local function resource_row(r: Resource, sym: string, style: string): Node
	local spans: { Span | string } = { ui.span(sym, style .. " strong"), " ", ui.span(r.addr, "mono") }
	if r.note then
		table.insert(spans, ui.span("  " .. r.note, "muted"))
	end
	local node = ui.text(spans)
	local p: { [string]: any } = node.p or {}
	p.role = "terraform." .. r.action
	p.title = "Copy address"
	p.actions = { click = "copy-address=" .. r.addr }
	node.p = p
	return node
end

-- A collapsible group of rows, keyed so its collapsed state survives updates.
local function group(key: string, head: Spans, rows: { Node }, total: number, collapsed: boolean): Node
	if total > #rows then
		table.insert(rows, ui.overflow(total - #rows))
	end
	local node = ui.section(head, rows)
	local p: { [string]: any } = node.p or {}
	p.key = key
	p.collapsible = true
	p.collapsed = collapsed
	node.p = p
	return node
end

local function counts(state: State): { [string]: number }
	local n: { [string]: number } = {}
	for _, r in state.resources do
		n[r.action] = (n[r.action] or 0) + 1
	end
	return n
end

local function badges(state: State): { Node }
	local out: { Node } = {}
	local errors, warnings = 0, 0
	for _, d in state.diags do
		if d.severity == "error" then
			errors += 1
		else
			warnings += 1
		end
	end
	local summary = state.summary
	if summary then
		local any = false
		for _, b in
			{
				{ "import", "to import", "info" },
				{ "add", "to add", "success" },
				{ "change", "to change", "warning" },
				{ "destroy", "to destroy", "error" },
				{ "forget", "to forget", "muted" },
			}
		do
			local n = summary[b[1]] or 0
			if n > 0 then
				any = true
				table.insert(out, ui.badge(n .. " " .. b[2], b[3]))
			end
		end
		if not any then
			table.insert(out, ui.badge("No resource changes", "muted"))
		end
	elseif state.nochanges then
		table.insert(out, ui.badge("No changes", "success"))
	elseif state.status == nil then
		table.insert(out, ui.badge("Planning", "pending"))
		local n = counts(state)
		for _, g in GROUPS do
			if n[g.action] then
				table.insert(out, ui.badge(n[g.action] .. " " .. string.lower(g.title), "muted"))
			end
		end
	elseif state.status ~= 0 and state.status ~= 2 then
		-- `-detailed-exitcode` exits 2 for a plan with changes, 1 for an error.
		table.insert(out, ui.badge("Plan failed", "error"))
	end
	if #state.outputs > 0 then
		local label = if #state.outputs == 1 then " output change" else " output changes"
		table.insert(out, ui.badge(#state.outputs .. label, "info"))
	end
	if errors > 0 then
		table.insert(out, ui.badge(errors .. (if errors == 1 then " error" else " errors"), "error"))
	end
	if warnings > 0 then
		table.insert(out, ui.badge(warnings .. (if warnings == 1 then " warning" else " warnings"), "warning"))
	end
	if state.refreshed > 0 then
		table.insert(out, ui.text({ ui.span(state.refreshed .. " refreshed", "muted") }))
	end
	return out
end

local function view(state: State): Node?
	local empty = #state.resources == 0
		and #state.drift == 0
		and #state.outputs == 0
		and #state.diags == 0
		and state.summary == nil
		and not state.nochanges
	if empty then
		return nil -- not a plan we can read (`-json`, `-help`): raw output
	end

	local children: { Node } = { ui.row(badges(state)) }
	for _, d in state.diags do
		table.insert(
			children,
			ui.diagnostic({
				severity = d.severity :: any,
				message = d.title,
				path = d.path,
				line = d.line,
				snippet = d.snippet,
				notes = d.notes,
			}, state.base)
		)
	end

	local danger = false
	for _, g in GROUPS do
		local rows: { Node } = {}
		local total = 0
		for _, r in state.resources do
			if r.action == g.action then
				total += 1
				if total <= GROUP_ROWS then
					table.insert(rows, resource_row(r, g.sym, g.style))
				end
			end
		end
		if total > 0 then
			danger = danger or g.action == "destroy" or g.action == "replace"
			local head = { ui.span(g.title, "strong " .. g.style), ui.span("  " .. total, "muted num") }
			table.insert(children, group(g.action, head, rows, total, g.action == "read"))
		end
	end

	if #state.drift > 0 then
		local rows: { Node } = {}
		for i, r in state.drift do
			if i > GROUP_ROWS then
				break
			end
			table.insert(rows, resource_row(r, "!", "muted"))
		end
		local head = {
			ui.span("Changed outside " .. state.tool, "strong muted"),
			ui.span("  " .. #state.drift, "muted num"),
		}
		table.insert(children, group("drift", head, rows, #state.drift, true))
	end

	if #state.outputs > 0 then
		local rows: { Node } = {}
		for _, o in state.outputs do
			local style = if o.sym == "+" then "success" elseif o.sym == "-" then "error" else "warning"
			local spans: { Span | string } = { ui.span(o.sym, style .. " strong"), " ", ui.span(o.name, "mono") }
			table.insert(rows, ui.text(spans))
		end
		local head = { ui.span("Outputs", "strong"), ui.span("  " .. #state.outputs, "muted num") }
		table.insert(children, group("outputs", head, rows, #state.outputs, false))
	end

	local root = ui.col(children)
	local p: { [string]: any } = root.p or {}
	p.role = if danger then "terraform.plan.danger" else "terraform.plan"
	root.p = p
	return root
end

tern.lens.define("plan", {
	open = function(run: LensRun, _cx: EffectCx): State
		return {
			base = base_dir(run),
			tool = if run.program == "tofu" then "OpenTofu" else "Terraform",
			section = "",
			resources = {},
			drift = {},
			outputs = {},
			summary = nil,
			nochanges = false,
			refreshed = 0,
			diags = {},
			open = nil,
			header = false,
			status = nil,
		}
	end,

	line = function(state: State, l: LensLine)
		local text = l.text

		-- Diagnostics: `╷`, then `│ Error: title` and its body, then `╵`.
		if starts(text, BOX_SIDE) then
			local body = string.gsub(string.sub(text, #BOX_SIDE + 1), "^ ", "")
			local severity, title = string.match(body, "^(%a+): (.+)$")
			if (severity == "Error" or severity == "Warning") and title then
				close(state)
				local d: Diagnostic = { severity = string.lower(severity), title = title, notes = {} }
				table.insert(state.diags, d)
				state.open = d
			elseif state.open then
				absorb(state.open, body)
			end
			return
		end
		if starts(text, BOX_END) then
			close(state)
			return
		end
		local bare_sev, bare_title = string.match(text, "^(%a+): (.+)$")
		if (bare_sev == "Error" or bare_sev == "Warning") and bare_title then
			-- Without the box (older releases): the title alone.
			table.insert(state.diags, { severity = string.lower(bare_sev), title = bare_title, notes = {} })
			return
		end

		-- Section markers.
		if starts(text, "Note: Objects have changed outside of ") then
			state.section = "drift"
		elseif string.find(text, "^%a+ will perform the following actions:") then
			state.section = "actions"
		elseif text == "Changes to Outputs:" then
			state.section = "outputs"
			return
		end

		-- Resource headers and the `# (because …)` notes right under them.
		local body = string.match(text, "^%s*# (.+)$")
		if body then
			local note = string.match(body, "^%((.*)%)$")
			local last = state.resources[#state.resources]
			if note then
				if state.header and last and last.note == nil then
					last.note = note
				end
				return
			end
			local r = header(body, state.section)
			if r then
				table.insert(if state.section == "drift" then state.drift else state.resources, r)
				state.header = state.section ~= "drift"
				return
			end
		end
		state.header = false

		if state.section == "outputs" then
			local sym, name = string.match(text, "^  ([%+~%-]) ([%w_%-]+)")
			if sym and name then
				table.insert(state.outputs, { sym = sym, name = name })
			elseif text ~= "" and not starts(text, " ") then
				state.section = ""
			end
		end

		local plan = string.match(text, "^Plan: (.+)$")
		if plan then
			local summary: { [string]: number } = {}
			for n, what in string.gmatch(plan, "(%d+) to (%a+)") do
				if n and what then
					summary[what] = tonumber(n) or 0
				end
			end
			state.summary = summary
		elseif starts(text, "No changes.") then
			state.nochanges = true
		elseif string.find(text, ": Refreshing state...", 1, true) then
			state.refreshed += 1
		end
	end,

	finish = function(state: State, status: number?)
		close(state)
		state.status = status
	end,

	view = view,

	event = function(_state: State, ev: UiEvent, cx: EffectCx)
		if ev.ev ~= "action" or ev.act ~= "copy-address" or type(ev.value) ~= "string" then
			return
		end
		cx:copy(ev.value)
		cx:toast("success", "Copied resource address", ev.value)
	end,
})
