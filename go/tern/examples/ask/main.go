// Command ask asks "Which size?" with a form in a flow surface and prints
// the answer; outside Tern it asks on the terminal instead.
package main

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"os"
	"strings"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/el"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

// size is one radio of the form.
func size(key, value, label string) tern.Element {
	return el.Label{Common: ui.Common{Key: key}, Children: ui.Nodes(
		el.Input{Type: el.Radio, Name: "size", Value: value, Common: ui.Common{Key: "r"}},
		el.Span{Common: ui.Common{Key: "t"}, Text: label},
	)}
}

func main() {
	form := el.Form{Common: ui.Common{Key: "ask"}, Children: ui.Nodes(
		ui.Card{Common: ui.Common{Key: "q", Max: &ui.Bounds{W: ui.Ch(48)}}, Head: ui.T("Which size?"),
			Children: ui.Nodes(
				ui.Row{Common: ui.Common{Key: "sizes"}, Gap: ui.GapLG, Children: ui.Nodes(
					size("s", "s", "Small"), size("m", "m", "Medium"), size("l", "l", "Large"),
				)},
				ui.Button{Common: ui.Common{Key: "go", Actions: &ui.Actions{Click: "submit"}},
					Text: "Create", Variant: ui.ButtonPrimary},
			)},
	)}
	answer, err := tern.Ask(context.Background(), tern.View{Main: tern.Nodes{form}},
		tern.AskOptions{Font: tern.FontSans})
	switch {
	case errors.Is(err, tern.ErrUnsupported):
		fmt.Print("Which size? [s/m/l]: ")
		line, _ := bufio.NewReader(os.Stdin).ReadString('\n')
		if line = strings.TrimSpace(line); line == "" {
			fmt.Println("cancelled")
			os.Exit(1)
		}
		fmt.Println("size:", line)
	case err != nil:
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	case answer == nil:
		fmt.Println("cancelled")
		os.Exit(1)
	default:
		picked := answer.Values.String("size")
		if picked == "" {
			picked = "none"
		}
		fmt.Println("size:", picked)
	}
}
