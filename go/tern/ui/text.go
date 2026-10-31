package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

// Text is styled spans, wrapped or truncated (kind text).
type Text struct {
	Common
	// Spans are the styled runs; they win over Text.
	Spans []Span `json:"spans,omitzero"`
	// Text is plain text, one unstyled span.
	Text string `json:"text,omitzero"`
	// Wrap is how the text wraps (default word).
	Wrap Wrap `json:"wrap,omitzero"`
	// Truncate is where an overflowing text is cut (default end).
	Truncate Truncate `json:"truncate,omitzero"`
	// Lines clamps to this many visual lines (0 is no clamp).
	Lines int `json:"lines,omitzero"`
	// Measure "prose" caps the line length at 80 cells.
	Measure string `json:"measure,omitzero"`
	// Cite is how cited spans look (default CiteChip).
	Cite CiteStyle `json:"cite,omitzero"`
	// Sources draws the numbered sources list under the text (default true
	// for CiteNote); a fitted text never draws it.
	Sources *bool `json:"sources,omitzero"`
}

// MeasureProse caps a text's line length for readable paragraphs.
const MeasureProse = "prose"

// CiteStyle is how a text's cited spans look.
type CiteStyle string

// Citation looks.
const (
	// CiteChip is a numbered accent chip after the run, one per source.
	CiteChip CiteStyle = "chip"
	// CitePill is the first source's site in a pill, +N for more.
	CitePill CiteStyle = "pill"
	// CiteNote underlines the run in dots with superscript numbers after
	// it, and lists the sources under the text.
	CiteNote CiteStyle = "note"
)

// Node builds the text.
func (t Text) Node() tern.Node { return Build(tern.KindText, t, nil) }

// Md is Markdown (kind md).
type Md struct {
	Common
	// Text is the Markdown source.
	Text string `json:"text,omitzero"`
	// Stream says the source is still arriving.
	Stream bool `json:"stream,omitzero"`
	// Marks are spans drawn in place of their literal text in prose.
	Marks []Span `json:"marks,omitzero"`
}

// Node builds the md.
func (m Md) Node() tern.Node { return Build(tern.KindMd, m, nil) }

// Code is highlighted code (kind code).
type Code struct {
	Common
	// Text is the code.
	Text string `json:"text,omitzero"`
	// Lang is the grammar by language name or extension.
	Lang string `json:"lang,omitzero"`
	// Path picks the grammar when Lang is absent and shows a file header.
	Path string `json:"path,omitzero"`
	// Numbers shows the line number gutter.
	Numbers bool `json:"numbers,omitzero"`
	// Start is the first line's number (default 1).
	Start *int `json:"start,omitzero"`
	// Nums is each line's number, strictly ascending, in place of counting
	// from Start; a … row is drawn where the numbers jump.
	Nums []int `json:"nums,omitzero"`
	// Marks are marked lines.
	Marks []CodeMark `json:"marks,omitzero"`
	// Wrap soft-wraps long lines.
	Wrap bool `json:"wrap,omitzero"`
	// Folds are runs of lines drawn folded: sorted, not overlapping; marks
	// inside apply once a fold is open.
	Folds []CodeFold `json:"folds,omitzero"`
	// Overview shows a strip above the lines: one bar per line (its height
	// the mark's heat, its color the mark's tone), a band over the lines
	// outside folds and a bracket over the lines in view. A click or drag on
	// it scrolls there.
	Overview bool `json:"overview,omitzero"`
}

