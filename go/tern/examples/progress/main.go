// Command progress shows a running card in a flow surface: a spinner and a
// timer, log lines streamed into it and a progress bar, then done. Outside
// Tern it prints the log lines.
package main

import (
	"context"
	"errors"
	"fmt"
	"os"
	"strings"
	"time"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// steps are the work the example pretends to do.
var steps = []string{
	"Resolving dependencies", "Fetching serde v1.0.228", "Fetching tokio v1.47.1",
	"Compiling proc-macro2", "Compiling serde", "Compiling tokio", "Compiling stencil-css",
	"Compiling stencil-term", "Linking tern", "Running 42 tests",
}

// view is the card for the log so far; done ends it.
func view(log []string, done bool, took time.Duration) tern.View {
	value := float64(len(log)) / float64(len(steps))
	status, head := ui.Running, ui.Row{Gap: ui.GapSM, Children: ui.Nodes(
		ui.Spinner{Style: ui.SpinnerDots, Label: ui.T("Building"), Common: ui.Common{Key: "spin"}},
		ui.Elapsed{Common: ui.Common{Key: "time"}},
	)}
	if done {
		status = ui.Done
		head = ui.Row{Children: ui.Nodes(ui.Text{Common: ui.Common{Key: "done"},
			Spans: []ui.Span{{T: "Finished", S: "success"}, {T: fmt.Sprintf(" in %.1fs", took.Seconds()), S: "muted"}}})}
	}
	return tern.View{Main: tern.Nodes{ui.Card{
		Head:   ui.T("cargo build"),
		Status: status,
		Common: ui.Common{Key: "build"},
		Children: ui.Nodes(
			head,
			ui.Ansi{Text: strings.Join(log, "\n"), Common: ui.Common{Key: "log"}},
			ui.Progress{Value: &value, Label: ui.T(fmt.Sprintf("%d/%d", len(log), len(steps))), Common: ui.Common{Key: "bar"}},
		),
	}}}
}

func main() {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	s, err := tern.Connect(ctx, tern.Options{})
	if errors.Is(err, tern.ErrUnsupported) {
		for _, line := range steps {
			time.Sleep(150 * time.Millisecond)
			fmt.Println(line)
		}
		fmt.Println("done")
		return
	}
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	defer s.Close()
	sf, err := s.Open(tern.SurfaceOptions{Mode: tern.Flow})
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		return
	}
	go func() {
		_ = s.Run(ctx, func(in tern.Input) error {
			if k, ok := in.(tern.Key); ok && (k.Is("ctrl+c") || k.Is("escape")) {
				cancel()
				return tern.ErrStop
			}
			return nil
		})
	}()
	start := time.Now()
	var log []string
	_ = sf.Render(view(log, false, 0))
	for _, line := range steps {
		select {
		case <-ctx.Done():
			return
		case <-time.After(300 * time.Millisecond):
		}
		log = append(log, line)
		_ = sf.Render(view(log, false, 0))
	}
	_ = sf.Render(view(log, true, time.Since(start)))
}
