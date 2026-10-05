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

	tern "github.com/stencil-hq/tern-sdk/go"
	"github.com/stencil-hq/tern-sdk/go/el"
	"github.com/stencil-hq/tern-sdk/go/ui"
)

// css is the form's stylesheet from the protocol book.
const css = `.ask{display:flex;flex-direction:column;gap:8px;max-width:460px;padding:10px 12px;border-radius:8px;background:var(--card);box-shadow:inset 0 0 0 1px var(--l2)}
.ask p{margin:0}
.ask .sizes{display:flex;gap:14px}
.ask :checked+span{color:var(--accent)}
.ask button{align-self:flex-start;padding:3px 12px;border-radius:6px;background:var(--accent-fill);color:#fff}`

// size is one radio of the form.
func size(key, value, label string) tern.Element {
	return el.Label{Common: ui.Common{Key: key}, Children: ui.Nodes(
		el.Input{Type: el.Radio, Name: "size", Value: value, Common: ui.Common{Key: "r"}},
		el.Span{Common: ui.Common{Key: "t"}, Text: label},
	)}
}

func main() {
	form := el.Form{Class: "ask", Common: ui.Common{Key: "ask"}, Children: ui.Nodes(
		el.P{Common: ui.Common{Key: "q"}, Text: "Which size?"},
		el.Div{Class: "sizes", Common: ui.Common{Key: "sizes"}, Children: ui.Nodes(
			size("s", "s", "Small"), size("m", "m", "Medium"), size("l", "l", "Large"),
		)},
		el.Button{Text: "Create", Common: ui.Common{Key: "go", Actions: &ui.Actions{Click: "submit"}}},
	)}
	answer, err := tern.Ask(context.Background(), tern.View{Main: tern.Nodes{form}}, tern.AskOptions{CSS: css})
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