// CodeFold is a run of lines a code block draws as one row until opened.
type CodeFold struct {
	// From and To are the first and last folded line, in the gutter's numbering.
	From int `json:"from"`
	To   int `json:"to"`
	// Label is the folded row's text, in the code's face (default "… [lines {from}-{to}]",
	// "… [line {from}]" for one line).
	Label string `json:"label,omitzero"`
	// Note says what the fold hides, after a sparkline of its lines' heat.
	Note Rich `json:"note,omitzero"`
	// Tone error tints the row; any tone colors the note.
	Tone Tone `json:"tone,omitzero"`
	// Open draws the fold open; a click on the folded row (or an open
	// fold's head) toggles it in place.
	Open bool `json:"open,omitzero"`
	// Hovercard is a card shown while the pointer rests on the folded row.
	Hovercard Hovercard `json:"hovercard,omitzero"`
}

// CodeMark marks a line of code, with stronger tinted byte ranges.
type CodeMark struct {
	// Line counts in the gutter's numbering.
	Line int `json:"line"`
	// Tone is the bar and tint (default accent, none when Heat or Wash is set).
	Tone Tone `json:"tone,omitzero"`
	// Heat is a score 0–1 on the heat ramp: the overview's bar, the fold
	// sparkline, and the row's wash unless Wash is set.
	Heat *float64 `json:"heat,omitzero"`
	// Ranges are [start, end) byte ranges in the line.
	Ranges [][2]int `json:"ranges,omitzero"`
	// Wash is scores 0–1 washing the row on the heat ramp: one flat,
	// several a gradient across it, left to right; in place of Heat's wash.
	Wash []float64 `json:"wash,omitzero"`
}

// Node builds the code.
func (c Code) Node() tern.Node { return Build(tern.KindCode, c, nil) }

// Diff is a unified or split diff (kind diff).
type Diff struct {
	Common
	// Text is a unified diff.
	Text string `json:"text,omitzero"`
	// Hunks are hunks given directly; they win over Text.
	Hunks []Hunk `json:"hunks,omitzero"`
	// Path picks the grammar when Lang is absent.
	Path string `json:"path,omitzero"`
	// Lang is the grammar by language name.
	Lang string `json:"lang,omitzero"`
	// Mode is unified, split or auto (the user's setting).
	Mode DiffMode `json:"mode,omitzero"`
}

// Hunk is one diff hunk: lines starting with +, - or a space.
type Hunk struct {
	OldStart *int     `json:"oldStart,omitzero"`
	NewStart *int     `json:"newStart,omitzero"`
	Lines    []string `json:"lines"`
}

// DiffMode is how a diff lays out.
type DiffMode string

// Diff modes.
const (
	DiffUnified DiffMode = "unified"
	DiffSplit   DiffMode = "split"
	DiffAuto    DiffMode = "auto"
)

// Node builds the diff.
func (d Diff) Node() tern.Node { return Build(tern.KindDiff, d, nil) }

// Ansi is a small terminal fed escape sequences (kind ansi).
type Ansi struct {
	Common
	// Text is the output, with escape sequences.
	Text string `json:"text,omitzero"`
	// Cols is the most columns.
	Cols int `json:"cols,omitzero"`
	// Preview clamps to PreviewLines(n) under a fade.
	Preview Preview `json:"preview,omitzero"`
	// Follow keeps the tail in view.
	Follow bool `json:"follow,omitzero"`
}

// Node builds the ansi.
func (a Ansi) Node() tern.Node { return Build(tern.KindAnsi, a, nil) }

// Rows are pre-rendered ANSI rows, a migration fallback (kind rows).
type Rows struct {
	Common
	// Lines are the rows, with escape sequences.
	Lines []string `json:"lines,omitzero"`
	// Cols is the exact width in columns (default 80).
	Cols int `json:"cols,omitzero"`
}

// Node builds the rows.
func (r Rows) Node() tern.Node { return Build(tern.KindRows, r, nil) }

// Math is TeX math (kind math).
type Math struct {
	Common
	// Text is the TeX, without delimiters.
	Text string `json:"text,omitzero"`
	// Display is display style, a centered block.
	Display bool `json:"display,omitzero"`
}

// Node builds the math.
func (m Math) Node() tern.Node { return Build(tern.KindMath, m, nil) }
