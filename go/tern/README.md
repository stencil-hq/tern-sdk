# Tern SDK for Go

`github.com/stencil-hq/tern-sdk/go/tern` lets a Go program running in a
terminal pane talk to Tern over the
[Tern Surface Protocol](https://docs.stencil.so/tern/protocol/index.html):
it describes its UI as a tree of nodes, and Tern draws it natively. Outside
Tern the same program falls back to plain text.

## Install

```sh
go get github.com/stencil-hq/tern-sdk/go/tern
```

Go 1.26 or later. The only dependencies are `golang.org/x/term` and
`golang.org/x/sys`.

## Quick start

```go
package main

import (
	"context"
	"errors"
	"fmt"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

func main() {
	ctx := context.Background()
	deployed := false
	var sf *tern.Surface
	var view func() tern.View
	deploy := func(*tern.Action) { deployed = true; _ = sf.Render(view()) }
	view = func() tern.View {
		status := ui.Pending
		if deployed {
			status = ui.Done
		}
		return tern.Main(ui.Card{
			Head:   ui.T("Deploy"),
			Icon:   ui.IconRocket,
			Meta:   ui.T("api · v2.14.0"),
			Status: status,
			Children: ui.Nodes(
				ui.KV{Items: []ui.Pair{{K: ui.T("Service"), V: ui.T("api")}}},
				ui.Button{Text: "Deploy", Keys: []string{"enter"}, Variant: ui.ButtonPrimary,
					Common: ui.Common{OnClick: deploy}},
			),
		})
	}
	s, err := tern.Connect(ctx, tern.Options{})
	if errors.Is(err, tern.ErrUnsupported) {
		fmt.Print(tern.Plain(view(), 0)) // plain text outside Tern
		return
	}
	if err != nil {
		panic(err)
	}
	defer s.Close()
	// Apps use the sans face; machine strings (code, mono spans) stay mono.
	sf, _ = s.Open(tern.SurfaceOptions{Title: "deploy", Font: tern.FontSans})
	_ = sf.Render(view())
	_ = s.Run(ctx, func(in tern.Input) error {
		k, ok := in.(tern.Key)
		switch {
		case ok && k.Is("escape"):
			return tern.ErrStop
		case ok && k.Is("enter"):
			deploy(nil) // the key the button's keycap shows
		}
		return nil
	})
}
```

Handlers run inside `Run`, on the goroutine that reads input. For a one-shot
output that stays in the scrollback, use `tern.Print`.

Outside Tern, `tern.Plain` (and so `Print`'s fallback) prints a `row`'s
children on one line: build fallback-friendly views (cols of cards), or pass
`PrintOptions.Fallback` with text of your own.

Run the examples from this directory: `go run ./examples/table`,
`./examples/ask`, `./examples/progress`, `./examples/chat`,
`./examples/dashboard` (a sans `screen` app laid out as a page: head row,
flat stats strip, underline tabs over three views, each a full-width table
beside a detail card; confirm strip and key legend in the dock; live
updates).

## Design rules

Follow [Designing views](https://docs.stencil.so/tern/guides/design.html);
in short:

- Open apps with `Font: tern.FontSans`; mono is for machine strings (span
  tokens `mono`, `code`, `num`, `path`, `key`; `Column.Mono`).
- Use kinds, not `el` + CSS: `ui.Button` for actions, `ui.Table` with
  `Selectable` for lists of records, `ui.Stats` for headline figures (a
  flat strip directly in main, never in a card), `ui.Tabs` (underline) to
  navigate between views, `ui.TabsSegmented` for a toolbar filter, cards
  only for objects (a selected item's detail, a confirm strip).
- Pages: a head row (a `title` span, muted meta, badges right), the stats
  strip, tabs, then one view: a full-width table (one `Grow` text column,
  numbers `AlignEnd` + `Mono`) beside a detail card (`Min` 44ch, `Max`
  56ch). See the dashboard example.
- `Column.Priority` 0 (the default) hides first when a table is narrow:
  give the `Grow` column the highest priority, numbers next, sparks,
  meters and secondary columns the lowest.
- Color carries meaning only: tones and span tokens, never your own colors.
- One `ButtonPrimary` per view: with a detail card it sits in the card's
  `End` with its key and rows carry no buttons; without one, use a small
  `ui.ButtonCell` on rows. `ButtonDanger` only opens a confirm strip for
  destructive actions.
- Design all four states: loading, empty (`Table.Empty`, `List.Empty`),
  error, success.

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

## Packages

| Package | Holds |
| --- | --- |
| `tern` | Wire, input, keys, nodes, reconcile, session and helpers |
| `tern/ui` | A builder struct per kind (`ui.Card`, `ui.Table`, `ui.Button`, `ui.Stats`, …), spans, enums, `ui.Common` |
| `tern/el` | A builder struct per `el` tag (`el.Form`, `el.Button`, `el.Input`, …) |

## Layers

1. **Wire.** Constants (`ProtocolVersion`, `DefaultAPC`, `DefaultCredits`,
   `Kind*`, `TextKinds`, `Feature*`, `TermFeature*`), message bodies
   (`HelloQuery`, `OpenMsg`, `FrameMsg`, `SheetMsg`,
   `CloseMsg`), the twelve ops (`OpAdd` … `OpResume`), `Encode` (Tern's
   chunker, `Chunks`), `EncodeBlob`, `SplitMessage`, `DecodeEvent`. JSON is
   compact and absent fields are omitted.
2. **Input.** `Parser.Feed`/`Flush` yields `Item`s in order: pass-through
   key bytes, `*Reply`, `Event` or DA1. `KeyDecoder` turns key bytes into
   `Key{Name, Text, Ctrl, Alt, Shift, Meta}` (legacy xterm, kitty CSI u,
   bracketed paste); `key.Is("ctrl+c")` matches a chord.
3. **Nodes.** `tern.Node{Kind, Props, Children, Handlers}` is any kind with
   any props; `tern.Nodes` is a list (a `col`). The `ui` and `el` builders
   are structs whose zero fields stay off the wire; props whose default is
   not the zero value are pointers (`Value: new(0.5)`). Text is `ui.T("…")`
   or `ui.Spans(ui.Span{T: "…", S: "muted"})`. `ui.Common` holds the props
   every node takes, `Props` for untyped extras, and the handlers.
   `Node.Slots` are nodes a prop points at by id: `ui.Card` `HeadNode` and
   `End`, `ui.Section` `HeadNode`, `ui.Overlay` `End` and `Foot`, `ui.Rule`
   `End` and `ui.Message` `Avatar` become the children keyed `head`, `end`,
   `foot` and `avatar`, and the prop is that child's id (`main.<card>.end`).
4. **Reconcile.** `Reconciler.Diff(view)` returns the ops from the last
   view with ids derived from keys (`main.<key or index>…`), exactly as
   Tern's plugin worker; two siblings with one id return a
   `*DuplicateKeyError` and send nothing.
5. **Session.** `Connect` → `Session.Open` → `Surface.Render`, with
   `Stylesheet`, `Focus`, `Reveal`, `Scroll`, `Settle`,
   `Suspend`, `Resume`, `Send` (raw ops), `Close` (keep) and `Remove`.
   `Session.Blob`, `Session.HaveBlobs`, `Session.Caps`.
6. **Helpers.** `Print`, `Ask` (`*Answer`, nil when cancelled) and
   `Plain(view, cols)`.

### Views

`View{Main, Dock, Layer}`; a nil region is absent. Tern draws a region
root's children straight into the region and ignores the root's own kind
and props, so give a region a list: a `tern.Nodes{…}` (or `ui.Nodes(…)`)
is wrapped in a `col`. A single node given as `Main`, `Dock` or `Layer` is
the region root itself and should be a container. `tern.Main(a, b)` is the
shorthand for a view whose `main` holds `a` and `b`.

### Flow control

At most `credits` frames (from the hello reply) are unacknowledged. A
`Render` while blocked only stores the view; queued view ops wait too.
When an ack returns a credit, one frame carries the difference from the
last view sent, then the queued ops. A `NoListen` surface sends at once.
`Surface.Close` (and `Session.Close`) first sends what is still pending as
one last frame, credit or not, then `x`, so the final view is the one left
on screen.

Frame state and sequence numbers commit only after successful writes; failed
frames retain their view and queued ops for retry. Ack-triggered write failures
are returned by `Next`/`Run`. `Adopt` preserves a previously closed surface's
last-sent main region and sequence; unknown adopted ids delete `main` first.

## Handlers

Set handlers on a node through `ui.Common` (or `tern.Handlers` on a
`tern.Node`):

```go
ui.Button{Text: "Create", Common: ui.Common{OnClick: func(a *tern.Action) { … }}}
ui.Table{Common: ui.Common{OnSelect: func(e *tern.Select) { pick(e.Item) },
	OnAction: map[string]func(*tern.Action){"retry": func(a *tern.Action) { retry(a.Item) }}},
	Selectable: true, Selected: picked, Cols: cols, Rows: rows}
ui.List{CommonWithoutMax: ui.CommonWithoutMax{OnSelect: func(e *tern.Select) { … }}, Children: items}
ui.Editor{Common: ui.Common{Key: "ed", OnSend: func(e *tern.Send) { … }}}
```

- `OnClick` / `OnDblClick` set `actions.click` / `actions.dblclick` to
  `"click"` / `"dblclick"` unless the node names that action itself.
- `OnMenu` adds its names (sorted) to `actions.menu` after the node's own.
- `OnAction` runs on `action` events by `act` (or `act=value` when the
  value is a string; a table's sort, an otp's cell or a questionnaire's
  answers arrive in `Action.Value` as decoded JSON).
- `OnToggle`, `OnSelect`, `OnActivate`, `OnChange`, `OnFocus`, `OnEdit`,
  `OnUndo`, `OnSend` run on those events.

An event is routed by its `sf` and `id` to the node with that id in the
view last rendered (a list's `select`/`activate` name the list). Handlers
run on the goroutine that reads input, inside `Session.Next` or
`Session.Run`; a handled event is consumed, every other key and event is
returned to the program. Acks are consumed by the session; `resize`,
`theme` and `motion` update `Session.Caps` and are returned too.

## Sessions

```go
s, err := tern.Connect(ctx, tern.Options{App: "deploy", Features: []string{tern.FeatureSend}})
if errors.Is(err, tern.ErrUnsupported) { /* plain output */ }
defer s.Close()
sf, _ := s.Open(tern.SurfaceOptions{Mode: tern.Inline})
_ = sf.Render(view)
err = s.Run(ctx, func(in tern.Input) error {
	switch in := in.(type) {
	case tern.Key:      // keys, Ctrl+C included (raw mode)
	case *tern.Resize:  // and the other unhandled events
	}
	return nil // tern.ErrStop ends Run
})
```

- `Connect` returns `ErrUnsupported` when `TERN_TSP=0`, stdin or stdout is
  not a terminal, inside screen/zellij, inside tmux unless Tern draws the
  session, or when DA1 answers before hello or nothing answers within
  `Timeout` (1 s). It switches the tty to raw mode before writing hello and
  restores it on failure. Inside tmux it sends hello without DA1 and turns
  the pane's `allow-set-title` off until it closes (`RunTmux` replaces the
  `tmux` command).
- After connecting it enables bracketed paste and pushes kitty keyboard
  flag 1 (`NoPaste`, `NoKitty` turn them off).
- The session's methods are safe for concurrent use: `Render` may run on
  one goroutine while another reads input. Only one goroutine at a time
  should call `Next`/`Run`.
- `Close` closes open surfaces (kept unless opened with `Discard`), undoes
  the modes, drains input for 50 ms and restores the tty. It also runs on
  SIGINT, SIGTERM and SIGHUP (then the signal is raised again).
- `Options.In`, `Options.Out` and `Options.MakeRaw` make the session
  drivable over pipes, as the tests do with a scripted terminal.
  Non-pollable custom input must supply `CancelRead` or implement
  `io.ReadCloser` (the session closes it). Close joins the actual reader,
  so no background pump consumes fallback input. Native terminal input is
  polled without closing stdin, including record-based Windows console input.
- Terminal ownership and signal cleanup are armed before raw mode; only one
  session may own the process tty. Setup failures and write failures still
  reset modes and restore the tty.
- Input flushes after 30 ms of inactivity independently of `Next` deadlines.
  Incomplete key sequences are dropped; pastes wait for their closing marker.

## Helpers

```go
tern.Print(ctx, view, tern.PrintOptions{CSS: ".size{text-align:right}"})
answer, err := tern.Ask(ctx, tern.Main(form), tern.AskOptions{Font: tern.FontSans}) // Submit defaults to "submit"
fmt.Print(tern.Plain(view, 80))
```

`Print` writes `Plain(view)` (or `Fallback`) without TSP. `Ask` returns
`ErrUnsupported` without TSP, and a nil answer on Escape, Ctrl+C or
Ctrl+D; the form stays in the scrollback.
A submit node's handler runs before its answer is returned, even when handled.

Typed kind-specific props replace colliding common props: `Tool` and `Prefs`
embed `CommonWithoutTitle`, `List` embeds `CommonWithoutMax`, and `Picker`
embeds `PickerCommon` and takes `Actions []PickerAction`. Agent statuses use
`AgentState` (pending/running/done/failed/aborted/idle/parked), separately from
card/tool statuses. Heatmap cells are nullable numbers. Table meter cells
encode as `{meter: {...}}`. Plain output removes terminal escapes, aligns
display cells (including CJK/combining marks), and prints card statuses in
parentheses.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every TSP message as JSONL (`surface-play` replays it) |
| `TMUX`, `TMUX_PANE` | Inside tmux: TSP only when Tern draws the session (`@tern-tsp`), with the pane's `allow-set-title` off while connected |
| `STY`, `ZELLIJ` | Inside screen or zellij: no TSP |

## Testing

```sh
go test ./... && go test -race ./... && go vet ./... && gofmt -l .
GOOS=windows go vet ./...
```
