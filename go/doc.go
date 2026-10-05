// Package tern lets a program running in a terminal pane talk to Tern over
// the Tern Surface Protocol (TSP): it describes its UI as a tree of nodes,
// and Tern lays it out, draws and animates it natively. Outside Tern the
// same program falls back to plain text.
//
// The package has six layers, each usable without the ones above it:
//
//   - Wire: constants, messages in both directions, Encode (framing and
//     chunking), EncodeBlob, SplitMessage and DecodeEvent.
//   - Input: Parser splits TSP replies, events and DA1 answers out of the
//     pty's input; KeyDecoder turns the rest into Keys.
//   - Nodes: Node, Nodes and View, with Handlers attached to nodes. Typed
//     builders for every kind live in package ui, and for el tags in
//     package el.
//   - Reconcile: Reconciler diffs views into frame ops with ids derived
//     from keys, exactly as Tern's plugin worker does.
//   - Session: Connect does the hello handshake in raw mode; Session.Open
//     opens surfaces whose Render keeps to credit-based flow control;
//     Session.Next and Session.Run read keys and unhandled events, running
//     node handlers on the reading goroutine; Session.Close restores the
//     terminal.
//   - Helpers: Print shows a static view kept in the scrollback, Ask shows
//     a form and returns its answer, Plain renders a view as text.
//
// A minimal program:
//
//	s, err := tern.Connect(ctx, tern.Options{})
//	if errors.Is(err, tern.ErrUnsupported) {
//		fmt.Print(tern.Plain(view, 0))
//		return
//	}
//	defer s.Close()
//	sf, _ := s.Open(tern.SurfaceOptions{Mode: tern.Flow})
//	_ = sf.Render(tern.Main(ui.Card{Head: ui.T("Deploy"), Status: ui.Running}))
//	_ = s.Run(ctx, func(in tern.Input) error {
//		if k, ok := in.(tern.Key); ok && k.Is("escape") {
//			return tern.ErrStop
//		}
//		return nil
//	})
//
// Environment: TERN_TSP=0 never speaks TSP; TERN_TSP_RECORD=<file> appends
// every TSP message as JSONL for Tern's surface-play; TMUX, STY and ZELLIJ
// turn TSP off (multiplexers swallow it).
package tern
