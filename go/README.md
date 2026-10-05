# Tern SDK for Go

`github.com/stencil-hq/tern-sdk/go` (package `tern`) lets a Go program running
in a terminal pane talk to Tern over the
[Tern Surface Protocol](https://docs.stencil.so/tern/protocol/index.html):
it describes its UI as a tree of nodes, and Tern draws it natively. Outside
Tern the same program falls back to plain text.

## Install

```sh
go get github.com/stencil-hq/tern-sdk/go
```

Go 1.26 or later. The only dependencies are `golang.org/x/term` and
`golang.org/x/sys`.

## Quick start

```go
package main

import (
	"context"

	tern "github.com/stencil-hq/tern-sdk/go"
	"github.com/stencil-hq/tern-sdk/go/ui"
)

func main() {
	view := tern.Main(ui.Card{
		Head:     ui.T("Deploy"),
		Status:   ui.Done,
		Children: ui.Nodes(ui.KV{Items: []ui.Pair{{K: ui.T("service"), V: ui.T("api")}}}),
	})
	// A native card kept in the scrollback in Tern; plain text anywhere else.
	_ = tern.Print(context.Background(), view, tern.PrintOptions{})
}
```

Run the examples from this directory: `go run ./examples/table`,
`./examples/ask`, `./examples/progress`, `./examples/chat`.

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

## Packages

| Package | Holds |
| --- | --- |
| `tern` | Wire, input, keys, nodes, reconcile, session and helpers |
| `tern/ui` | A builder struct per kind (`ui.Card`, `ui.Table`, `ui.Editor`, …), spans, enums, `ui.Common` |
| `tern/el` | A builder struct per `el` tag (`el.Form`, `el.Button`, `el.Input`, …) |

## Layers

1. **Wire.** Constants (`ProtocolVersion`, `DefaultAPC`, `DefaultCredits`,
   `Kind*`, `TextKinds`, `Feature*`, `TermFeature*`), message bodies
   (`HelloQuery`, `OpenMsg`, `FrameMsg`, `SheetMsg`, `PaletteMsg`,
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
4. **Reconcile.** `Reconciler.Diff(view)` returns the ops from the last
   view with ids derived from keys (`main.<key or index>…`), exactly as
   Tern's plugin worker; two siblings with one id return a
   `*DuplicateKeyError` and send nothing.
5. **Session.** `Connect` → `Session.Open` → `Surface.Render`, with
   `Stylesheet`, `Palette`, `Focus`, `Reveal`, `Scroll`, `Settle`,
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
el.Button{Text: "Create", Common: ui.Common{OnClick: func(a *tern.Action) { … }}}
ui.List{CommonWithoutMax: ui.CommonWithoutMax{OnSelect: func(e *tern.Select) { … }}, Children: items}
ui.Editor{Common: ui.Common{Key: "ed", OnSend: func(e *tern.Send) { … }}}
```

- `OnClick` / `OnDblClick` set `actions.click` / `actions.dblclick` to
  `"click"` / `"dblclick"` unless the node names that action itself.
- `OnMenu` adds its names (sorted) to `actions.menu` after the node's own.
- `OnAction` runs on `action` events by `act` (or `act=value`).
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
  not a terminal, inside tmux/screen/zellij, or when DA1 answers before
  hello or nothing answers within `Timeout` (1 s). It switches the tty to
  raw mode before writing hello and restores it on failure.
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
answer, err := tern.Ask(ctx, tern.Main(form), tern.AskOptions{CSS: css}) // Submit defaults to "submit"
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
| `TMUX`, `STY`, `ZELLIJ` | Inside a multiplexer: no TSP |

## Testing

```sh
go test ./... && go test -race ./... && go vet ./... && gofmt -l .
GOOS=windows go vet ./...
```
