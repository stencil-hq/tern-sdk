# tern-sdk (Python)

The Python SDK for the [Tern Surface Protocol](https://docs.stencil.so/tern/protocol/index.html):
a program running in a terminal pane describes its UI as a tree of nodes and
Tern draws it natively. Outside Tern the same program falls back to plain
text.

Python 3.11 or newer, standard library only.

## Install

```sh
uv add tern-sdk
```

The distribution is `tern-sdk`; the import is `tern_sdk`.

## Quick start

```python
import tern_sdk
from tern_sdk import ui

# Static output that stays in the scrollback (plain text outside Tern).
tern_sdk.show(ui.card(ui.md("All **42** tests pass."), head="cargo test", status="done"))

# One question, its answer returned.
sizes = (("s", "Small"), ("m", "Medium"), ("l", "Large"))
form = ui.html.form(
    ui.card(
        ui.row(
            *(
                ui.html.label(ui.html.input(type="radio", name="size", value=v), ui.html.span(label), key=v)
                for v, label in sizes
            ),
            gap="lg",
        ),
        ui.button("Create", variant="primary", actions={"click": "submit"}),
        head="Which size?",
        max={"w": "48ch"},
    ),
    key="ask",
)
try:
    answer = tern_sdk.ask(form, font="sans")
    print(answer.values.get("size") if answer else "cancelled")
except tern_sdk.Unsupported:
    size = input("Which size? [s/m/l] ")
```

A live app (`font="sans"`: people's text in the UI face, machine strings stay mono):

```python
session = tern_sdk.connect(app="build")
if session is None:
    print("building...")            # no TSP: plain output
else:
    with session, session.open(mode="screen", font="sans") as surface:
        stop = ui.button("Stop", icon="stop", keys=["esc"], key="stop", on_click=lambda ev: ...)
        surface.render(ui.card(ui.spinner("Compiling"), head="Build", icon="bolt", end=stop, key="b"))
        for item in session.input(timeout=0.1):
            ...
```

## Design rules

Build views from kinds and props, not CSS; the full guide is
[Designing views](https://docs.stencil.so/tern/guides/design.html).

- Apps open with `font="sans"`; mono is for machine strings (`mono`, `code`,
  `num`, `path` span tokens, `mono` table columns).
- Buttons are `ui.button`, never `ui.html.button` with a stylesheet. One
  `primary` per view; `quiet` for secondary, `danger` for destructive ones
  behind a confirm strip (`card(tone="error")` with `quiet` + `danger` buttons).
  Show shortcuts with `keys`.
- Dashboards are pages, not piles of cards: a head row (the name in the
  `title` span token, muted meta, badges and global buttons at the end), a
  flat `ui.stats` strip (at most four tiles, never in a card), underline
  `ui.tabs` switching views (Tab/⇧Tab), and per view a full-width
  `ui.table(selectable=True, selected=...)` with rich cells (never
  `ui.html.table`; exactly one `grow` text column, numbers `align="end"` +
  `mono`) beside a detail `card` (`min={"w": "44ch"}`, `max={"w": "56ch"}`)
  holding the selected row's `kv` and its one `primary` action. The confirm
  strip and a key legend go in the dock.
- Color carries meaning only: `tone` and span tokens, no custom colors.
- Design all four states: loading (`spinner`), `empty`, error (`tone="error"`), success.

`connect()` returns `None` when TSP isn't available (no exception for that
case); a `Session` and a `Surface` are context managers that close on exit.

## Layers

Each layer is usable without the ones above it.

| Module | What it provides |
| --- | --- |
| `tern_sdk.wire` | Constants (`VERSION`, `APC_LIMIT`, `CREDITS`, `KINDS`, `TEXT_KINDS`, features), program -> terminal messages (`hello`, `blobs_query`, `open_surface`, `frame`, `blob`, `stylesheet`, `close_surface`, `Ops.*` for the twelve frame ops), the encoder (`encode`, `chunks`, `ChunkIds`) and typed replies and events (`decode_reply`, `decode_event`; every one keeps its `raw` object) |
| `tern_sdk.input` | `InputParser`: feed pty input, get key bytes, `Reply`, `Event` and `DA1` items; `flush()` releases undecided prefixes |
| `tern_sdk.keys` | `KeyDecoder`: key bytes to `Key(name, text, ctrl, alt, shift, meta)`, legacy xterm, kitty `CSI u` and bracketed paste |
| `tern_sdk.ui` | Builders for every kind but `block`, `span`, `el` builders per tag under `ui.html`, and `node(kind, props, children)` |
| `tern_sdk.reconcile` | `View.build(view)` and `View.ops(next, surface)`: Tern's own diff with derived ids |
| `tern_sdk.session` | `connect`, `start`, `Session`, `Surface`, `Capabilities` |
| `tern_sdk.helpers` | `show`, `ask`, `plain` (also at the top level) |

### Nodes

Children are positional and props keyword-only. Text is a string or styled
spans (`ui.span("3 failed", "error")`). Snake-case keywords map to the
wire's camelCase names (`target_kind` is `targetKind`, `max_lines` is
`maxLines`); `el`'s `class` is `class_`. Structured props are `TypedDict`s
whose keys are the wire's names. Enumerations are `Literal`s; `props={...}`
sets any prop untyped and `ui.node(kind, props, children)` builds any kind.

```python
ui.col(
    ui.card(ui.kv([("service", "api"), ("region", "eu-west-1")]), head="Deploy", status="running", key="d"),
    ui.row(ui.badge("v2.3.1", tone="accent"), ui.icon("check", tone="success"), ui.kbd("cmd", "k")),
    ui.html.table(ui.html.tr(ui.html.td("Cargo.toml"), ui.html.td("2.1K", class_="size"))),
)
```

The common props (`key`, `role`, `tone`, `hidden`, `mark`, `actions`,
`title`, `aria`, `href`, `grow`, `shrink`, `basis`, `min`, `max`) are
keywords on every builder. In `ui.html.*`, a leading string child is the
element's `text`; later string children become `span` elements.

### Handlers

Any builder takes handlers; each receives the typed event:

| Keyword | Runs on |
| --- | --- |
| `on_click`, `on_dblclick` | the click / double-click action (sets `actions.click` to `"click"` unless the node names its own) |
| `on_menu={"rerun": fn}` | context-menu picks (adds the names to `actions.menu`) |
| `on_action={"name": fn}` | `action` events with that `act` |
| `on_toggle`, `on_select`, `on_activate`, `on_change`, `on_focus`, `on_edit`, `on_undo`, `on_send` | those events |

Events are routed by `sf` and `id` to the node with that id in the view last
rendered (for a list's `select`/`activate`, the list). Handlers run inside
`session.poll()` / `session.input()`; handled events are consumed and every
other event reaches the loop. Named gesture actions such as `sort=name`
match the event's `act="sort"` and `value="name"`. `ask` returns its first
submit action even when a node handler handles it; that handler runs first.

Kind-specific props replace colliding common props: `tool`/`picker`/`trace`/`empty`
titles accept spans, `prefs` owns its title, `list`/`avatars`/`breadcrumb`
own `max` (a count), `slider` and `calendar`/`date` own `min` and `max`, `split`
owns `min`, and `picker`, `message` and `attachment` own their action lists.
`AgentStatus` is separate from card/tool `Status`; `ToolStatus` adds `ask`.
Table meter cells use `{"meter": {"value": 0.5}}` (or `parts` inside
`meter`), typed as `MeterCell` containing `MeterCellProps`; plain tables
render those meters too.

### Design kinds and props

- `ui.button(text, *, icon, keys, variant, size, on, disabled, busy)`: a
  push button; a click sends `action` with `act` `actions.click` or
  `"click"` (`on_click` handles it). `variant` is `default`, `primary`,
  `quiet` or `danger`; `size` `md` (28px) or `sm` (22px); `disabled` as a
  string is the reason tooltip.
- `ui.stats([{"label", "value", "sub"?, "tone"?}])`: a flat strip of stat tiles.
- `ui.table(cols, rows, *, selectable, selected, empty)`: selectable rows
  send `select` on click and `activate` on double-click with the row id as
  `item`. Rows take `disabled`, `menu` (right-click actions, sent as
  `action` with `item`) and `title`; columns take `mono`. Cells are text or
  `TextCell` (`{"text", "dot"?, "icon"?, "tone"?}`), `BadgeCell`
  (`{"badge": {...}}`), `ButtonCell` (`{"button": {"text", "act", ...}}`, a
  click sends `action` with `act` and the row as `item`), `SparkCell`
  (`{"spark": {"values": [...]}}`) or `MeterCell`. `ActionEvent.item` names the row.
- `ui.tabs(items, active=...)`: an underlined tab strip for a page's views;
  `variant="segmented"` draws an inline segmented control.
- `ui.card(..., icon, meta, end)`, `ui.overlay(..., icon, meta, end, foot)`:
  panel heads with an icon, muted meta text and controls at the head's end;
  an overlay's `foot` is a sticky footer bar. A node given for `head`, `end`
  or `foot` (also `ui.section(head=...)`) becomes a child keyed by the
  prop's name, and the prop is sent as that child's id.

### Controls, navigation and conversation kinds

Check the hello reply's `kinds` (`session.caps.kinds`) before using these
with an older Tern.

- Controls: `ui.select(options, *, value, variant, label, search, query, open, active, ...)`,
  `ui.switch(label, *, on, variant, hint, labels)`, `ui.slider(value, *, min, max, step, unit, format, ticks)`
  (a `(lo, hi)` value is a range), `ui.field(*controls, label, hint, error, warning, required, label_width, variant)`,
  `ui.otp(text, *, length, group, cursor, mask, state, hint)`, `ui.buttons(*buttons, variant, size)`,
  `ui.calendar(value, *, variant, range, month, marks, presets, ...)` and `ui.date(value, *, open, placeholder, ...)`.
  `ui.input` takes addons (`variant="field"`, `icon`, `keys`, `prefix`, `suffix`, `unit`, `clear`,
  `stepper`, `secret`, `reveal`, `buttons`); `ui.editor` takes `variant` and `buttons`.
- Navigation: `ui.breadcrumb(items)`, `ui.pager(page, *, variant, pages, total, size)`, `ui.nav(items, *, active, open)`,
  `ui.menu(items, *, text, icon, trigger, caret, open, active)` and `ui.menubar(menus, *, open, active)`.
  Menu rows are `MenuEntry`s: `"-"` a separator, `{"header": ...}`, or `MenuItem`s with `icon`, `keys`,
  `detail`, `checked`, `radio`, `disabled`, `danger` and submenu `items`; `actions={"menu": [...]}` takes
  the same rows.
- Layout and data: `ui.split(*panes, dir, sizes, min)`, `ui.scroll(*children, stick="end")`,
  `ui.carousel(*slides, index)`, `ui.empty(*actions, title, text)`, `ui.skeleton(shape)`,
  `ui.avatar(text)`, `ui.avatars(items, *, max)` and `ui.attachment(name, *, state, progress, removable)`.
- Conversation and work: `ui.message(*body, author, time, side, avatar, reactions, actions)` (an
  `avatar` node becomes a child keyed `avatar`), `ui.trace(lines, *contents)` (a child keyed by a line's
  `id` is its content), `ui.stacktrace(frames, *, error, selected)`, `ui.suggestions(items)` and
  `ui.questions(items, *other_inputs, step, answers)`.
- Extended kinds: `ui.rule(label, *, variant, icon, time, end)`, `ui.text(..., cite, sources)` with
  `ui.span(t, cite=...)`, `ui.chart` kinds `line`, `area` and `donut` (`ChartSeries`, `labels`, `unit`,
  `last`, `total`, `caption`), `ui.table(..., sort, multi, picked, variant, bar, hint)` with `sortable`
  columns, `ui.overlay(..., side, variant)` sheets and drawers, and `ui.tool(status="ask", confirm, ask, approved)`.

Their events: `ToggleEvent.open` (with `item` for a menubar menu or trace
line) for selects, menus and traces; `ActionEvent.value` is any JSON (a
table's `{col, dir}` sort or picked ids, a questionnaire's answers) and
`ActionEvent.checked` a menu row's new state; `ChangeEvent.value` is a bool,
number, pair, list or string, with `on` for a message reaction and `item`
for a question. A stacktrace's frame index arrives as `item="3"`.

### Session

- `connect(app=None, *, version=None, features=(), timeout=1.0, paste=True, kitty=True)`
  checks the environment, switches the tty to raw mode, sends `hello` + DA1
  and returns a `Session`, or `None`.
- `start(input, output, ...)` runs the handshake on given file descriptors
  or binary streams without the environment checks (tests drive it over
  `os.pipe()`).
- `session.open(id=None, *, mode="inline", title=None, role=None, listen=True, adopt=None, keep=True, closable=False, font=None)`
  opens a surface (`s1`, `s2`, ...). `closable=True` tells Tern closing the
  pane loses nothing, so it closes without the "a program is running" prompt.
  `font="sans"` draws the surface in the UI face (default: the terminal's mono).
- `surface.render(view)` or `surface.render(main=..., dock=..., layer=...)`
  sends only the difference. A node given for a region is that region's
  root, whose own kind and props Tern doesn't draw (only its children); a
  list region is wrapped in a `col`; a bare node or list passed as the whole
  view becomes `main`'s children. Also `stylesheet(name, css)`,
  `focus`, `reveal`, `scroll`, `settle`, `suspend`, `resume`, raw
  `send(ops)` and `close(keep=None)`.
- Flow control: at most `credits` frames in flight; a render while blocked
  stores the view, and the next `ack` sends one frame with the difference
  from the last view sent, then queued ops. `close` sends what is still
  pending as a last frame even without credit, so a kept surface shows its
  final state. A failed frame write raises without advancing the sent view,
  sequence, or queued ops, including a flush triggered by an ack. Re-rendering
  a mutable node detects changes to nested props.
- `adopt=True` reuses a closed surface's last-sent `main` and frame sequence
  in this session (not its former `dock`/`layer`). An unknown adopted id
  starts empty and deletes `main` before its first frame's additions.
- Input: `for item in session.input(timeout=None):` yields `Key`s and
  unhandled `Event`s and ends at end of input, on close or after `timeout`
  seconds idle; `session.poll(timeout)` returns one item or `None`.
  Even `poll(0)` reads available input, including acks. Iterating buffered
  keys also dispatches available input, so acks promptly flush pending frames
  rather than waiting for the key queue to empty. Held prefixes flush
  only after 30 ms without input, not when a shorter poll expires. Key
  decoder flush drops incomplete sequences except a lone Escape; neither
  parser flushes a paste in progress.
- `session.blob(data, mime)` sends a blob once and returns its id;
  `session.blobs(ids)` asks which Tern holds.
- `session.close()` closes open surfaces, undoes bracketed paste and the
  kitty flag, drains input for 50 ms and restores the tty. It also runs from
  the `with` block, at process exit (`atexit`) and on SIGINT / SIGTERM / SIGHUP
  when started on Python's main thread. Cleanup is armed before raw mode
  and the handshake; only one session may own tty state at a time.
  Close attempts `x` even after a failed final frame, resets modes and restores
  the tty despite write errors, then raises the first error. The drain records
  late replies without invoking handlers. Windows console waits discard
  non-character records rather than blocking on a character read.

### Helpers

- `show(view, *, css=None, fallback=None, font=None, session=None)`: static output in a
  `flow` surface opened with `listen: false`. It is named `show`, not
  `print`, so it never shadows the builtin. Without TSP it prints
  `plain(view)` or `fallback`.
- `ask(view, *, css=None, submit="submit", font=None, session=None) -> Answer | None`:
  returns the first `action` whose `act` is `submit` as
  `Answer(id, act, values)`, or `None` on Escape, Ctrl+C or Ctrl+D. Without
  TSP it raises `tern_sdk.Unsupported`.
- `plain(view, cols=80) -> str`: readable plain text without escapes. It
  prints a `row`'s children on one line, so build fallback-friendly views
  (a `col` of cards) or pass `fallback` to `show`.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every TSP message as JSONL (`surface-play` replays it) |
| `TMUX`, `TMUX_PANE` | Inside tmux: TSP only when Tern draws the session (`@tern-tsp`), with the pane's `allow-set-title` off while connected |
| `STY`, `ZELLIJ` | Inside screen or zellij: no TSP |

## Examples

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

```sh
uv run examples/table.py      # the current directory as a table
uv run examples/ask.py        # "Which size?"
uv run examples/progress.py   # a running card, streamed log, progress bar
uv run examples/chat.py       # transcript + editor; Enter sends, Escape quits
uv run examples/dashboard.py  # head, stats, tabbed table + detail; j/k, Tab, Enter, a, r, q
```

## Development

```sh
uv run pytest                 # wire, UI, session, helpers
uvx mypy --strict src examples
uvx ruff check src tests examples
uvx ruff format --check src tests examples
```
