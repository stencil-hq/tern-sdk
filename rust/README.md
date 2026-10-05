# tern-sdk (Rust)

Talk to Tern over the Tern Surface Protocol from any Rust program running in
a terminal pane: describe the UI as a tree of nodes, and Tern lays it out,
draws and animates it natively. Outside Tern the same program prints plain
text.

Synchronous, no async runtime: `serde`, `serde_json`, `sha2`, `base64`,
`thiserror`, `tracing`, and `libc` / `windows-sys` for the tty.

## Install

```sh
cargo add tern-sdk
```

The library is `tern_sdk`.

The wire, input, node and reconcile layers build on desktop (macOS, Linux,
Windows), iOS and `wasm32-unknown-unknown`. The process tty (`term::Tty`,
tty detection, `Session::connect`, `print` and `ask`) is available only on
Unix and Windows; these APIs are target-gated rather than stubbed on wasm.
`term::Terminal` and the session's custom-transport API remain available.

Tern's plugin worker uses `tern_sdk::reconcile` directly.

## Quick start

```rust
use tern_sdk::{PrintOptions, ui, ui::View};

fn main() -> Result<(), tern_sdk::Error> {
	let view: View = View::new().main([ui::card()
		.head("Deploy")
		.status(ui::Status::Done)
		.child(ui::kv().item("service", "api-gateway").item("region", "eu-west-1"))]);
	// A native card that stays in the scrollback; plain text anywhere else.
	tern_sdk::print(view, PrintOptions::new())
}
```

An interactive program connects a session, opens a surface, renders views
and reads input:

```rust
use tern_sdk::{Input, Options, Session, SurfaceOptions, ui};

#[derive(Clone)]
enum Msg {
	Deploy,
}

fn main() -> Result<(), tern_sdk::Error> {
	let Some(mut session) = Session::<Msg>::connect(Options::new())? else {
		return Ok(()); // no TSP here: use your own output
	};
	let sf = session.open(SurfaceOptions::flow())?;
	session.render(sf, ui::html::button("Deploy").on_click(Msg::Deploy))?;
	while let Some(input) = session.next(None)? {
		match input {
			Input::Msg(Msg::Deploy, _) => break,
			Input::Key(key) if key.is("escape") => break,
			_ => {},
		}
	}
	session.close()
}
```

## The layers

| Layer | Module | What |
| --- | --- | --- |
| 1. Wire | `tern_sdk::wire` | Constants (`VERSION`, `APC_LIMIT`, `CREDITS`, `kind::*`, `feature::*`), the messages (`Query`, `Open`, `Frame` with every `Op`, `Blob`, `Palette`, `Sheet`, `Close`), the encoder (`encode`, `chunks`, `Encoder`) and the decoded `Reply` and `Event` (typed variants plus `Unknown`; `raw()` is the JSON object as received) |
| 2. Input | `tern_sdk::input`, `tern_sdk::keys` | `input::Parser` splits replies, events and DA1 answers out of the pty's input and passes everything else through; `keys::Decoder` turns key bytes into `Key`s (`key.is("ctrl+c")`) |
| 3. Nodes | `tern_sdk::ui`, `tern_sdk::ui::html` | A builder per kind (`ui::card()`, `ui::table()`, `ui::editor()`, …) with typed props, the common props, `Text` as a string or `spans![…]`, typed enumerations with an `Other` escape hatch, `el` builders per tag (`html::form()`, `html::radio(name, value)`, …), and `.prop(name, value)` / `ui::node(kind)` for anything untyped |
| 4. Reconcile | `tern_sdk::reconcile` | `Doc` (a view with derived ids) and `doc.ops(&next, surface)`, Tern's plugin algorithm |
| 5. Session | `tern_sdk::Session` | Detection and handshake, raw mode, bracketed paste and kitty keys, surfaces, credits, blobs, routing, recording, close |
| 6. Helpers | `tern_sdk::{print, ask, plain}` | Static output, one question, the plain-text fallback |

### Views and regions

A `View` has three regions, `main`, `dock` and `layer`. A node given for a
region is the region root itself; a list (an array, a `Vec` or `nodes![…]`)
is wrapped in a `col` root. Tern draws a region's children into an element of
its own and ignores the root's kind and props (but `role`), so a root should
be a container. A bare node or list converts into a view as `main`'s
children, so `session.render(sf, node)` and `session.render(sf, [a, b])`
work.

Ids derive from the tree: a region root is `main`, `dock` or `layer`, a child
is `<parent>.<key>` (its `key` prop, else its index). The editor in
`View::new().dock([ui::editor().key("ed")])` is `dock.ed`, which is what
`session.focus(sf, Some("dock.ed"))` names.

`Doc::from_json` accepts the plugin wire form `{main?, dock?, layer?}`:
region roots must be nodes with a string `k`, not lists. A null view is
empty; null or absent `p`/`c` means no props/children. An empty object `c`
also means no children, matching an empty Luau table. Other non-object
props and non-list children are errors. Unknown regions are rejected even
when null. Duplicate sibling identities report the parent path and key
(for example, `main: duplicate child key "a"`).

