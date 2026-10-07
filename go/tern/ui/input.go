package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

// Field holds the props editor and input share. The program owns the
// text: answer keys with new Text and Cursor.
type Field struct {
	// Text is the field's text.
	Text string `json:"text,omitzero"`
	// Cursor is the caret in UTF-16 units (default the end).
	Cursor *int `json:"cursor,omitzero"`
	// Anchor is the selection anchor in UTF-16 units.
	Anchor *int `json:"anchor,omitzero"`
	// Decor are styled ranges of the text.
	Decor []Decor `json:"decor,omitzero"`
	// Ghost is an inline completion drawn dim after the caret.
	Ghost string `json:"ghost,omitzero"`
	// Placeholder shows while Text is empty.
	Placeholder Rich `json:"placeholder,omitzero"`
	// Prompt are spans before the first line.
	Prompt []Span `json:"prompt,omitzero"`
	// Mode is a vim mode label; it turns native editing off.
	Mode string `json:"mode,omitzero"`
	// Lang highlights the text as code.
	Lang string `json:"lang,omitzero"`
	// Readonly makes the field not editable.
	Readonly bool `json:"readonly,omitzero"`
	// Sendable says the owner accepts a send event now.
	Sendable bool `json:"sendable,omitzero"`
}

// Decor styles UTF-16 range [From, To) of a field's text.
type Decor struct {
	From int    `json:"from"`
	To   int    `json:"to"`
	S    string `json:"s"`
	Fx   Fx     `json:"fx,omitzero"`
}

// Editor is a multi-line text field with a caret (kind editor).
type Editor struct {
	Common
	Field
	// MaxLines caps the height at n lines and scrolls inside.
	MaxLines int `json:"maxLines,omitzero"`
}

// Node builds the editor.
func (e Editor) Node() tern.Node { return Build(tern.KindEditor, e, nil) }

// Input is a single-line text field (kind input).
type Input struct {
	Common
	Field
}

// Node builds the input.
func (i Input) Node() tern.Node { return Build(tern.KindInput, i, nil) }
