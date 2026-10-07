-- Type definitions for the `tern` module of Tern plugins (luau-lsp definition file).
-- `tern plugin types DIR` writes this file into DIR; add its path to the
-- `luau-lsp.types.definitionFiles` setting so `host.luau` and `window.luau` type-check.
-- Members only one context has say `host only` or `window only` in their docs.

--- `"host"` in `host.luau` (daemon side), `"window"` in `window.luau` (UI side).
export type Context = "host" | "window"

--- Level of a toast in Tern's windows.
export type ToastLevel = "success" | "info" | "error"

--- Where a file or block opens relative to the focused pane.
export type How = "default" | "beside" | "below" | "split" | "tab" | "replace" | "preview"

--- What asked Tern to open a file.
export type Origin = "palette" | "files" | "drop" | "link" | "system" | "cli" | "carly"

--- Side of a pane to split or move toward.
export type Dir = "right" | "down" | "left" | "up"

--- The loaded plugin.
export type PluginInfo = {
	--- Manifest `id`.
	id: string,
	--- Manifest `name`.
	name: string,
	--- Absolute path of the plugin directory.
	dir: string,
	--- Absolute path of the plugin's data directory (holds `kv.json`).
	data: string,
}

--- The machine and VM the plugin runs in.
export type RuntimeInfo = {
	--- Operating system: `"macos"`, `"linux"`, `"windows"` or `"ios"`.
	os: string,
	--- True when Luau native codegen is active (false on iOS, in development builds and where unsupported).
	jit: boolean,
}

--- An async result; plugins register `next`, Carly's lua calls use `await`.
export type Awaitable<T> = {
	--- Operation kind for diagnostics.
	kind: string,
	--- VM-local operation id.
	id: number,
	--- Runs on the VM thread as hook `await`; window callbacks receive a fresh `cx`.
	next: (self: Awaitable<T>, fn: (result: T?, err: string?, cx: WindowCx?) -> ()) -> (),
}

--- Options of `tern.process.run`.
export type ProcessOptions = {
	--- Working directory; relative paths resolve against the plugin dir.
	cwd: string?,
	--- Variables added to the inherited environment.
	env: { [string]: string }?,
	--- Text written to the process's stdin, which then closes.
	stdin: string?,
	--- Kills the process after this many milliseconds.
	timeout_ms: number?,
}

--- Outcome of `tern.process.run`.
export type ProcessResult = {
	--- Exit code; -1 when killed (timeout) or ended by a signal.
	status: number,
	--- Captured stdout, capped at 16 MiB.
	stdout: string,
	--- Captured stderr, capped at 16 MiB.
	stderr: string,
	--- True when `timeout_ms` elapsed and the process was killed.
	timed_out: boolean,
}

--- Options of `tern.fetch`.
export type FetchOptions = {
	--- HTTP method, any case (default `"GET"`).
	method: string?,
	--- Request headers; `User-Agent` is `Tern` unless set here.
	headers: { [string]: string }?,
	--- Request body bytes; without it no body is sent.
	body: string?,
	--- Fails the request when the whole exchange, redirects included, takes longer (default 30000).
	timeout_ms: number?,
}

--- Outcome of `tern.fetch`.
export type FetchResult = {
	--- HTTP status (every status is a response); 0 when `error` is set.
	status: number,
	--- Response headers by lowercase name; a repeated header's values joined with `", "`.
	headers: { [string]: string },
	--- Response body bytes, at most 16 MiB.
	body: string,
	--- Why no whole response arrived (host, connection, TLS, timeout, body over 16 MiB); nil when one did.
	error: string?,
	--- True when the timeout ended the request.
	timed_out: boolean,
}

--- A styled text span, as `tern.ui.span`, `link` and `path` return.
export type Span = {
	--- The text.
	t: string,
	--- Space-separated style tokens (`"muted"`, `"strong error"`; `"key"` renders a keycap chip); absent for plain text.
	s: string?,
	--- Link target.
	href: string?,
}

--- A span list: spans and plain strings in order, or one span or plain string.
export type Spans = { Span | string } | Span | string

--- A TSP wire node `{k, p, c}`.
export type Node = {
	--- Node kind (`"text"`, `"col"`, `"table"`, …).
	k: string,
	--- Props; any node also takes `key`, `tone`, `role` and `collapsed`. A `list` keeps its selected row in view only with `max = {lines = N}`.
	p: { [string]: any }?,
	--- Children.
	c: { Node }?,
}

--- Cell alignment in a table column.
export type Align = "start" | "center" | "end"

--- Where a table cell that doesn't fit is cut: `end` keeps the start, `start` keeps the end.
export type Truncate = "end" | "start" | "middle"

--- A table column.
export type Col = {
	--- Id cells are keyed by.
	id: string,
	--- Header text.
	head: string,
	--- Cell alignment (default `"start"`).
	align: Align?,
	--- Columns with the lowest priority hide first when narrow (default 0).
	priority: number?,
	--- Share of spare width this column takes (default 0).
	grow: number?,
	--- Where cells that don't fit are cut (default `"end"`).
	truncate: Truncate?,
}

--- A thin-bar table cell from `tern.ui.meter_cell` or `meter_parts_cell`.
export type MeterCell = {
	--- The bar; set `meter.tone` to tint a single-value meter.
	meter: {
		--- Fill, 0 to 1.
		value: number?,
		--- Fill tinted `warn`/`bad` at or above these values.
		thresholds: { warn: number, bad: number }?,
		--- Stacked parts from the left.
		parts: { { value: number, token: string } }?,
		--- Fill tone.
		tone: string?,
	},
}

--- A table cell: a span list or a meter cell.
export type Cell = Spans | MeterCell

--- A node of `tern.ui.tree`.
export type TreeNode = {
	--- Its label.
	label: Spans,
	--- Named icon.
	icon: string?,
	--- Whether its children show (default false).
	open: boolean?,
	--- Its children.
	children: { TreeNode }?,
}

--- A row of `tern.ui.list`.
export type Item = {
	--- Main text.
	label: Spans,
	--- Secondary text after the label.
	detail: Spans?,
	--- Right-aligned value.
	value: Spans?,
	--- Named icon.
	icon: string?,
	--- Tone of the row.
	tone: string?,
}

--- How severe a diagnostic is.
export type Severity = "error" | "warning" | "note" | "help"

--- A compiler or linter diagnostic for `tern.ui.diagnostic`.
export type Diag = {
	--- How severe it is (default `"error"`).
	severity: Severity?,
	--- Lint or error code (`"E0308"`).
	code: string?,
	--- The message.
	message: string,
	--- File it points at, as printed.
	path: string?,
	--- 1-based line.
	line: number?,
	--- 1-based column.
	col: number?,
	--- `{firstLine, source}`: an excerpt and the line number of its first line.
	snippet: { any }?,
	--- Notes and help lines under it.
	notes: { string }?,
	--- Suggested edits, each `{message, firstLine, code}`.
	suggestions: { { any } }?,
}

--- A whitespace-aligned table read by `tern.parse.columns`.
export type Columns = {
	--- Header of each column.
	head: { string },
	--- Cells of each row, one per column, trimmed.
	rows: { { string } },
}

--- An event from a node of a block or lens view (`{ev = "action", act = …, …}`). Blocks get only
--- action/select/activate/change/toggle/focus/edit/undo; lens events are not filtered. An action name containing
--- `=` is split into `act` and `value`; `mods` lists `shift`/`ctrl`/`alt`/`meta`, present only when a modifier was
--- held. `change` comes from a clicked `tern.ui.el` checkbox/radio (`id`, `name`, `value`, `checked`).
export type UiEvent = {
	--- Event name.
	ev: string,
	--- The node the event came from.
	id: string?,
	--- `change`: the control's `name`, when it has one.
	name: string?,
	--- `change`: whether the control is now checked.
	checked: boolean?,
	--- action/select/activate/change inside an `el` form: its enabled named controls (radio group: the checked
	--- value, nil when none; lone checkbox: boolean; same-named checkboxes: array of checked values).
	values: { [string]: any }?,
	--- Event-specific fields.
	[string]: any,
}

--- A key press in a plugin block (paste arrives as `{name = "paste", text = …}`).
export type Key = {
	--- Key name (`"a"`, `"+"`, `"enter"`, `"up"`, `"f5"`, `"space"`, …).
	name: string,
	--- Text the key types, when it types any.
	text: string?,
	--- Ctrl held.
	ctrl: boolean,
	--- Alt/Option held.
	alt: boolean,
	--- Shift held.
	shift: boolean,
	--- Cmd/Super held.
	meta: boolean,
}

--- What a block's `view` returns: the node tree of each region. A region's root node is never drawn, only its
--- `role` (as the region's data-role) and its children: wrap content in `tern.ui.col` (a bare `text` shows nothing,
--- a bare `list` loses `selected`, `max` and its events).
export type BlockView = {
	--- The block's main area.
	main: Node?,
	--- A dock under the main area.
	dock: Node?,
	--- A layer above the main area.
	layer: Node?,
}

--- Host only: a block type, registered with `tern.block.define`; `S` is the block's state.
export type BlockDef<S> = {
	--- Creates the state from the launch `args` and the last `save` result (nil when new).
	init: (cx: BlockCx, args: { string }, saved: any) -> S,
	--- The view of `state`; Tern diffs it against the last one and sends the changes.
	view: (state: S, cx: BlockCx) -> BlockView,
	--- The pane title for `state`.
	title: ((state: S) -> string?)?,
	--- Handles a key press. Return exactly `false` when the key changed nothing (no render, no save); any other
	--- return (`true`, nil) renders and saves. Changing state and returning `false` leaves the view stale.
	key: ((state: S, key: Key, cx: BlockCx) -> boolean?)?,
	--- Handles an event from the view's nodes.
	event: ((state: S, ev: UiEvent, cx: BlockCx) -> ())?,
	--- Called when the pane resizes to `cols` × `rows` cells, then the block renders and saves; without it the block
	--- still re-renders (no save), with `cx.cols`/`cx.rows` updated.
	resize: ((state: S, cols: number, rows: number, cx: BlockCx) -> ())?,
	--- What to persist (JSON values); it is passed back to `init` after a restart.
	save: ((state: S) -> any)?,
}

