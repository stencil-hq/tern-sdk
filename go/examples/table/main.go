// Command table prints the current directory as a table that stays in the
// scrollback, with sizes right-aligned by a stylesheet; plain text outside
// Tern.
package main

import (
	"context"
	"fmt"
	"os"

	tern "github.com/stencil-hq/tern-sdk/go"
	"github.com/stencil-hq/tern-sdk/go/el"
	"github.com/stencil-hq/tern-sdk/go/ui"
)

// css lays the table out and right-aligns the size column.
const css = `.ls{border-collapse:collapse;align-self:flex-start}
.ls th{text-align:left;color:var(--t3);font-weight:600;padding:0 16px 2px 0}
.ls td{padding:0 16px 0 0}
.ls .size{text-align:right}
.ls .dir{color:var(--accent)}`

func main() {
	entries, err := os.ReadDir(".")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	view := directoryView(entries)
	if err := tern.Print(context.Background(), view, tern.PrintOptions{CSS: css}); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// directoryView keeps the header and file rows in separate key namespaces.
func directoryView(entries []os.DirEntry) tern.View {
	rows := ui.Nodes(el.Tr{Common: ui.Common{Key: "head"},
		Children: ui.Nodes(el.Th{Text: "Name"}, el.Th{Class: "size", Text: "Size"})})
	for _, e := range entries {
		name, size, class := e.Name(), "", ""
		if e.IsDir() {
			name, class = name+"/", "dir"
		} else if info, err := e.Info(); err == nil {
			size = human(info.Size())
		}
		rows = append(rows, el.Tr{Common: ui.Common{Key: "entry:" + e.Name()}, Children: ui.Nodes(
			el.Td{Class: class, Text: name},
			el.Td{Class: "size", Text: size},
		)})
	}
	return tern.View{Main: tern.Nodes{el.Table{Class: "ls", Children: rows}}}
}

// human is a byte count as 312, 2.1K or 14M.
func human(n int64) string {
	const units = "KMGTPE"
	if n < 1024 {
		return fmt.Sprint(n)
	}
	f, i := float64(n)/1024, 0
	for f >= 1024 && i < len(units)-1 {
		f /= 1024
		i++
	}
	if f < 10 {
		return fmt.Sprintf("%.1f%c", f, units[i])
	}
	return fmt.Sprintf("%.0f%c", f, units[i])
}