### Handlers

Nodes are generic over the program's message type `M` (`Node<M>`,
`View<M>`, default `()`). Handlers map events to messages instead of
closures with side effects:

| Builder method | Runs on |
| --- | --- |
| `.on_click(msg)`, `.on_dblclick(msg)` | the node's click / double-click action (set to `"click"` / `"dblclick"` unless the node names it) |
| `.on_menu("rerun", msg)` | the context-menu entry it adds |
| `.on_action("name", msg)`, `.on_action_with("name", \|a\| …)` | `action` events with that `act` (`name` or `name=value`) |
| `.on_toggle(\|collapsed\| …)`, `.on_select(\|item\| …)`, `.on_activate(\|item\| …)`, `.on_change(\|c\| …)`, `.on_edit(\|e\| …)`, `.on_send(\|text\| …)` | those events |
| `.on_focus(msg)`, `.on_undo(msg)` | those events |

An event is routed by its `sf` and `id` to the node with that id in the view
last rendered (for a list's `select`/`activate`, the item's own handler first,
then the list's). `Session::next` returns `Input::Msg(msg, event)` for a
handled event, `Input::Event(event)` for any other (resize, theme, motion,
errors, `gone`, …) and `Input::Key(key)` for keys. Acks are consumed inside.

### Flow control and closing

At most `credits` frames are unacknowledged per listening surface. A
`render` while blocked only stores the view (and view ops queue); the frame
that carries the difference goes out when an ack arrives, which happens
while the program calls `next` or `pump`. `listen(false)` surfaces send at
once. `close_surface` (and the session's close) first sends whatever still
waits as one last frame, credits or not, then `x`. Dropping a `Session`
closes it: open surfaces (kept unless `SurfaceOptions::keep(false)`), the
input modes, a 50 ms drain of late input, the tty. On Unix, SIGTERM, SIGHUP,
SIGINT and SIGQUIT also restore the tty before the process dies.

`SurfaceOptions::default()` listens and keeps its surface. Adopting an id
closed by this session resumes its last sent `main` and frame sequence;
`dock` and `layer` start empty. An unknown adopted id first deletes `main`.
Frame writes commit the sent view, sequence and queued operations only on
success; an ack-triggered write error reaches `next`/`pump` for handling.
Close still attempts `x`, mode reset, and tty restoration after a write error.

Only one `Tty` can own process terminal state at a time. Setup arms cleanup
before changing modes and rolls back partial failures. Unix signal handlers
use async-signal-safe mode/tty restoration, then re-raise the signal; they
cannot run Rust allocation, user handlers, or the normal session close from
the signal context.

Input idle flushes wait for 30 ms without input, not a poll's shorter timeout.
Paste content and partial paste end markers remain held across idle flushes;
other incomplete key sequences are dropped. Zero-timeout polling dispatches
already-ready input, including acknowledgements.

Tests and custom transports drive a session over anything implementing
`term::Terminal` (`Session::with_terminal`).

## Helpers

- `print(view, PrintOptions::new().css(…).fallback(…))`: a `flow` surface
  opened with `listen:false`, its sheet, one frame, `x` with keep. Without TSP
  (or when stdout is not a tty) the plain rendering, or the fallback text.
- `ask(view, AskOptions::new().css(…).submit("submit"))` returns
  `Answer::Submitted(Submission { id, act, values })`, `Answer::Cancelled`
  (Escape, Ctrl+C, Ctrl+D) or `Answer::Unsupported`. `ask_with(view, opts,
  |msg| …)` also runs the form's handlers while it waits, and renders the view
  the closure returns.
- `plain(view, cols)`: readable plain text without escapes.

`print` and `plain` take message-free views, so bare builder chains need no
type annotations. `ask_with` remains generic over handler messages; a submit
handler runs before the submission is returned. For example:

```rust
tern_sdk::print(
	tern_sdk::ui::card().head("cargo test").status(tern_sdk::ui::Status::Done)
		.child(tern_sdk::ui::md("All **42** tests pass.")),
	tern_sdk::PrintOptions::new(),
)?;
```

Agent builders use `ui::AgentStatus` (`Pending`, `Running`, `Done`, `Failed`,
`Aborted`, `Idle`, `Parked`, or `Other`), distinct from card/tool `Status`.
Table meter cells serialize as `{ "meter": { "value": 0.5 } }`; their plain
fallback renders the same nested value. Inline key/value layouts stay on one
line, and blocked checklist entries have a distinct `[!]` marker. Headed
statuses use parentheses, for example `cargo test (done)`, consistently
across cards, sections, overlays, tools and agents.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every message, both ways, as JSONL |
| `TMUX`, `STY`, `ZELLIJ` | Inside a multiplexer: no TSP |

## Examples

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

```sh
cargo run -p tern-sdk --example table     # print: the current directory
cargo run -p tern-sdk --example ask       # ask: "Which size?"
cargo run -p tern-sdk --example progress  # a flow surface with a running card
cargo run -p tern-sdk --example chat      # an inline surface with a composer
```

## Tests

```sh
cargo test
```

The suite drives sessions over a scripted in-memory terminal.