--- Host only: the command a lens claimed.
export type LensRun = {
	--- The command line (expanded by the shell when it reports it, else as typed).
	line: string,
	--- The line as typed.
	typed: string,
	--- The shell's working directory, when known.
	cwd: string?,
	--- The program, after wrappers and `VAR=` prefixes.
	program: string,
	--- Its arguments.
	args: { string },
}

--- Host only: an OSC 8 hyperlink the program printed on a lens output line (1-based inclusive byte offsets into `text`).
export type LensLink = {
	--- First byte.
	from: number,
	--- Last byte.
	to: number,
	--- Link target.
	href: string,
}

--- Host only: one output line fed to a lens, when its row scrolls off an 8-row scratch screen (the last rows arrive
--- at finish); soft-wrapped rows joined, escapes applied, trailing blanks trimmed.
export type LensLine = {
	--- The line's text.
	text: string,
	--- OSC 8 hyperlinks the program printed on it.
	links: { LensLink },
}

--- Host only: a command lens, registered with `tern.lens.define`; `S` is its state.
export type LensDef<S> = {
	--- Starts a capture; a lens cannot decline (the manifest's `match` decided).
	open: (run: LensRun, cx: EffectCx) -> S,
	--- Feeds one output line.
	line: (state: S, l: LensLine) -> (),
	--- The command finished with exit `status` (always a number).
	finish: (state: S, status: number) -> (),
	--- The view; nil leaves the block showing raw output.
	view: (state: S) -> Node?,
	--- Handles an event from the view's nodes; `view` runs again after it.
	event: ((state: S, ev: UiEvent, cx: EffectCx) -> ())?,
}

--- Host only: a shell pane about to spawn, for the `spawn` hook.
export type SpawnSpec = {
	--- Program to run.
	program: string,
	--- Its arguments.
	args: { string },
	--- Working directory (always set; only the daemon expands a leading `~` in a returned one).
	cwd: string,
	--- Only the variables the daemon sets for the pane (`TERN_IDENTITY`, `TERN_PANE`, `TERN_PANE_SOCKET`,
	--- `TERN_WINDOW_KEY`, `TERN_WINDOW_SOCKET`, `TERN_LENSES`, `TERN_COMPLETE`, `TERN_BLOB_DIR`); the pane's base environment is the
	--- login environment, not the daemon's (`tern.getenv` here reads the daemon's).
	env: { [string]: string },
}

--- Native pane kinds; agent and plugin blocks are programs with terminal surfaces.
export type PaneKind = "terminal" | "agent" | "plugin" | "browser" | "file" | "git"
	| "database" | "notebook" | "board" | "procs" | "screen" | "profile"

--- A pane from host listing or window inspection; native blocks have no shell-state fields.
export type PaneInfo = {
	--- Pane id.
	pane: number,
	--- Working directory; in the window always a string (`""` when unknown), only `tern.pane.list` may give nil.
	cwd: string?,
	--- Window inspection: program only for terminal/agent; host listing always supplies it.
	program: string?,
	--- Pane title.
	title: string,
	--- Window terminal/agent only: whether the program reports work in progress.
	busy: boolean?,
	--- Window only: native block kind and containing tab.
	kind: PaneKind?,
	tab: number?,
	--- Window only: focus, floating picture and parked state (a parked pane has no tab).
	focused: boolean?,
	floating: boolean?,
	parked: boolean?,
	--- Window terminal/agent only: prompt/listening state.
	at_prompt: boolean?,
	--- Window terminal/agent only: current command and elapsed milliseconds.
	running: { line: string, ms: number }?,
	--- Window terminal/agent only: last completed command, status and duration in milliseconds.
	last: { line: string, status: number, took_ms: number }?,
	--- Window terminal/agent only: program exit status.
	exited: number?,
	--- Window only: startup error and unseen alert.
	error: string?,
	alert: boolean?,
	--- Window only: browser address, native file/root path and plugin-qualified block kind.
	url: string?,
	path: string?,
	block: string?,
}

--- Existing status-formatter input; backing program and busy remain populated for every kind.
export type StatusPaneInfo = PaneInfo & { program: string, busy: boolean }

--- Capped pane reads; absent options use the documented defaults.
export type ReadOpts = {
	--- Table (default) or compact text.
	as: ("table" | "text")?,
	--- Terminal/file lines: default 60, maximum 2000.
	lines: number?,
	--- Terminal history above the screen (default true); includes TSP surface text.
	scrollback: boolean?,
	--- First file line, one-based; default follows the view.
	from: number?,
	--- Native data records: default 25, maximum 500.
	rows: number?,
	--- Text characters: default 8000, maximum 64000.
	max_chars: number?,
}

--- A capped TSP or accessibility subtree; hidden blocks are never focused to read them.
export type SurfaceOpts = {
	as: ("table" | "text")?,
	--- TSP node id; omitted reads the main, dock and layer region roots.
	root: string?,
	--- Levels below the root (default all, maximum 64).
	depth: number?,
	--- Tree nodes: default 500, maximum 5000; omitted children carry more.
	nodes: number?,
	max_chars: number?,
}

--- TSP wire node, with an omitted-descendant count when capped.
export type TspNode = {
	id: string, k: string, p: { [string]: any }?, c: { TspNode }?, more: number?,
}

--- The top live surface, or newest retained inline surface, in TSP wire form.
export type SurfaceTree = {
	surface: { id: string, title: string?, role: string?, mode: "inline" | "screen" | "block" | "flow" },
	main: TspNode?, dock: TspNode?, layer: TspNode?,
	--- Descendants in whole regions omitted after the node budget was exhausted.
	more: number?,
}

--- Tern's exposed block UI subtree, without ids, bounds or actions.
export type ViewNode = {
	role: string, name: string?, value: (string | number)?, states: { string }?,
	children: { ViewNode }?, more: number?,
}

