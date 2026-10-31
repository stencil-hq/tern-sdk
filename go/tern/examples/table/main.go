// Command table prints the current directory as a table that stays in the
// scrollback, with sizes right-aligned in the mono face; plain text outside
// Tern.
package main

import (
	"context"
	"fmt"
	"os"

	"github.com/stencil-hq/tern-sdk/go/tern"
	"github.com/stencil-hq/tern-sdk/go/tern/ui"
)

func main() {
	entries, err := os.ReadDir(".")
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	view := directoryView(entries)
	if err := tern.Print(context.Background(), view, tern.PrintOptions{Font: tern.FontSans}); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

// directoryView is one table row per entry, keyed by its name.
func directoryView(entries []os.DirEntry) tern.View {
	table := ui.Table{
		Common: ui.Common{Key: "ls"},
		Cols: []ui.Column{
			{ID: "name", Head: ui.T("Name"), Grow: 1, Truncate: ui.TruncateMiddle, Priority: 2},
			{ID: "size", Head: ui.T("Size"), Align: ui.AlignEnd, Mono: true, Priority: 1},
		},
		Empty: ui.T("Empty directory"),
	}
	for _, e := range entries {
		name, icon, tone, size := e.Name(), ui.IconFile, ui.Tone(""), ""
		if e.IsDir() {
			name, icon, tone = name+"/", ui.IconFolder, ui.ToneAccent
		} else if info, err := e.Info(); err == nil {
			size = human(info.Size())
		}
		table.Rows = append(table.Rows, ui.TableRow{ID: e.Name(), Cells: map[string]ui.Cell{
			"name": ui.TextCell{Text: ui.T(name), Icon: icon, Tone: tone},
			"size": ui.T(size),
		}})
	}
	return tern.Main(table)
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
