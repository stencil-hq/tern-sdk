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
form = ui.html.form(
    ui.html.p("Which size?"),
    ui.html.label(ui.html.input(type="radio", name="size", value="s"), "Small"),
    ui.html.label(ui.html.input(type="radio", name="size", value="l"), "Large"),
    ui.html.button("Create", actions={"click": "submit"}),
)
try:
    answer = tern_sdk.ask(form)
    print(answer.values["size"] if answer else "cancelled")
except tern_sdk.Unsupported:
    size = input("Which size? ")
```

A live surface:

```python
session = tern_sdk.connect(app="build")
if session is None:
    print("building...")            # no TSP: plain output
else:
    with session, session.open(mode="flow") as surface:
        surface.render(ui.card(ui.spinner("Compiling"), head="Build", status="running", key="b"))
        for item in session.input(timeout=0.1):
            ...
```

`connect()` returns `None` when TSP isn't available (no exception for that
case); a `Session` and a `Surface` are context managers that close on exit.

## Layers

Each layer is usable without the ones above it.

| Module | What it provides |
| --- | --- |
| `tern_sdk.wire` | Constants (`VERSION`, `APC_LIMIT`, `CREDITS`, `KINDS`, `TEXT_KINDS`, features), program -> terminal messages (`hello`, `blobs_query`, `open_surface`, `frame`, `blob`, `palette`, `stylesheet`, `close_surface`, `Ops.*` for the twelve frame ops), the encoder (`encode`, `chunks`, `ChunkIds`) and typed replies and events (`decode_reply`, `decode_event`; every one keeps its `raw` object) |
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

Kind-specific props replace colliding common props: `tool`/`picker` titles
accept spans, `prefs` owns its title, `list` owns `max`, and `picker` owns
its action list. `AgentStatus` is separate from card/tool `Status`.
Table meter cells use `{"meter": {"value": 0.5}}` (or `parts` inside
`meter`), typed as `MeterCell` containing `MeterCellProps`; plain tables
render those meters too.

### Session

- `connect(app=None, *, version=None, features=(), timeout=1.0, paste=True, kitty=True)`
  checks the environment, switches the tty to raw mode, sends `hello` + DA1
  and returns a `Session`, or `None`.
- `start(input, output, ...)` runs the handshake on given file descriptors
  or binary streams without the environment checks (tests drive it over
  `os.pipe()`).
- `session.open(id=None, *, mode="inline", title=None, role=None, listen=True, adopt=None, keep=True)`
  opens a surface (`s1`, `s2`, ...).
- `surface.render(view)` or `surface.render(main=..., dock=..., layer=...)`
  sends only the difference. A node given for a region is that region's
  root, whose own kind and props Tern doesn't draw (only its children); a
  list region is wrapped in a `col`; a bare node or list passed as the whole
  view becomes `main`'s children. Also `stylesheet(name, css)`, `palette(dark=, light=, name=)`,
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

- `show(view, *, css=None, fallback=None, session=None)`: static output in a
  `flow` surface opened with `listen: false`. It is named `show`, not
  `print`, so it never shadows the builtin. Without TSP it prints
  `plain(view)` or `fallback`.
- `ask(view, *, css=None, submit="submit", session=None) -> Answer | None`:
  returns the first `action` whose `act` is `submit` as
  `Answer(id, act, values)`, or `None` on Escape, Ctrl+C or Ctrl+D. Without
  TSP it raises `tern_sdk.Unsupported`.
- `plain(view, cols=80) -> str`: readable plain text without escapes.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every TSP message as JSONL (`surface-play` replays it) |
| `TMUX`, `STY`, `ZELLIJ` | Inside a multiplexer: no TSP |

## Examples

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

```sh
uv run examples/table.py      # the current directory as a table
uv run examples/ask.py        # "Which size?"
uv run examples/progress.py   # a running card, streamed log, progress bar
uv run examples/chat.py       # transcript + editor; Enter sends, Escape quits
```

## Development

```sh
uv run pytest                 # wire, UI, session, helpers
uvx mypy --strict src examples
uvx ruff check src tests examples
uvx ruff format --check src tests examples
```