--- Shared read result, keyed by kind; unavailable model data is absent, not invented.
--- Terminal/agent: lines, more, surface. Plugin: title, text.
--- Browser: url, title, loading, progress, error, back, forward, driven.
--- File: path, host, file_kind, mode, dirty, load, first, total, lines, caret, selection,
--- size, image, hex. Git: root, ready, provisional, head, upstream, operation,
--- staged, unstaged, conflicts, selection, open, commits, stashes, running.
--- Database: path, memory, phase, read_only, dirty, area, schema, browse, execute.
--- Notebook: path, phase, kernel, dirty, active, cells (capped source and outputs,
--- without the golden state's 80-character source clip).
--- Board: path, phase, view, filter, hide_done, lanes (cards, tags, due, subtasks, cursor).
--- Procs: sampled, totals, sort, descending, tree, filter, rows, selected.
--- Profile: process, pid, take, samples, functions, focus, thread, query.
--- Screen: host, display, phase, failure, control, frame; pixels are not text.
export type BlockRead = { info: PaneInfo } & (
	{ kind: "terminal" | "agent", lines: { string }, more: number, surface: boolean }
	| { kind: "plugin", title: string, text: string }
	| {
		kind: "browser", url: string, title: string, loading: boolean, progress: number?,
		error: string?, back: boolean, forward: boolean, driven: boolean,
	}
	| {
		kind: "file", path: string?, host: string?, file_kind: string?, mode: string?,
		dirty: boolean?, load: any?, first: number?, total: number?, lines: { string }?,
		caret: { line: number, col: number }?, selection: any?, size: number?,
		image: { [string]: any }?, hex: { [string]: any }?, more: number?,
		chars_clipped: boolean?,
	}
	| {
		kind: "git", root: string?, ready: boolean?, provisional: boolean?,
		head: { branch: string?, oid: string?, detached: boolean }?,
		upstream: { name: string, ahead: number, behind: number }?, operation: string?,
		staged: { { path: string, change: string?, old_path: string? } }?,
		unstaged: { { path: string, change: string?, old_path: string? } }?,
		conflicts: { string }?, selection: { kind: string, ids: { string }? }?,
		open: { path: string, mode: string }?,
		commits: { {
			id: string, subject: string, author: string, time: any, subject_clipped: boolean?,
		} }?,
		stashes: number?, running: { string }?, first: number?, total: number?, more: number?,
		staged_more: number?, unstaged_more: number?, conflicts_more: number?, has_more: boolean?,
	}
	| {
		kind: "database", path: string?, memory: boolean?, phase: any?,
		read_only: boolean?, dirty: boolean?, area: string?,
		schema: { { name: string, kind: string, columns: {
			{ name: string, type: string, pk: boolean, notnull: boolean }
		} } }?,
		browse: { [string]: any }?, execute: { [string]: any }?,
		schema_more: number?,
	}
	| {
		kind: "notebook", path: string?, phase: any?, kernel: any?, dirty: boolean?,
		active: number?, cells: { {
			index: number, kind: string, count: number?, state: string, source: string,
			outputs: { { [string]: any } }, tags: { string },
			source_more: number?, source_chars_clipped: boolean?, outputs_more: number?,
		} }?, first: number?, total: number?, more: number?,
	}
	| {
		kind: "board", path: string?, phase: any?, view: string?, filter: string?,
		hide_done: boolean?, lanes: { {
			title: string, complete: boolean, folded: boolean, cards: { {
				id: number, text: string, done: boolean, tags: { string }, due: string?,
				text_clipped: boolean?,
				subtasks: { done: number, total: number }?, cursor: boolean,
			} }, more: number?,
		} }?, more: number?, first: number?, total: number?,
	}
	| {
		kind: "procs", sampled: boolean?, totals: { [string]: any }?, sort: string?,
		descending: boolean?, tree: boolean?, filter: string?, selected: { number }?,
		rows: { {
			pid: number, parent: number, name: string, user: string?, state: string,
			cpu: number, memory: number, threads: number, depth: number,
		} }?, first: number?, total: number?, more: number?,
	}
	| {
		kind: "profile", process: string?, pid: number?, take: any?, show: string?,
		samples: number?, functions: {
			{ name: string, self: number, total: number, name_clipped: boolean? }
		}?,
		focus: string?, thread: any?, query: string?, first: number?, total: number?, more: number?,
	}
	| {
		kind: "screen", host: string?, display: any?, phase: string?,
		failure: { kind: string, message: string }?,
		control: any?, frame: { w: number, h: number }?,
	}
)

--- A session lifecycle row; ids use the same exact Lua numbering as panes and tabs.
export type ManagedSession = {
	id: number, name: string, host: number, current: boolean,
	locked: boolean, note: string?, tabs: number,
}
--- Session ids, exact names, or "@current"; nil also means the current session.
export type SessionSpec = number | string
--- Window only: named session lifecycle; operations reveal their results.
declare extern type SessionsCx with
	--- Lists every session on every attached host, without switching.
	function list(self): { ManagedSession }
	--- Returns the current session's metadata.
	function current(self): ManagedSession
	--- Creates and switches to a session on the current host; cwd sets its first tab directory.
	function create(self, opts: { name: string?, cwd: string? }?): ManagedSession
	--- Shows a session by id, exact name or "@current"; omission means current.
	function switch(self, session: SessionSpec?): ()
	--- Renames the specified session; blank names error.
	function rename(self, session: SessionSpec?, name: string): ()
	--- Ends the session's programs; the only remaining session stays open and empty.
	--- Raw close ignores interactive confirmation and locks, like cx.layout:close.
	function close(self, session: SessionSpec?): ()
	--- Locks a session against interactive closing; nil selects the current session.
	function lock(self, session: SessionSpec?): ()
	--- Removes the interactive-close lock; nil selects the current session.
	function unlock(self, session: SessionSpec?): ()
end
--- Host slots, case-insensitive names, exact addresses, "local" or "@current".
export type HostSpec = number | string
--- A machine's real connection and identity metadata, with its sessions and panes.
export type ManagedHost = {
	id: number, name: string, kind: "local" | "remote", address: string?,
	state: "connecting" | "connected" | "stalled" | "verify" | "reconnecting" | "failed",
	detail: { fingerprint: string?, attempt: number?, error: string? }?,
	identity: { [string]: any }?, current: boolean, rtt_ms: number?,
	sessions: { number }, panes: { number },
}
--- Window only: attached machines and terminals, using the ordinary remote lifecycle.
declare extern type HostsCx with
	--- Lists local and remote hosts, connection states, identity chains, RTT and membership.
	function list(self): { ManagedHost }
	--- Returns the current session's host.
	function current(self): ManagedHost
	--- Desktop: connects to user@host:port (user/port optional); await or use :next in plugins.
	--- Resolves with the connected host, fails on connection failure/disconnect.
	--- First-contact key verification stays pending: inspect detail.fingerprint and verify it.
	function add(self, address: string): Awaitable<ManagedHost>
	--- Shows the connected host's most recently shown session (else its first).
	function switch(self, host: HostSpec?): ()
	--- Pins an independently verified first-contact fingerprint; errors if it does not match.
	function trust(self, host: HostSpec, fingerprint: string): ()
	--- Disconnects and forgets a remote host; programs keep running there; local errors.
	function disconnect(self, host: HostSpec): ()
	--- Opens and focuses a shell tab on a connected host; command runs in its login shell.
	--- Returns the pane id, usable with cx.session:read, cx:run and cx.session:settle.
	function terminal(self, host: HostSpec?, opts: { cwd: string?, command: string? }?): number
end

--- Browser automation's answer envelope, identical to tern browser.
export type BrowserResult = {
	ok: { [string]: any }?,
	error: { kind: string, message: string }?,
}

--- An agent block in any session; unknown optional fields are tern.json.null.
export type AgentBlock = {
	pane: number, command: string?, model: string?, cwd: string,
	state: "idle" | "working" | "waiting_input" | "exited", last_message: string?,
	--- Why a queued prompt could not be delivered (for example, omp needs updating).
	delivery_error: string?,
}

--- A chat message; tools retain the TSP call's name, target, status, metadata and text.
export type AgentMessage = {
	role: "user" | "assistant", text: string, tools: { { [string]: any } },
}

--- Window only: agent conversations; targets are pane ids or session resolve specs.
declare extern type AgentsCx with
	--- Synchronously lists agent blocks across all sessions, with progress/input state and last assistant text.
	function list(self): { AgentBlock }
	--- Synchronously opens an agent; the window retains prompt until its composer reports sendable=true.
	--- Omitted command/how use agent settings; queued prompts survive a Carly/plugin VM reset.
	--- cwd "~"/"~/…" expands using the launch host's home. Native omp requires TSP send.
	function start(self, opts: {
		prompt: string, cwd: string, command: string?,
		how: ("split" | "tab" | "replace_if_alone")?,
	}): number
	--- Synchronously queues/submits one prompt; native omp uses one atomic TSP send event.
	--- Native prompts queue until sendable=true, even if already editable/focused; bootstrap false is not an error.
	--- Blank/concurrent pending prompts or unsupported native omp error.
	function ask(self, agent: number | string, text: string): ()
	--- Desktop async: resolves with last assistant text when idle; timeout is ms (default 30000).
	--- Queued/newly submitted prompts are not idle; delivery errors reject the awaitable.
	--- Carly uses await(...); plugins use :next(function(message, err, cx) ... end).
	function wait(self, agent: number | string, opts: { timeout: number? }?): Awaitable<string?>
	--- Last messages in chronological order (default 50, max 1000); last=0 returns none.
	--- Reads the omp.session TSP chat, excluding chrome/thinking; non-omp agents return {}.
	function transcript(self, agent: number | string, opts: { last: number? }?): { AgentMessage }
	--- Sends Ctrl-C to interrupt the current turn without closing its block.
	function interrupt(self, agent: number | string): ()
	--- Closes the agent block and ends its program, without an interactive confirmation.
	function stop(self, agent: number | string): ()
end

--- A palette action, including unavailable commands and plugin registrations.
export type PaletteAction = {
	id: string, label: string, group: string, keys: { string },
	available: boolean, plugin: string?,
}

--- A sampled local process; unknown OS details are nil and CPU is percent of one core.
export type ProcessInfo = {
	pid: number, parent: number?, name: string, executable: string?, cwd: string?,
	command: { string }, user: string?, state: string, cpu: number, memory: number,
	gpu: number?, gpu_memory: number?, threads: number?, started: number,
	disk_read: number, disk_write: number, depth: number?, context: boolean?,
}
--- Process query: all text filters are case-insensitive substrings and combine with AND.
export type ProcessQuery = {
	sort: ("name" | "pid" | "user" | "cpu" | "memory" | "gpu" | "threads" | "disk")?,
	descending: boolean?, name: string?, user: string?, cwd: string?,
	limit: number?, tree: boolean?,
}
--- A capped inventory; tree results include ancestors marked context=true.
export type ProcessList = {
	["local"]: boolean, sample: number, total: number, more: number, processes: { ProcessInfo },
}
--- Local process signals; no confirmation or permission policy is added.
export type ProcessSignal = "terminate" | "kill" | "interrupt" | "hangup" | "stop" | "continue"
--- A finished local profile; function counts are sampled stacks, not elapsed times.
export type ProcessProfile = {
	pane: number, pid: number, process: string, show: string, samples: number,
	functions: { { name: string, ["self"]: number, total: number, name_clipped: boolean } },
	first: number, total: number, more: number,
}

--- Desktop window only: the local machine, even while viewing a remote host.
declare extern type ProcsCx with
	--- Await a sampled inventory; default sort CPU descending, limit 100 (1..5000), tree false.
	--- Names/PIDs/users sort ascending by default; amounts descend. Tree keeps matching ancestors.
	--- Cwd is nil if the OS cannot read it. Ports/port filtering are unavailable in the sampler.
	function list(self, options: ProcessQuery?): Awaitable<ProcessList>
	--- Await the busiest name substring match or exact positive PID; nil when not found.
	function find(self, name_or_pid: string | number): Awaitable<ProcessInfo?>
	--- Await immediate children only (not all descendants), capped at 5000.
	function children(self, pid: number): Awaitable<ProcessList>
	--- Send a signal to one positive local PID; OS/validation errors raise with kind/message/pid/signal.
	function signal(self, pid: number, signal: ProcessSignal): { pid: number, signal: string, signaled: boolean }
	--- Open and focus a Processes task-manager block beside the focused pane, or in a new tab if empty.
	function open(self): number
	--- Record a local PID and open a profile block beside focus; await the top 25 self-heavy functions.
	--- seconds defaults to 20 (1..3600). Unsupported profiling/OS errors fail the awaitable.
	--- Plugins use :next(function(result, err, cx) ... end); Carly uses await(cx.procs:profile(pid, opts)).
	function profile(self, pid: number, options: { seconds: number? }?): Awaitable<ProcessProfile>
end

--- Window only: command palette discovery and ordinary action dispatch.
declare extern type ActionsCx with
	--- Lists all actions; optional query matches every word against id, label and group.
	function list(self, query: string?): { PaletteAction }
	--- Runs an id or unique exact label; errors on unknown or ambiguous names.
	function run(self, id_or_label: string): string
	--- Returns effective chord -> action ids in focus-resolution priority order.
	function keys(self): { [string]: { string } }
	--- Describes an exact action id, including current availability and key chords.
	function describe(self, id: string): PaletteAction
end

--- A local repository; pass this descriptor, its root, a pane id or a resolve spec to git methods.
export type GitRepo = {
	root: string, branch: string?, head: string?, upstream: string?,
	ahead: number, behind: number, operation: string?,
}
--- Either side of a changed path; rename sources and conflict kinds are preserved.
export type GitFile = {
	path: string, old_path: string?, staged: string?, unstaged: string?,
	staged_stat: { number }?, unstaged_stat: { number }?, conflict: string?,
}
--- Changed paths grouped by index, worktree and unmerged state; a path can occur on both sides.
export type GitStatus = {
	staged: { GitFile }, unstaged: { GitFile }, conflicts: { GitFile }, operation: string?,
}
--- Repository input: descriptor from repo(), path, local pane id, or session:resolve-style spec.
export type GitTarget = GitRepo | string | number
--- Desktop window only: git block reads and mutations; await in Carly, :next(fn) in plugins.
declare extern type GitCx with
	--- Finds a repository from path or local pane/spec (e.g. "@focused"), reporting branch/upstream/state.
	function repo(self, path_or_pane: GitTarget): Awaitable<GitRepo>
	--- Reads fresh staged, unstaged and conflict paths, including rename sources and per-side line counts.
	function status(self, repo: GitTarget): Awaitable<GitStatus>
	--- Reads graph history across refs (default 50, maximum 100000); path follows HEAD file renames.
	function log(self, repo: GitTarget, opts: { limit: number?, path: string? }?): Awaitable<{ any }>
	--- Returns unified patch text: default unstaged, staged=true for index, rev for one commit's patch.
	--- path is a literal repo-relative filename; staged and rev cannot be combined.
	function diff(self, repo: GitTarget, opts: { path: string?, staged: boolean?, rev: string? }?): Awaitable<string>
	--- Stages literal repo-relative paths (including untracked/deleted files); empty paths changes nothing.
	function stage(self, repo: GitTarget, paths: { string }): Awaitable<string>
	--- Unstages literal paths without changing worktree files; include both paths of a rename.
	function unstage(self, repo: GitTarget, paths: { string }): Awaitable<string>
	--- Commits staged changes with a full message; amend=true rewrites HEAD, not the worktree.
	function commit(self, repo: GitTarget, message: string, opts: { amend: boolean? }?): Awaitable<string>
	--- Lists local and remote branches with name/target/remote/upstream/ahead/behind/current/gone fields.
	function branches(self, repo: GitTarget): Awaitable<{ any }>
	--- Checks out a local/remote branch; detached=true checks out a revision without moving a branch.
	function checkout(self, repo: GitTarget, name: string, opts: { detached: boolean? }?): Awaitable<string>
	--- Creates and checks out a branch, at HEAD unless at names another revision.
	function create_branch(self, repo: GitTarget, name: string, opts: { at: string? }?): Awaitable<string>
	--- Stashes tracked and untracked changes, with an optional message.
	function stash_push(self, repo: GitTarget, opts: { message: string? }?): Awaitable<string>
	--- Applies and drops the stash at zero-based index (default newest, 0); conflicts fail normally.
	function stash_pop(self, repo: GitTarget, opts: { index: number? }?): Awaitable<string>
	--- Opens a git block (default focuses one in the tab); how uses native block placement.
	--- All mutations use the block's shared worker, undo journal and refresh; failures reject awaitables.
	function open(self, repo: GitTarget, opts: { how: How? }?): Awaitable<number>
end

--- Description derived from the real preference's serde type, defaults and Rust docs.
export type SettingDescription = {
	type: string, enum: {string}?, range: {min: number, max: number}?,
	default: any, nullable: boolean, doc: string,
}

--- Window only: the whole persisted preferences tree; headless changes remain ephemeral.
declare extern type SettingsCx with
	--- Reads a dotted preference or subtree: get("carly.model"), get("sqlite.drop").
	function get(self, key: string): any
	--- Validates and applies one preference; returns its effective (clamped) value.
	--- Root keys match settings.json; terminal.font_size also aliases font_size. nil clears options.
	function set(self, key: string, value: any): any
	--- Atomically validates a dotted-key table or nested patch; returns effective preferences.
	--- Example: set_many({["carly.model"] = "@smol", notebook = {start_on_open = true}}).
	--- Unknown/type-invalid keys and overlapping paths fail before any changes.
	function set_many(self, values: {[string]: any}): {[string]: any}
	--- Lists every leaf's dotted key and current value; prefix filters without hiding any domain.
	--- Unset optional values stay present as tern.json.null; get(key) returns nil instead.
	function list(self, prefix: string?): {[string]: any}
	--- Describes type, enum spellings, clamp range, default and existing preference documentation.
	function describe(self, key: string): SettingDescription
	--- Lists installed theme names; set theme_dark/theme_light to pin, nil to follow omp.
	function themes(self): {string}
	--- Lists the user's overrides (effective bindings are cx.actions:keys()).
	function keybinds(self): {[string]: string | {string}}
	--- Binds a shortcut or sequence to an action id or priority-ordered action ids.
	function bind(self, chord: string, actions: string | {string}): ()
	--- Leaves this shortcut to the program by writing an empty settings.keybinds override.
	--- To restore defaults instead, replace keybinds with a table omitting that shortcut.
	function unbind(self, chord: string): ()
end

--- An opaque VM-local connection id returned by cx.db:open; not a pane id.
export type Database = number

--- Exact SQL cells: NULL is tern.json.null, blobs are bytes, large integers are decimal text.
--- Nonfinite reals use {real = "inf" | "-inf"}; finite values are ordinary numbers.
export type DbCell = number | string | JsonNull
	| { integer: string } | { real: string } | { blob: { number } }

--- Positional query rows, aligned with columns even when cells are SQL NULL.
export type DbResult = {
	--- Result column names, in order.
	columns: { string },
	--- Returned rows, capped by query's limit.
	rows: { { DbCell } },
}

--- Options for opening an existing local SQLite database.
export type DbOpenOptions = {
	--- Defaults true; only explicit false permits exec, using a read-write SQLite connection.
	read_only: boolean?,
}

--- Options for a read-only row-producing query.
export type DbQueryOptions = {
	--- Maximum rows returned, default 1000; zero returns only column names.
	limit: number?,
}

--- Desktop window only: VM-owned SQLite connections, all work off the UI thread.
declare extern type DbCx with
	--- Opens an existing local file; relative paths use focused pane cwd (else home). Default read-only.
	function open(self, path: string, opts: DbOpenOptions?): Awaitable<Database>
	--- Lists main database's user tables and views by name, excluding SQLite internal objects.
	function tables(self, db: Database): Awaitable<{ string }>
	--- Returns a table or view's stored CREATE SQL; unknown names fail the awaitable.
	function schema(self, db: Database, table: string): Awaitable<string>
	--- Executes one row-producing read-only statement; writes are refused even on writable handles.
	function query(self, db: Database, sql: string, opts: DbQueryOptions?): Awaitable<DbResult>
	--- Runs SQL statements only for explicitly writable opens; SQLite failures fail the awaitable.
	--- Writes autocommit unless SQL explicitly begins a transaction; success returns tern.json.null.
	function exec(self, db: Database, sql: string): Awaitable<JsonNull>
end

--- A local Markdown path or the numeric id of an open board pane.
export type BoardSpec = string | number
--- Opaque model identity, not a row number; stable across edits, invalid after reload/reopen.
export type BoardId = string
--- New-card fields; subtasks can be strings or records. Omitted metadata is retained by edit.
export type BoardCardEdit = { text: string?, tags: {string}?, due: (string | false)? }
export type BoardNewCard = {
	lane: string, text: string, tags: {string}?, due: (string | false)?,
	subtasks: {string | {text: string, done: boolean?}}?,
}
--- Bounded common block-reader data, with addressable card and lane ids.
export type BoardSnapshot = {
	path: string, phase: string, open: number, done: number, first: number,
	total: number, more: number,
	lanes: {{id: BoardId, title: string?, complete: boolean, more: number,
		cards: {{id: BoardId, text: string, done: boolean, tags: {string}, due: string?}}}},
}
--- Desktop Markdown task boards; all except open return awaitables.
--- Carly uses await(...); plugins use :next(function(result, err, cx) ... end).
declare extern type BoardCx with
	--- Opens or reveals a board; relative paths use focused local cwd; returns its pane.
	function open(self, path: string, opts: {how: "default" | "beside" | "below" | "tab" | "pip" | "preview"}?): number
	--- Open boards plus task/Kanban Markdown files at local pane cwds and checkout roots.
	function boards(self): Awaitable<{{path: string, pane: number?, open: boolean}}>
	--- Reads the live model when open, disk otherwise; rows defaults 25 (max 500), from is a lane.
	--- Card/lane ids stay valid through API edits; external reloads or reopening invalidate them.
	function read(self, board: BoardSpec, opts: {rows: number?, from: number?, max_chars: number?}?): Awaitable<BoardSnapshot>
	--- Adds to a lane by exact title or id; tags omit #, due is YYYY-MM-DD, subtasks are optional.
	function add(self, board: BoardSpec, card: BoardNewCard): Awaitable<BoardId>
	--- Moves the card id from read/add; position is one-based without the card, default append.
	--- Moving into Done checks the card; moving out reopens it.
	function move(self, card: BoardId, lane: string, opts: {position: number?}?): Awaitable<boolean>
	--- Checks/reopens the card and follows the engine's done-lane movement.
	function check(self, card: BoardId, done: boolean): Awaitable<boolean>
	--- Replaces supplied fields only; tags={} clears tags, due=false clears due; keeps notes.
	function edit(self, card: BoardId, fields: BoardCardEdit): Awaitable<boolean>
	--- Removes the card with its notes and subtasks; mounted edits share normal Undo/autosave.
	function remove(self, card: BoardId): Awaitable<boolean>
	--- Inserts a titled lane; position is one-based, default append; returns its stable id.
	function add_lane(self, board: BoardSpec, opts: {title: string, position: number?}): Awaitable<BoardId>
	--- Renames a lane addressed by exact title or id; ambiguous titles require ids.
	function rename_lane(self, board: BoardSpec, lane: string, title: string): Awaitable<boolean>
	--- Removes a lane and all its cards; closed-file changes write atomically.
	function remove_lane(self, board: BoardSpec, lane: string): Awaitable<boolean>
end

--- Window only: shared browser automation, including snapshot and ref-based act.
declare extern type BrowserCx with
	--- Runs a tern browser op; await in Carly, :next(function(result, err, cx) ... end) in plugins.
	--- Carly and plugins share the same browser operations; snapshot/state/blocks/events are reads.
	--- Browser failures resolve with result.error; snapshot never enables Tern's driver/dialog policy.
	function call(self, op: { [string]: any }): Awaitable<BrowserResult>
end

--- A display on this machine; its stable id can be passed to share.
export type ScreenDisplay = {
	id: string, name: string, width: number, height: number, scale: number, primary: boolean,
}
--- Window only: discover displays and share them in screen panes.
declare extern type ScreenCx with
	--- Lists local display ids/names/pixel sizes; await in Carly or use :next in plugins.
	function displays(self): Awaitable<{ScreenDisplay}>
	--- Shares display id (nil or "main" picks primary); how is "view" (default) or "control".
	function share(self, display: string?, opts: {how: "view" | "control"}?): number
	--- Stops sharing by closing the screen pane; accepts an id or session resolve spec.
	function stop(self, pane: number | string): ()
	--- Returns phase, host, display, control, frame and failure for a screen pane.
	function status(self, pane: number | string): {[string]: any}
end

--- Window only: a picture-in-picture corner.
export type Corner = "tl" | "tr" | "bl" | "br"
--- Window only: a tab menu color label.
export type TabColor = "red" | "orange" | "yellow" | "green" | "teal" | "blue" | "purple" | "pink"
--- Reveal a result (default true); plugins and Carly share the same defaults.
export type Reveal = { focus: boolean? }
--- Window only: a named session and its ordered tabs.
export type SessionInfo = { id: number, name: string, current: boolean, locked: boolean, tabs: { number } }
--- A split tree: pane at a leaf; [1] left/top and [2] right/bottom at a split.
export type TabTree = { pane: number?, split: ("right" | "down")?, ratio: number?, [number]: TabTree }
--- A floating block and its tiled owner.
export type Float = { pane: number, over: number, corner: Corner }
--- A tab's current tree, floats, zoom and focused block.
export type TabLayout = { tab: number, root: TabTree, floats: { Float }, zoomed: boolean, focus: number }
--- Every tab's arrangement and parked panes, not programs; restore refuses after blocks open or close.
declare extern type LayoutSnapshot with end
--- A regrouping target; blocks may be ids or resolve specs (parked blocks are dealt into it), empty
--- means rename-only.
export type TabGroup = {
	tab: (number | string)?,
	name: string?,
	color: TabColor?,
	blocks: { number | string },
	split: ("columns" | "rows" | "grid")?,
}
--- Regroups one session; unmentioned blocks stay put, or move to one final Other tab.
export type Arrange = { session: (number | string)?, tabs: { TabGroup }, rest: ("keep" | "last")? }
--- A resulting tab, in one-based session order.
export type TabArrangement = { id: number, position: number, name: string, blocks: { number } }

--- A tab, as `tern.chrome.tab_title` and `cx.session:tabs()` see it.
export type TabInfo = {
	--- Tab id.
	id: number,
	--- Its session, color, one-based position and focused block (also formatter cache keys).
	session: number,
	color: TabColor?,
	position: number,
	pane: number,
	--- Name the user gave it.
	name: string?,
	--- Focused pane's working directory (`""` when unknown).
	cwd: string?,
	--- Focused pane's program.
	program: string,
	--- Focused pane's title.
	title: string,
	--- Number of panes.
	panes: number,
	--- Whether a command is running in it.
	busy: boolean,
}

--- Window only: the window being titled, as a table of its state.
export type WindowInfo = { [string]: any }

--- Window only: a segment of the status line.
export type StatusSegment = {
	--- Its text.
	text: string,
	--- Named icon.
	icon: string?,
	--- Tone (`"muted"`, `"error"`, …).
	tone: string?,
	--- Action name run on click (`"new_tab"`, `"plugin.<id>.<cmd>"`).
	command: string?,
}

--- Window only: a palette command for `tern.command`.
export type CommandDef = {
	--- Id; the action name is `plugin.<plugin>.<id>`.
	id: string,
	--- Palette label.
	title: string,
	--- Named icon.
	icon: string?,
	--- Palette group (default: the plugin name).
	group: string?,
	--- Default chords (`"cmd+shift+h"`, `"ctrl+a>c"`).
	keys: { string }?,
	--- Whether the row shows now (4 ms budget).
	available: ((cx: WindowCx) -> boolean)?,
	--- Runs the command.
	run: (cx: WindowCx) -> (),
}

--- Window only: a file open that `tern.route.open` may claim.
export type OpenRequest = {
	--- Absolute path.
	path: string,
	--- 1-based line to reveal.
	line: number?,
	--- 1-based column to reveal.
	col: number?,
	--- Host the file lives on.
	host: string,
	--- Where it opens.
	how: How,
	--- What asked for it.
	origin: Origin,
}

--- Window only: a link click that `tern.route.link` may claim.
export type LinkRequest = {
	--- The URL.
	url: string,
	--- Pane the link is in.
	pane: number,
	--- Modifiers held: `cmd`, `alt`, `shift`, `ctrl`.
	mods: { cmd: boolean, alt: boolean, shift: boolean, ctrl: boolean },
}

--- Window only: a route's answer, with one of `block`, `url`, `path` or `handled` set; returning nil runs the built-in.
export type RouteDecision = {
	--- Opens a plugin block of this kind (`"plugin.kind"`) instead, on the file's host (`route.open`) or the clicked
	--- pane's (`route.link`); when no Ready plugin there defines it, an error toast names the kind and host.
	block: string?,
	--- Launch arguments of that block.
	args: { string }?,
	--- Opens this URL instead.
	url: string?,
	--- Rewrites the path, then runs the built-in open.
	path: string?,
	--- Where the rewritten path opens.
	how: How?,
	--- True when the plugin handled it; nothing else runs.
	handled: boolean?,
}

--- Window only: what a new pane runs (default: a shell).
export type Launch = {
	--- Working directory.
	cwd: string?,
	--- Shell command line to run.
	command: string?,
	--- Plugin block kind (`"plugin.kind"`).
	block: string?,
	--- Block launch arguments.
	args: { string }?,
}

--- Window only: a workspace tab for `cx.layout:tab`; `[1]` and `[2]` are the two sides of `split`.
export type TabSpec = {
	--- Working directory.
	cwd: string?,
	--- Tab name.
	name: string?,
	--- Direction of the split between `[1]` and `[2]`.
	split: Dir?,
	--- The first subtree's share (default one half).
	ratio: number?,
	--- What this leaf runs.
	launch: Launch?,
	--- `[1]` and `[2]`: the sides of `split`.
	[number]: TabSpec,
}

--- `command_started` payload: a shell began running a command line.
export type CommandStartedEvent = {
	--- Pane id.
	pane: number,
	--- The command line as the shell's OSC 133;C reports it (typed, not alias-expanded).
	line: string,
	--- Host only: the shell's working directory, when known.
	cwd: string?,
}

--- `command_finished` payload: the command line ended (fires only when its start was seen).
export type CommandFinishedEvent = {
	--- Pane id.
	pane: number,
	--- The command line.
	line: string,
	--- Its exit status.
	status: number,
	--- How long it ran, in milliseconds.
	took_ms: number,
}

--- `cwd` payload: a pane's working directory changed.
export type CwdEvent = {
	--- Pane id.
	pane: number,
	--- The new directory.
	path: string,
}

--- `title` payload: a pane's title changed.
export type TitleEvent = {
	--- Pane id.
	pane: number,
	--- The new title.
	title: string,
}

--- Host only: `pane_exited` payload: a pane's program exited.
export type PaneExitedEvent = {
	--- Pane id.
	pane: number,
	--- Exit status.
	status: number,
}

--- Window only: `focus` payload: the focused pane changed.
export type FocusEvent = {
	--- Focused pane id.
	pane: number,
	--- Its tab id.
	tab: number,
}

--- Window only: `pane_created` / `pane_closed` payload.
export type PaneEvent = {
	--- Pane id.
	pane: number,
}

--- Window only: `tab_created` / `tab_closed` payload.
export type TabEvent = {
	--- Tab id.
	tab: number,
}

--- `tern.json.null`: stands for JSON null in tables (Lua nil can't).
declare extern type JsonNull with end

--- A pending `tern.timer`.
declare extern type TimerHandle with
	--- Cancels the timer; no effect once it fired.
	function cancel(self): ()
end

--- Host only: the `cx` of lenses and host hooks; effects go to every window of the pane's window key.
declare extern type EffectCx with
	--- Shows a toast; `sub` is a secondary line.
	function toast(self, level: ToastLevel, text: string, sub: string?): ()
	--- Opens a file path or URL through the window's `route.link`, then `route.open` with origin `link`.
	function open(self, target: string): ()
	--- Copies `text` to the clipboard.
	function copy(self, text: string): ()
end

--- Host only: the `cx` of a plugin block.
declare extern type BlockCx extends EffectCx with
	--- Pane id.
	pane: number
	--- Working directory, when known.
	cwd: string?
	--- Width in cells.
	cols: number
	--- Height in cells.
	rows: number
	--- Re-renders the view (after changes from timers or processes).
	function render(self): ()
	--- Runs `save` and persists the result now.
	function save(self): ()
	--- Ends the block with exit `code`.
	function exit(self, code: number): ()
	--- Sends raw TSP frame ops.
	function frame(self, ops: { any }): ()
	--- Stores `bytes` of type `mime` (`"image/png"`); returns the blob id for an `image` node's `blob` prop.
	function blob(self, bytes: string, mime: string): string
end

--- A persistent window-authored canvas; view and md are mutually exclusive.
export type CanvasSpec = {
	--- Header title (80 characters maximum).
	title: string,
	--- Drawn root node; input, editor and image kinds are not allowed.
	view: Node?,
	--- Markdown alternative to view.
	md: string?,
	--- Optional bottom bar.
	dock: Node?,
	--- Purpose and authorized actions (2000 characters maximum).
	about: string?,
	--- Placement, default beside.
	how: How?,
	--- Focuses the new pane by default; false preserves shown focus.
	focus: boolean?,
}

--- An owner-only canvas update; paths refer to keys within the supplied view.
export type CanvasChange = {
	--- New header title.
	title: string?,
	--- New purpose and authorized actions.
	about: string?,
	--- Replaces the drawn view.
	view: Node?,
	--- Markdown alternative to view.
	md: string?,
	--- Replaces the bottom bar.
	dock: Node?,
	--- Replaces keyed nodes; false removes; new keys append under an existing parent.
	set: { [string]: Node | false }?,
}

--- Canvas metadata; list omits the potentially large view and dock.
export type CanvasInfo = {
	--- Pane id.
	pane: number,
	--- Header title.
	title: string,
	--- Purpose and authorized actions.
	about: string,
	--- Creating plugin id, or carly.
	owner: string,
	--- Current drawn root.
	view: Node?,
	--- Current bottom bar.
	dock: Node?,
}

--- A canvas click delivered only to the owner's handlers in this window.
export type CanvasAction = {
	--- Canvas pane.
	pane: number,
	--- Named action.
	act: string,
	--- Clicked node's visible text, quoted as data in Carly's fallback turn.
	text: string?,
	--- Canvas title.
	title: string?,
	--- Purpose and authorized actions.
	about: string?,
	--- Clicked node id.
	node: string,
	--- Optional event value.
	value: string?,
	--- Held modifiers.
	mods: { string }?,
	--- Creating owner.
	owner: string,
}

--- A native or Ready plugin block available on the current host.
export type BlockTypeInfo = {
	--- Fully qualified block kind.
	kind: string,
	--- Display title.
	title: string,
	--- Named icon.
	icon: string,
	--- Whether the palette offers creation.
	palette: boolean,
}

--- A one-based inclusive span of lines touched by a document edit.
export type DocRange = { from: number, to: number }
--- A parsed Markdown heading (ATX or setext; fenced code is excluded).
export type DocHeading = { title: string, level: number, line: number }
--- A case-sensitive literal match; columns count Unicode characters, starting at one.
export type DocMatch = { line: number, column: number, text: string }
--- Exactly one editing form: literal replace, insert before a line, or append to a section.
export type DocEdit = { find: string, replace: string, all: boolean? }
	| { line: number, insert: string } | { heading: string, append: string }

--- Desktop text/Markdown documents; paths resolve like file opens, relative to focused cwd.
--- Async methods use the live buffer when open (undo/dirty/autosave), atomic disk writes otherwise.
declare extern type DocsCx with
	--- Opens a document and returns its pane; line is one-based; how defaults to default.
	function open(self, path: string, opts: { how: How?, line: number? }?): number
	--- Reads exact text including unsaved edits and line endings; from defaults to 1, lines to all.
	function read(self, path: string, opts: { from: number?, lines: number? }?): Awaitable<string>
	--- Parses Markdown headings with visible titles, levels and one-based source lines.
	function outline(self, path: string): Awaitable<{DocHeading}>
	--- Replaces the whole text; creates a missing disk file (parent directory must exist).
	function write(self, path: string, text: string): Awaitable<DocRange>
	--- Replaces first/all literal matches, inserts before line, or appends before next same/higher heading.
	--- Heading is its visible title; missing/duplicate headings and missing find text fail without edits.
	function edit(self, path: string, spec: DocEdit): Awaitable<DocRange>
	--- Appends exact text at end; returns the affected inclusive line range.
	function append(self, path: string, text: string): Awaitable<DocRange>
	--- Saves an open buffer and waits for confirmation; closed existing files need no save.
	function save(self, path: string): Awaitable<boolean>
	--- Creates a session-owned note seeded with '# title', then markdown; returns its pane.
	function new_note(self, title: string, markdown: string): number
	--- Finds case-sensitive literal matches in the live document, with line, column and line text.
	function search(self, path: string, query: string): Awaitable<{DocMatch}>
end

--- Persistent native canvases, capped at 256 KiB per update, 2000 nodes and depth 32.
declare extern type CanvasCx with
	--- Opens an owner-tagged canvas; unavailable hosts return nil and an error.
	function open(self, spec: CanvasSpec): (number?, string?)
	--- Validates and updates an owned canvas.
	function set(self, pane: number, change: CanvasChange): ()
	--- Reads any owner's canvas from this window's replica.
	function get(self, pane: number): CanvasInfo?
	--- Lists canvas metadata without view or dock.
	function list(self): { CanvasInfo }
end

--- Window only: `cx.session`, the window's panes and tabs.
declare extern type SessionCx with
	--- Every named session and its ordered tab ids.
	function sessions(self): { SessionInfo }
	--- Split tree, floats, zoom and focus; nil for an unknown tab.
	function layout(self, tab: number): TabLayout?
	--- A one-based position or title words, within session (default current).
	function find_tab(self, spec: string, session: number?): number?
	--- Session by id, exact name, then name ignoring case.
	function find_session(self, spec: string): number?
	--- The tab holding a tiled or floating block.
	function tab_of(self, pane: number): number?
	--- One pane by id or resolve spec, nil when absent from this window.
	function pane(self, pane: number | string): PaneInfo?
	--- What a pane shows, by kind; surfaces included. No focus or browser eval.
	function read(self, pane: number | string, opts: ReadOpts?): (BlockRead | string)?
	--- A capped TSP tree for the live surface or newest retained surface.
	function surface(self, pane: number | string, opts: SurfaceOpts?): (SurfaceTree | string)?
	--- Tern's block UI accessibility subtree; nil when hidden or not mounted.
	function view(self, pane: number | string, opts: SurfaceOpts?): (ViewNode | string)?
	--- Sends a TSP event such as {ev = "action", id = "main.button", act = "click"}; a canvas takes
	--- only action events and delivers them to its owner as clicks once the caller returns.
	function event(self, pane: number | string, ev: { [string]: any }): ()
	--- Every pane, parked ones included.
	function panes(self): { PaneInfo }
	--- Every tab.
	function tabs(self): { TabInfo }
	--- The focused pane id.
	function focused(self): number?
	--- Pane id for `spec`: an id, `"@focused"`, a title or a program.
	function resolve(self, spec: string): number?
	--- Native and Ready plugin block types on this window's current host.
	function block_types(self): { BlockTypeInfo }
	--- Desktop: waits for the pane's shell command to finish (OSC 133), or the deadline; returns its output text.
	--- Defaults to 30000 ms, capped at 120000; a closed pane fails. Plugins use :next; Carly uses await.
	function settle(self, pane: number, ms: number?): Awaitable<string>
end

--- Window only: `cx.layout`, pane and tab arrangement.
declare extern type LayoutCx with
	--- Opens a tab running launch and reveals it by default.
	function new_tab(self, launch: Launch?, reveal: Reveal?): number?
	--- Splits target; the default gives its tab's focus to the new pane without switching tabs.
	function split(self, target: number, dir: Dir, launch: Launch?, reveal: Reveal?): number?
	--- Focuses a pane in this window.
	function focus(self, pane: number): ()
	--- Closes a pane through the raw session operation; no confirmation guard and no return.
	function close(self, pane: number): ()
	--- Move beside target or swap; reveals by default, or preserves shown focus for focus=false.
	function move(self, pane: number, target: number, dir: Dir | "swap", reveal: Reveal?): boolean
	--- Grows (or, negative, shrinks) pane toward dir by cells; false when no divider moved.
	function resize(self, pane: number, dir: Dir, cells: number): boolean
	--- Builds and names a workspace tree; returns tab and any launch failure.
	function tab(self, spec: TabSpec, reveal: Reveal?): (number?, string?)
	--- Moves pane beside the destination tab's focused leaf.
	function move_to_tab(self, pane: number, tab: number, reveal: Reveal?): (boolean, string?)
	--- Moves pane to a new last tab (default its own session), returning the tab.
	function move_to_new_tab(self, pane: number, session: number?, reveal: Reveal?): (number?, string?)
	--- Floats pane over a leaf; defaults to its old-area owner and nearest corner.
	function float(self, pane: number, over: number?, corner: Corner?, reveal: Reveal?): (boolean, string?)
	--- Docks a floating block; defaults to its original split, else beside its owner.
	function dock(self, pane: number, target: number?, dir: Dir?, reveal: Reveal?): (boolean, string?)
	--- Parks pane in its session's deck, its program still running; its split collapses, or its tab
	--- closes when it was the only pane.
	function park(self, pane: number): (boolean, string?)
	--- Deals a parked pane into the current tab beside its focused block; reveals by default.
	function unpark(self, pane: number, reveal: Reveal?): (boolean, string?)
	--- Gives the tab's tiled blocks equal shares.
	function even(self, tab: number): ()
	--- Names a tab; nil or blank restores its pane's label.
	function name_tab(self, tab: number, name: string?): ()
	--- Sets a tab's color; nil clears it.
	function color_tab(self, tab: number, color: TabColor?): ()
	--- Reorders a tab at a one-based position within its session.
	function order_tab(self, tab: number, position: number): ()
	--- Renames a session; blank and unknown sessions return false with a reason.
	function rename_session(self, session: number, name: string): (boolean, string?)
	--- Captures all trees, floats, names, colors, tab order and parked panes.
	function snapshot(self): LayoutSnapshot
	--- Restores a snapshot, refusing if blocks opened or closed since.
	function restore(self, snap: LayoutSnapshot): (boolean, string?)
	--- Regroups, renames, recolors and orders tabs atomically; empty tabs close, no program ends.
	--- Keeps focus on the same block and shows the tab now holding it.
	function arrange(self, spec: Arrange): ({ TabArrangement }?, string?)
end

--- Full notebook output; MIME values retain JSON values and base64 binary payloads.
export type NotebookOutput =
	{ type: "stream", name: "stdout" | "stderr", text: string }
	| { type: "display_data", data: { [string]: any }, metadata: { [string]: any },
		display_id: string? }
	| { type: "execute_result", count: number?, data: { [string]: any },
		metadata: { [string]: any } }
	| { type: "error", ename: string, evalue: string, traceback: { string } }

--- A notebook cell; numeric id is stable while its loaded notebook view lives.
export type NotebookCell = {
	id: number, nbformat_id: string, kind: "code" | "markdown" | "raw",
	source: string, count: number?, state: "idle" | "queued" | "running",
	outputs: { NotebookOutput },
}

--- Live kernel details; absent values are tern.json.null in returned tables.
export type NotebookKernel = {
	status: "not_started" | "starting" | "idle" | "busy" | "dead",
	label: string, python: string?, version: string?, implementation: string?,
	env: string?, pid: number?, waiting: number, running: number, dead: string?,
	input: { prompt: string, password: boolean }?,
}

--- Window only: notebooks, undoable cell edits and asynchronous Python execution.
declare extern type NotebookCx with
	--- Opens the path beside the focused pane; returns its notebook pane id.
	function open(self, path: string): number
	--- All loaded cells in document order, including complete source and outputs.
	function cells(self, nb: number): { NotebookCell }
	--- Runs exactly one stable cell id or all=true; awaits full output snapshots in run order.
	--- Python exceptions are error outputs; kernel loss or replaced cells fail the awaitable.
	function run(self, nb: number, opts: { cell: number?, all: boolean? }): Awaitable<{ NotebookCell }>
	--- Inserts an undoable cell and returns its id; no after appends, after=0 prepends.
	function add(self, nb: number, opts: {
		kind: "code" | "markdown" | "raw", source: string, after: number?
	}): number
	--- Undoably replaces source, preserving outputs; queued or running cells cannot be replaced.
	function edit(self, nb: number, cell: number, source: string): ()
	--- Reads interpreter, execution/input state and any kernel failure.
	function kernel_status(self, nb: number): NotebookKernel
	--- Restarts without a confirmation sheet; awaits the new kernel's idle status.
	function kernel_restart(self, nb: number): Awaitable<NotebookKernel>
end

--- Window only: the `cx` of window handlers, valid only during the call.
--- Plugins and Carly share ordinary window capabilities, reveal defaults and raw closes.
--- Argument validation, target lookup and operation errors still apply.
declare extern type WindowCx with
	--- Panes and tabs.
	session: SessionCx
	--- Named sessions across attached machines.
	sessions: SessionsCx
	--- Machines, connections, identities and remote shell tabs.
	hosts: HostsCx
	--- Pane and tab arrangement.
	layout: LayoutCx
	--- Owner-tagged persistent native canvases.
	canvas: CanvasCx
	--- Desktop text/Markdown files and session-owned notes.
	docs: DocsCx
	--- Desktop Markdown task boards, lanes and stable card references.
	board: BoardCx
	--- Browser automation; snapshot reads and ref-based actions return awaitables.
	browser: BrowserCx
	--- Desktop SQLite queries and explicit writes on VM-owned connections.
	db: DbCx
	--- Whole-tree validated preferences, theme catalog and user keybind overrides.
	settings: SettingsCx
	--- Local repository reads, staging, commits, branches and stashes through the git block engine.
	git: GitCx
	--- Notebook cells and asynchronous kernel execution.
	notebook: NotebookCx
	--- Palette actions and the effective keymap, including plugin commands.
	actions: ActionsCx
	--- Agent conversations, progress-aware waits and omp chat transcripts.
	agents: AgentsCx
	--- Local-machine processes, signals, task manager and sampled profiles (desktop only).
	procs: ProcsCx
	--- Display discovery and screen sharing.
	screen: ScreenCx
	--- Types `text` into `pane` (end with `"\r"` to run it).
	function run(self, pane: number, text: string): ()
	--- Opens a file path or URL (plugin calls skip routes; Carly files use origin `carly`, URLs use link routes).
	function open(self, target: string, how: How?, reveal: Reveal?): ()
	--- Opens a plugin block of `kind` (`"plugin.kind"`) on the window's current host; returns its pane (nil, with no
	--- toast, when no Ready plugin there defines it).
	function new_block(self, kind: string, args: { string }?, how: How?, reveal: Reveal?): number?
	--- Runs a built-in command by id (`"new_tab"`; built-in ids only, else ``unknown command `<id>` ``); overrides don't re-run.
	function command(self, id: string): ()
	--- Runs any action name, including plugin.<id>.<command> and keymap/view actions.
	--- Plugins and Carly use the same ordinary action dispatch.
	function action(self, name: string): ()
	--- Opens Carly and asks a shown question from a plugin callback.
	function ask_carly(self, text: string): ()
	--- Shows a toast; `sub` is a secondary line.
	function toast(self, level: ToastLevel, text: string, sub: string?): ()
	--- Copies `text` to the clipboard.
	function copy(self, text: string): ()
end

--- Desktop window only: durable Carly task. Exactly one of when/on is required.
export type TaskSpec = {
	--- Nonempty title, at most 60 characters.
	title: string,
	--- "in 20m", "at 15:00", "at 2026-10-04 09:00", "every 30m",
	--- "daily 09:00", "weekdays 09:00" or "mon,thu 18:30".
	--- Intervals >= 5 minutes for model turns; >= 1 minute for promptless checks.
	when: string?,
	--- Window event: command_finished, command_started, pane_closed, pane_created,
	--- tab_created, tab_closed, title, cwd or focus. Needs a check or pane.
	on: string?,
	--- Luau source returning function(ev, cx, state), never a saved closure.
	--- nil is quiet; false ends the task; string/table escalates with prompt.
	--- A string without prompt is shown directly. Compile/dry-run errors name check.
	check: string?,
	--- Self-contained instructions for the turn; defaults to title when check is absent.
	prompt: string?,
	--- Anchor pane in the creating window.
	pane: number?,
	--- End after first escalation; defaults true for in/at and event tasks.
	once: boolean?,
	--- Event watch lifetime, duration such as "12h"; default 12h, maximum 7d.
	expires: string?,
	--- Replace this task while retaining its history; built-in task 0 cannot be replaced.
	id: number?,
}

export type TaskRun = {
	at: string,
	outcome: "quiet" | "said" | "failed" | "missed" | "skipped",
	text: string?,
}

--- All windows' tasks, including the built-in heartbeat (id 0).
export type TaskInfo = TaskSpec & {
	id: number,
	window: string,
	by: string, -- "carly", "user", "plugin:<id>" or "heartbeat"
	paused: boolean,
	next: string?, -- RFC 3339; nil for event tasks
	runs: { TaskRun }, -- last ten runs
}

--- Callback-safe handle for one Carly export; retained after its scoped cx expires.
declare extern type ExportCall with
	--- True after cancellation, expiry, reply or plugin reload.
	cancelled: boolean
	--- Answers once; JSON-safe values only, at most 1 MiB encoded (a longer string is cut and ends with
	--- `[… N bytes cut]`); false after completion.
	function reply(self, value: any): boolean
	--- Fails once; false after completion.
	function fail(self, message: string): boolean
	--- Return this marker to answer from a later callback (default 30000, max 120000 ms).
	function wait(self, ms: number?): userdata
end

--- Signature and purpose shown by help("plugins.<id>").
export type ExportSpec = {
	--- Luau signature, 1–1024 bytes.
	sig: string,
	--- Plain-language documentation, 1–1024 bytes.
	doc: string,
}

declare tern: {
	--- Window only: exports/context, durable scheduling and Carly canvas handlers.
	carly: {
		--- Registers/replaces an export; name matches [a-z][a-z0-9_]{0,23}, max 16.
		--- Receives positional JSON-safe args followed by a fresh cx and ExportCall.
		--- Carly calls await(plugins[id][name](...)); returns cross as one value or an array.
		export: (name: string, spec: ExportSpec, fn: (...any) -> ...any) -> (),
		--- Registers/replaces request data; fn(cx) returns string or nil, budget 4 ms.
		--- Loaded providers appear, capped at 400 chars each and 1600 total.
		context: (fn: (cx: WindowCx) -> string?) -> (),
		--- Desktop scoped window call; loaded plugins record by = "plugin:<id>".
		--- Returns an id; invalid fields and check compile/dry-run failures raise.
		schedule: (spec: TaskSpec) -> number,
		--- Every window's tasks, including heartbeat task 0.
		tasks: () -> { TaskInfo },
		--- False when missing; built-in heartbeat task 0 cannot be deleted.
		cancel: (id: number) -> boolean,
		--- Carly VM only: canvas action handler; return true to suppress the quoted fallback turn.
		on_canvas: (fn: (ev: CanvasAction, cx: WindowCx) -> boolean?) -> (),
	},
	--- Which half of the plugin this VM runs.
	context: Context,
	--- The loaded plugin.
	plugin: PluginInfo,
	--- The machine and VM.
	runtime: RuntimeInfo,

	--- Logging to Tern's log (target `tern::plugin`); arguments are joined with tabs via `tostring` (global `print` is `info`).
	--- The default filter drops info/debug: set e.g. `STENCIL_LOG=warn,stencil=info,tern::plugin=debug`.
	log: {
		--- Logs at debug level.
		debug: (...any) -> (),
		--- Logs at info level.
		info: (...any) -> (),
		--- Logs at warn level.
		warn: (...any) -> (),
		--- Logs at error level.
		error: (...any) -> (),
	},

	--- JSON conversion.
	json: {
		--- Encodes `value` as JSON, indented when `pretty`; tables with keys 1..n become arrays.
		encode: (value: any, pretty: boolean?) -> string,
		--- Decodes JSON `text`; arrays come back array-tagged, null object members are dropped, member order is lost,
		--- numbers outside i64 become floats.
		decode: (text: string) -> any,
		--- Tags `t` (or a new empty table) as a JSON array, so an empty one encodes as `[]`.
		array: (<T>(t: T) -> T) & (() -> { any }),
		--- JSON null, for array items and explicit nulls.
		null: JsonNull,
	},

	--- Base64 conversion of binary strings.
	base64: {
		--- Encodes `bytes` in the standard alphabet with padding; `url = true` uses the URL-safe alphabet without padding.
		encode: (bytes: string, opts: { url: boolean? }?) -> string,
		--- Decodes the standard or URL-safe alphabet, padding optional, ASCII whitespace ignored;
		--- invalid input raises `tern.base64.decode: <reason>`.
		decode: (text: string) -> string,
	},

	--- Monotonic milliseconds (fractional).
	now: () -> number,

	--- The value of environment variable `name` in this context's process, or nil.
	getenv: (name: string) -> string?,

	--- Calls `fn` once after `ms` milliseconds; in the window context it receives a fresh `cx`.
	timer: (ms: number, fn: (cx: WindowCx?) -> ()) -> TimerHandle,

	--- Resolves after finite, nonnegative `ms` milliseconds on the owner's timer clock.
	sleep: (ms: number) -> Awaitable<nil>,

	--- Persistent key/value store of JSON values in `<data>/kv.json`, shared by both halves and every window: each
	--- VM reads the file again when it changed on disk, so gets see other VMs' sets.
	--- A missing store is empty. Read/invalid-JSON errors raise, retain the last
	--- successful cache and retry on the next call; errors never overwrite it.
	kv: {
		--- The value stored under `key`, or nil.
		get: (key: string) -> any,
		--- Stores `value` under `key`; nil deletes it.
		set: (key: string, value: any) -> (),
	},

	--- Window only (plugin halves, not Carly's VM): this plugin's secrets in the system keychain, account
	--- `plugin.<id>.<name>`; headless sessions keep them in memory. Names match [a-z0-9][a-z0-9._-]{0,63}.
	--- Bad names, values over 64 KiB and store failures fail the awaitable.
	secrets: {
		--- The secret stored under `name` (any bytes), or nil.
		get: (name: string) -> Awaitable<string?>,
		--- Stores `value` under `name`, replacing any; nil deletes it.
		set: (name: string, value: string?) -> Awaitable<nil>,
	},

	--- Synchronous file access on this context's machine; relative paths resolve against the plugin dir, failures raise.
	fs: {
		--- Reads raw bytes (no UTF-8 check). With max_bytes, requires a regular
		--- non-symlink file and rejects oversize input before allocating/reading it.
		--- Raced growth is bounded too; nonregular targets never block on open.
		read: (path: string, max_bytes: number?) -> string,
		--- Writes `text` to a file, replacing it.
		write: (path: string, text: string) -> (),
		--- Entry names of a directory, sorted.
		list: (path: string) -> { string },
		--- Whether the path exists.
		exists: (path: string) -> boolean,
		--- Creates a directory and its parents.
		mkdir: (path: string) -> (),
		--- Removes a file, or a directory recursively.
		remove: (path: string) -> (),
	},

	--- Child processes on this context's machine.
	process: {
		--- Runs `argv`; without `cb` returns an awaitable, otherwise calls `cb` (window: plus a fresh `cx`); spawn failures raise. Raises on iOS.
		run: ((argv: { string }, opts: ProcessOptions?, cb: (result: ProcessResult, cx: WindowCx?) -> ()) -> ())
			& ((argv: { string }, cb: (result: ProcessResult, cx: WindowCx?) -> ()) -> ())
			& ((argv: { string }, opts: ProcessOptions?) -> Awaitable<ProcessResult>),
	},

	--- Sends an HTTP(S) request; without `cb` returns an awaitable, otherwise calls `cb` with the whole response
	--- (window: plus a fresh `cx`); a malformed URL, method or header raises, network failures set `result.error`.
	fetch: ((url: string, opts: FetchOptions?, cb: (result: FetchResult, cx: WindowCx?) -> ()) -> ())
		& ((url: string, cb: (result: FetchResult, cx: WindowCx?) -> ()) -> ())
		& ((url: string, opts: FetchOptions?) -> Awaitable<FetchResult>),

	--- View builders, the same ones Tern's built-in lenses use.
	ui: {
		--- A span of `t` styled with the tokens `s`.
		span: (t: string, s: string?) -> Span,
		--- A span of `t` linking to `href`.
		link: (t: string, href: string, s: string?) -> Span,
		--- A `path` span linking to the file (relative paths resolve against `cwd`; unlinked without it).
		path: (p: string, cwd: string?) -> Span,
		--- A text node of `spans`.
		text: (spans: Spans) -> Node,
		--- A vertical stack of parts, small gaps between them.
		col: (children: { Node }) -> Node,
		--- Lines of one text stream stacked with no gap.
		lines: (children: { Node }) -> Node,
		--- A horizontal row, small gaps between children.
		row: (children: { Node }) -> Node,
		--- A card headed by `head`.
		card: (head: Spans, children: { Node }) -> Node,
		--- A lighter disclosure group headed by `head`.
		section: (head: Spans, children: { Node }) -> Node,
		--- A divider, labeled when `label` isn't empty.
		rule: (label: Spans) -> Node,
		--- A pill chip of `t` in `tone`.
		badge: (t: string, tone: string) -> Node,
		--- An aligned key/value list of `{keySpans, valueSpans}` pairs.
		kv: (items: { { Spans } }) -> Node,
		--- A table; `rows[i][j]` is the cell of `cols[j]`.
		table: (cols: { Col }, rows: { { Cell } }) -> Node,
		--- A meter cell at `value` (0–1), tinted warn/bad at or above `thresholds` `{warn, bad}`.
		meter_cell: (value: number, thresholds: { number }?) -> MeterCell,
		--- A meter cell stacking `{value, themeToken}` parts from the left.
		meter_parts_cell: (parts: { { any } }) -> MeterCell,
		--- A disclosure tree.
		tree: (nodes: { TreeNode }) -> Node,
		--- A list of items.
		list: (items: { Item }) -> Node,
		--- Highlighted code in `lang`; line numbers show from `start` when given.
		code: (text: string, lang: string?, start: number?) -> Node,
		--- A unified diff; `path` picks the grammar.
		diff: (text: string, path: string?) -> Node,
		--- Markdown.
		md: (text: string) -> Node,
		--- A meter at `value` (0–1) labeled `label`.
		meter: (value: number, label: Spans) -> Node,
		--- A progress bar at `value` (0–1), indeterminate when nil.
		progress: (value: number?, label: Spans) -> Node,
		--- A bar chart of `{label, value}` pairs.
		bars: (series: { { any } }) -> Node,
		--- A sparkline of `{label, value}` pairs.
		spark: (series: { { any } }) -> Node,
		--- A mini terminal showing ANSI `text`.
		ansi: (text: string) -> Node,
		--- A diagnostic card; its location links to the file (relative paths resolve against `cwd`).
		diagnostic: (d: Diag, cwd: string?) -> Node,
		--- A pass/fail/skip meter with counts and an optional duration.
		test_summary: (passed: number, failed: number, skipped: number, took: string?) -> Node,
		--- A muted "… N more (Raw shows everything)" line.
		overflow: (hidden: number) -> Node,
		--- Any TSP node: `{k = kind, p = props, c = children}`.
		node: (kind: string, props: { [string]: any }?, children: { Node }?) -> Node,
		--- A generic HTML-subset element `tag` (`div`, `form`, `label`, `button`, `input`, …); props: `class`,
		--- `attrs` (`data-*`, `aria-*`, `role`, `colspan`, `rowspan`), `text`, common props; `input` takes `type`
		--- (`checkbox`|`radio`), `name`, `value`, `checked`, `disabled`.
		el: (tag: string, props: { [string]: any }?, children: { Node }?) -> Node,
	},

	--- Output parsers, the same ones Tern's built-in lenses use.
	parse: {
		--- Reads whitespace-aligned lines (the first is the header) as a table; nil for prose.
		columns: (lines: { string }) -> Columns?,
		--- Splits packed multi-column listings (`ls`) into their entries.
		packed: (lines: { string }) -> { string },
		--- A JSON document (or stream of documents) as a collapsible tree; nil when not JSON.
		json_tree: (text: string) -> Node?,
		--- `file:line[:col]` at the start of `s`: path, line, column.
		location: (s: string) -> (string?, number?, number?),
		--- Human size (`"4.0K"`, `"1.2GB"`) in bytes.
		size: (s: string) -> number?,
	},

	--- Host only: plugin block types.
	block: {
		--- Host only: implements manifest block `id` (kind `<plugin>.<id>`); annotate `def` as `BlockDef<MyState>` to type its state.
		define: (id: string, def: BlockDef<any>) -> (),
	},

	--- Host only: command lenses.
	lens: {
		--- Host only: implements manifest lens `id` (matched by its `match` globs); annotate `def` as `LensDef<MyState>` to type its state.
		define: (id: string, def: LensDef<any>) -> (),
	},

	--- Host only: the host's panes.
	pane: {
		--- Host only: writes `text` into a pane's pty (`cmd .. "\r"` runs it).
		write: (pane: number, text: string) -> (),
		--- Host only: every pane on this host.
		list: () -> { PaneInfo },
	},

	--- Subscribes `fn` to lifecycle event `name`; annotate its parameters with the payload types.
	--- Host: `spawn(spec: SpawnSpec) -> SpawnSpec?` (sync filter) and `(ev, cx: EffectCx)` for `command_started`,
	--- `command_finished`, `cwd`, `title`, `pane_exited`. Window: `window_start(cx: WindowCx)` and `(ev, cx: WindowCx)`
	--- for `focus`, `pane_created`, `pane_closed`, `tab_created`, `tab_closed`, `command_started`, `command_finished`, `cwd`,
	--- `title` and `canvas_action(ev: CanvasAction, cx) -> boolean?` (truthy consumes the owner's click).
	--- `window_start` runs once per plugin per window. The `spawn` wait (50 ms) counts from when the request is queued.
	on: (name: string, fn: (...any) -> ...any) -> (),

	--- Window only: adds a palette command, also bindable as action `plugin.<plugin>.<id>`.
	command: (def: CommandDef) -> (),

	--- Window only: binds `chord` to an action name (`"new_tab"`, `"plugin.*"`; not `"unbind"`) or a function; binds whose
	--- chord or action doesn't parse, or whose chord the preset takes, are left out.
	bind: (chord: string, action: string | ((cx: WindowCx) -> ())) -> (),

	--- Window only: runs `fn` before built-in command `command_id` (built-in ids only, else ``unknown command `<id>` ``);
	--- any truthy return skips the built-in.
	override: (command_id: string, fn: (cx: WindowCx) -> boolean?) -> (),

	--- Window only: routing of file opens and link clicks.
	route: {
		--- Window only: may claim file opens; nil runs the built-in open.
		open: (fn: (req: OpenRequest, cx: WindowCx) -> RouteDecision?) -> (),
		--- Window only: may claim link clicks; nil runs the built-in handling.
		link: (fn: (link: LinkRequest, cx: WindowCx) -> RouteDecision?) -> (),
	},

	--- Window only: pure formatters for window chrome, cached per input (cleared on reload, a registration or
	--- `refresh`). The chrome redraws on session changes and pane output; `refresh` redraws it for anything else.
	chrome: {
		--- Window only: a tab's title; nil keeps the default.
		tab_title: (fn: (tab: TabInfo) -> string?) -> (),
		--- Window only: the window's title; nil keeps the default.
		window_title: (fn: (win: WindowInfo) -> string?) -> (),
		--- Window only: status line segments for the focused pane; nil adds none.
		status: (fn: (pane: StatusPaneInfo) -> { StatusSegment }?) -> (),
		--- Window only: drops this window's cached formatter results and redraws its tabs, status line and title on
		--- the next frame, so answers that depend on more than their input (a clock, a file, a process result) show.
		--- Call it from a timer, event or command, not from a formatter (that raises an error).
		refresh: () -> (),
	},

	--- Window only: installs global CSS as sheet `plugin:local:<id>:<name>`, replacing an earlier one of that name.
	--- Plugin blocks carry `data-surface="plugin.<plugin>.<block>"` on `.sf-main`/`.sf-dock`/`.sf-layer`; lens blocks
	--- are `.sf-block[data-role='lens.plugin.<plugin>.<lens>']`. The web tab runs no window entry, so `tern.css`
	--- sheets don't reach it; manifest `styles` do.
	css: (name: string, source: string) -> (),
}
