package tern_test

import (
	"context"
	"errors"
	"fmt"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/el"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// Print shows a view in Tern, or its plain text elsewhere (here: a test's
// stdout is not a terminal).
func ExamplePrint() {
	view := tern.Main(ui.Card{
		Head:     ui.T("Deploy"),
		Status:   ui.Done,
		Children: ui.Nodes(ui.KV{Items: []ui.Pair{{K: ui.T("service"), V: ui.T("api")}, {K: ui.T("region"), V: ui.T("eu-west-1")}}}),
	})
	if err := tern.Print(context.Background(), view, tern.PrintOptions{}); err != nil {
		fmt.Println(err)
	}
	// Output:
	// Deploy (done)
	//   service  api
	//   region   eu-west-1
}

// Ask returns ErrUnsupported without TSP, so the program asks its own way.
func ExampleAsk() {
	form := el.Form{Children: ui.Nodes(
		el.P{Text: "Which size?"},
		el.Label{Children: ui.Nodes(el.Input{Type: el.Radio, Name: "size", Value: "s"}, el.Span{Text: "Small"})},
		el.Button{Text: "Create", Common: ui.Common{Actions: &ui.Actions{Click: "submit"}}},
	)}
	answer, err := tern.Ask(context.Background(), tern.Main(form), tern.AskOptions{})
	switch {
	case errors.Is(err, tern.ErrUnsupported):
		fmt.Println("no TSP: ask on stdin")
	case answer != nil:
		fmt.Println("size:", answer.Values.String("size"))
	}
	// Output: no TSP: ask on stdin
}

// A session renders views into frames and reads keys and events.
func ExampleConnect() {
	ctx := context.Background()
	s, err := tern.Connect(ctx, tern.Options{App: "demo"})
	if err != nil {
		fmt.Println(err)
		return
	}
	defer s.Close()
	sf, _ := s.Open(tern.SurfaceOptions{Mode: tern.Flow})
	count := 0
	render := func() { _ = sf.Render(tern.Main(ui.Text{Text: fmt.Sprint("clicked ", count)})) }
	render()
	_ = s.Run(ctx, func(in tern.Input) error {
		if k, ok := in.(tern.Key); ok && k.Is("escape") {
			return tern.ErrStop
		}
		count++
		render()
		return nil
	})
	// Output: tern: TSP is not available
}

// A reconciler diffs views into frame ops with ids derived from keys.
func ExampleReconciler() {
	r := tern.NewReconciler("s1")
	_, _ = r.Diff(tern.Main(ui.Md{Text: "Checking", Common: ui.Common{Key: "m"}}))
	ops, _ := r.Diff(tern.Main(ui.Md{Text: "Checking the build", Common: ui.Common{Key: "m"}}, ui.Rule{}))
	b, _ := tern.Marshal(ops)
	fmt.Println(string(b))
	// Output: [["add","main.1","main",null,{"id":"main.1","k":"rule"}],["text","main.m","append"," the build"]]
}

// Handlers attach to nodes; the click action is named for them.
func ExampleHandlers() {
	r := tern.NewReconciler("s1")
	button := el.Button{Text: "Go", Common: ui.Common{Key: "go", OnClick: func(a *tern.Action) {}}}
	ops, _ := r.Diff(tern.Main(button))
	b, _ := tern.Marshal(ops)
	fmt.Println(string(b))
	// Output: [["add","main","s1",null,{"id":"main","k":"col","c":[{"id":"main.go","k":"el","p":{"actions":{"click":"click"},"key":"go","tag":"button","text":"Go"}}]}]]
}

// The input parser splits TSP events out of keys; the key decoder names keys.
func ExampleParser() {
	var p tern.Parser
	var d tern.KeyDecoder
	for _, it := range p.Feed([]byte("a\x1b_tsp;e;{\"ev\":\"ack\",\"sf\":\"s1\",\"s\":3}\x1b\\\x1b[1;5C")) {
		switch {
		case it.Keys != nil:
			for _, k := range d.Feed(it.Keys) {
				fmt.Println("key", k)
			}
		case it.Event != nil:
			if ack, ok := it.Event.(*tern.Ack); ok {
				fmt.Println("ack", ack.S)
			}
		}
	}
	// Output:
	// key a
	// ack 3
	// key ctrl+right
}

// Plain renders a view as text for output outside Tern.
func ExamplePlain() {
	fmt.Print(tern.Plain(tern.Main(
		ui.Progress{Value: new(0.25), Label: ui.T("uploading")},
		ui.List{Children: ui.Nodes(ui.Item{Label: ui.T("api")}, ui.Item{Label: ui.T("web")})},
	), 0))
	// Output:
	// [#####---------------] 25% uploading
	// - api
	// - web
}
