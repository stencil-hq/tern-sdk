// Package ui has typed builders for every TSP node kind but block (Tern's
// own) and el (package el): one struct per kind whose fields are the
// kind's documented props, embedding Common (or a collision-free variant)
// for shared props and event handlers. A zero field is left out of the wire; props whose
// default is not the zero value are pointers (Go 1.26's new(v) fills them).
//
//	ui.Card{Head: ui.T("Deploy"), Status: ui.Running, Collapsible: true,
//		Children: ui.Nodes(ui.Md{Text: "Uploading **api**"})}
//
// Enumerations are typed strings with constants; a value newer than the SDK
// converts directly (ui.Tone("brand")). Common.Props sets any prop untyped.
package ui

import (
	"bytes"
	"encoding/json"
	"strconv"

	"github.com/stencil-hq/tern-sdk/go/tern"
)

// Nodes lists elements, as children or as a view region (a col).
func Nodes(els ...tern.Element) tern.Nodes { return els }

// Common holds the props every node takes and the node's event handlers.
type Common struct {
	// Key is the node's identity among its siblings; its id derives from it.
	Key string `json:"key,omitzero"`
	// Role is a name of yours, data-role for stylesheets.
	Role string `json:"role,omitzero"`
	// Tone is the semantic color of the node's chrome.
	Tone Tone `json:"tone,omitzero"`
	// Hidden keeps the node mounted but not laid out.
	Hidden bool `json:"hidden,omitzero"`
	// Mark draws a transient selection over the node.
	Mark Mark `json:"mark,omitzero"`
	// Actions say what a pointer does on the node.
	Actions *Actions `json:"actions,omitzero"`
	// Title is a tooltip.
	Title string `json:"title,omitzero"`
	// Aria is the accessible name of a node that shows no text.
	Aria string `json:"aria,omitzero"`
	// Href is a link for the node.
	Href string `json:"href,omitzero"`
	// Grow is the flex grow inside a row or col.
	Grow float64 `json:"grow,omitzero"`
	// Shrink is the flex shrink inside a row or col (default 1).
	Shrink *float64 `json:"shrink,omitzero"`
	// Basis is the flex basis: a fraction of the parent, or BasisContent.
	Basis Basis `json:"basis,omitzero"`
	// Min bounds the node's size from below.
	Min *Bounds `json:"min,omitzero"`
	// Max bounds the node's size from above (a max height also clips).
	Max *Bounds `json:"max,omitzero"`

	// Props sets props untyped, after the typed ones; a nil value removes one.
	Props map[string]any `json:"-"`

	// OnClick runs on the node's click action (setting actions.click).
	OnClick func(*tern.Action) `json:"-"`
	// OnDblClick runs on the node's dblclick action (setting actions.dblclick).
	OnDblClick func(*tern.Action) `json:"-"`
	// OnMenu adds context-menu actions and runs on them.
	OnMenu map[string]func(*tern.Action) `json:"-"`
	// OnAction runs on action events by act (or act=value).
	OnAction map[string]func(*tern.Action) `json:"-"`
	// OnToggle runs when the user folds or unfolds the node.
	OnToggle func(*tern.Toggle) `json:"-"`
	// OnSelect runs on select events (a list's item selects).
	OnSelect func(*tern.Select) `json:"-"`
	// OnActivate runs on activate events (a list's item activations).
	OnActivate func(*tern.Activate) `json:"-"`
	// OnChange runs when a control or prefs row changes.
	OnChange func(*tern.Change) `json:"-"`
	// OnFocus runs when a click asks for the keys in the node.
	OnFocus func(*tern.Focus) `json:"-"`
	// OnEdit runs on native edits to the field.
	OnEdit func(*tern.Edit) `json:"-"`
	// OnUndo runs on undo in the field.
	OnUndo func(*tern.Undo) `json:"-"`
	// OnSend runs when Tern submits text into the field.
	OnSend func(*tern.Send) `json:"-"`
}

// CommonWithoutTitle holds common props for Tool and Prefs, whose Title is
// content rather than a tooltip. The conflicting common prop is not exposed.
type CommonWithoutTitle struct {
	Key        string                        `json:"key,omitzero"`
	Role       string                        `json:"role,omitzero"`
	Tone       Tone                          `json:"tone,omitzero"`
	Hidden     bool                          `json:"hidden,omitzero"`
	Mark       Mark                          `json:"mark,omitzero"`
	Actions    *Actions                      `json:"actions,omitzero"`
	Aria       string                        `json:"aria,omitzero"`
	Href       string                        `json:"href,omitzero"`
	Grow       float64                       `json:"grow,omitzero"`
	Shrink     *float64                      `json:"shrink,omitzero"`
	Basis      Basis                         `json:"basis,omitzero"`
	Min        *Bounds                       `json:"min,omitzero"`
	Max        *Bounds                       `json:"max,omitzero"`
	Props      map[string]any                `json:"-"`
	OnClick    func(*tern.Action)            `json:"-"`
	OnDblClick func(*tern.Action)            `json:"-"`
	OnMenu     map[string]func(*tern.Action) `json:"-"`
	OnAction   map[string]func(*tern.Action) `json:"-"`
	OnToggle   func(*tern.Toggle)            `json:"-"`
	OnSelect   func(*tern.Select)            `json:"-"`
	OnActivate func(*tern.Activate)          `json:"-"`
	OnChange   func(*tern.Change)            `json:"-"`
	OnFocus    func(*tern.Focus)             `json:"-"`
	OnEdit     func(*tern.Edit)              `json:"-"`
	OnUndo     func(*tern.Undo)              `json:"-"`
	OnSend     func(*tern.Send)              `json:"-"`
}

func (c CommonWithoutTitle) common() Common {
	return Common{
		Props: c.Props, OnClick: c.OnClick, OnDblClick: c.OnDblClick,
		OnMenu: c.OnMenu, OnAction: c.OnAction, OnToggle: c.OnToggle,
		OnSelect: c.OnSelect, OnActivate: c.OnActivate, OnChange: c.OnChange,
		OnFocus: c.OnFocus, OnEdit: c.OnEdit, OnUndo: c.OnUndo, OnSend: c.OnSend,
	}
}

// CommonWithoutMax holds common props for List, whose Max is a height cap.
type CommonWithoutMax struct {
	Key        string                        `json:"key,omitzero"`
	Role       string                        `json:"role,omitzero"`
	Tone       Tone                          `json:"tone,omitzero"`
	Hidden     bool                          `json:"hidden,omitzero"`
	Mark       Mark                          `json:"mark,omitzero"`
	Actions    *Actions                      `json:"actions,omitzero"`
	Title      string                        `json:"title,omitzero"`
	Aria       string                        `json:"aria,omitzero"`
	Href       string                        `json:"href,omitzero"`
	Grow       float64                       `json:"grow,omitzero"`
	Shrink     *float64                      `json:"shrink,omitzero"`
	Basis      Basis                         `json:"basis,omitzero"`
	Min        *Bounds                       `json:"min,omitzero"`
	Props      map[string]any                `json:"-"`
	OnClick    func(*tern.Action)            `json:"-"`
	OnDblClick func(*tern.Action)            `json:"-"`
	OnMenu     map[string]func(*tern.Action) `json:"-"`
	OnAction   map[string]func(*tern.Action) `json:"-"`
	OnToggle   func(*tern.Toggle)            `json:"-"`
	OnSelect   func(*tern.Select)            `json:"-"`
	OnActivate func(*tern.Activate)          `json:"-"`
	OnChange   func(*tern.Change)            `json:"-"`
	OnFocus    func(*tern.Focus)             `json:"-"`
	OnEdit     func(*tern.Edit)              `json:"-"`
	OnUndo     func(*tern.Undo)              `json:"-"`
	OnSend     func(*tern.Send)              `json:"-"`
}

func (c CommonWithoutMax) common() Common {
	return Common{
		Props: c.Props, OnClick: c.OnClick, OnDblClick: c.OnDblClick,
		OnMenu: c.OnMenu, OnAction: c.OnAction, OnToggle: c.OnToggle,
		OnSelect: c.OnSelect, OnActivate: c.OnActivate, OnChange: c.OnChange,
		OnFocus: c.OnFocus, OnEdit: c.OnEdit, OnUndo: c.OnUndo, OnSend: c.OnSend,
	}
}

// PickerCommon holds common props for Picker, whose Title and Actions are
// the heading and action bar rather than a tooltip and pointer gestures.
type PickerCommon struct {
	Key        string                        `json:"key,omitzero"`
	Role       string                        `json:"role,omitzero"`
	Tone       Tone                          `json:"tone,omitzero"`
	Hidden     bool                          `json:"hidden,omitzero"`
	Mark       Mark                          `json:"mark,omitzero"`
	Aria       string                        `json:"aria,omitzero"`
	Href       string                        `json:"href,omitzero"`
	Grow       float64                       `json:"grow,omitzero"`
	Shrink     *float64                      `json:"shrink,omitzero"`
	Basis      Basis                         `json:"basis,omitzero"`
	Min        *Bounds                       `json:"min,omitzero"`
	Max        *Bounds                       `json:"max,omitzero"`
	Props      map[string]any                `json:"-"`
	OnClick    func(*tern.Action)            `json:"-"`
	OnDblClick func(*tern.Action)            `json:"-"`
	OnMenu     map[string]func(*tern.Action) `json:"-"`
	OnAction   map[string]func(*tern.Action) `json:"-"`
	OnToggle   func(*tern.Toggle)            `json:"-"`
	OnSelect   func(*tern.Select)            `json:"-"`
	OnActivate func(*tern.Activate)          `json:"-"`
	OnChange   func(*tern.Change)            `json:"-"`
	OnFocus    func(*tern.Focus)             `json:"-"`
	OnEdit     func(*tern.Edit)              `json:"-"`
	OnUndo     func(*tern.Undo)              `json:"-"`
	OnSend     func(*tern.Send)              `json:"-"`
}

func (c PickerCommon) common() Common {
	return Common{
		Props: c.Props, OnClick: c.OnClick, OnDblClick: c.OnDblClick,
		OnMenu: c.OnMenu, OnAction: c.OnAction, OnToggle: c.OnToggle,
		OnSelect: c.OnSelect, OnActivate: c.OnActivate, OnChange: c.OnChange,
		OnFocus: c.OnFocus, OnEdit: c.OnEdit, OnUndo: c.OnUndo, OnSend: c.OnSend,
	}
}

// common is how Build finds the Common a builder embeds.
func (c Common) common() Common { return c }

// handlers is the node's handlers.
func (c Common) handlers() tern.Handlers {
	return tern.Handlers{
		OnClick: c.OnClick, OnDblClick: c.OnDblClick, OnMenu: c.OnMenu, OnAction: c.OnAction,
		OnToggle: c.OnToggle, OnSelect: c.OnSelect, OnActivate: c.OnActivate, OnChange: c.OnChange,
		OnFocus: c.OnFocus, OnEdit: c.OnEdit, OnUndo: c.OnUndo, OnSend: c.OnSend,
	}
}

// Actions say what a pointer does on a node: an action name per gesture.
type Actions struct {
	// Click is the action a click runs.
	Click string `json:"click,omitzero"`
	// DblClick is the action a double click runs.
	DblClick string `json:"dblclick,omitzero"`
	// Menu lists the context menu's actions.
	Menu []string `json:"menu,omitzero"`
}

// Build makes a node of kind from props, a struct of json-tagged fields
// that embeds Common or one of its variants, and children. Package el and
// custom builders use it.
func Build(kind string, props any, children []tern.Element) tern.Node {
	n := tern.Node{Kind: kind, Children: children}
	if b, err := tern.Marshal(props); err == nil {
		dec := json.NewDecoder(bytes.NewReader(b))
		dec.UseNumber()
		_ = dec.Decode(&n.Props)
	}
	if c, ok := props.(interface{ common() Common }); ok {
		common := c.common()
		n.Handlers = common.handlers()
		for k, v := range common.Props {
			if n.Props == nil {
				n.Props = map[string]any{}
			}
			if v == nil {
				delete(n.Props, k)
			} else {
				n.Props[k] = v
			}
		}
	}
	if len(n.Props) == 0 {
		n.Props = nil
	}
	return n
}

// Span is a styled run of text.
type Span struct {
	// T is the text.
	T string `json:"t"`
	// S is space-separated style tokens: muted, dim, strong, em, accent,
	// success, warning, error, info, code, mono, path, key, link, num, ins,
	// del, mark, typo, icon, hide, or a program palette token.
	S string `json:"s,omitzero"`
	// Fx is an effect.
	Fx Fx `json:"fx,omitzero"`
	// Href makes the span a link.
	Href string `json:"href,omitzero"`
}

// Fx is a span effect.
type Fx string

// Span effects.
const (
	FxShimmer Fx = "shimmer"
	FxPulse   Fx = "pulse"
	FxNone    Fx = "none"
)

// Rich is text given as a plain string (T) or as styled spans (Spans). Its
// zero value is absent.
type Rich struct {
	text    string
	spans   []Span
	isSpans bool
}

// T is plain text.
func T(s string) Rich { return Rich{text: s} }

// Spans is text made of styled spans.
func Spans(spans ...Span) Rich { return Rich{spans: spans, isSpans: true} }

// IsZero reports whether r is absent.
func (r Rich) IsZero() bool { return !r.isSpans && r.text == "" }

// String is r's text without styles.
func (r Rich) String() string {
	if !r.isSpans {
		return r.text
	}
	var b bytes.Buffer
	for _, s := range r.spans {
		b.WriteString(s.T)
	}
	return b.String()
}

// MarshalJSON writes a string or a span list.
func (r Rich) MarshalJSON() ([]byte, error) {
	if r.isSpans {
		if r.spans == nil {
			return []byte("[]"), nil
		}
		return tern.Marshal(r.spans)
	}
	return tern.Marshal(r.text)
}

func (Rich) cell() {}

// Tone is the semantic color of a node's chrome.
type Tone string

// Tones.
const (
	ToneNeutral Tone = "neutral"
	ToneAccent  Tone = "accent"
	ToneInfo    Tone = "info"
	ToneSuccess Tone = "success"
	ToneWarning Tone = "warning"
	ToneError   Tone = "error"
	TonePending Tone = "pending"
	ToneMuted   Tone = "muted"
	ToneUser    Tone = "user"
)

// Mark is a transient selection drawn over a node.
type Mark string

// Marks.
const (
	// MarkPick draws an accent bar at the node's left.
	MarkPick Mark = "pick"
	// MarkDrop dims the node.
	MarkDrop Mark = "drop"
)

// RunState is the status of a card or tool.
type RunState string

// Run states.
const (
	Pending   RunState = "pending"
	Running   RunState = "running"
	Done      RunState = "done"
	Errored   RunState = "error"
	Cancelled RunState = "cancelled"
)

// AgentState is an agent's status, distinct from card and tool run states.
type AgentState string

// Agent states.
const (
	AgentPending AgentState = "pending"
	AgentRunning AgentState = "running"
	AgentDone    AgentState = "done"
	AgentFailed  AgentState = "failed"
	AgentAborted AgentState = "aborted"
	AgentIdle    AgentState = "idle"
	AgentParked  AgentState = "parked"
)

// Extent is one size bound: character cells, text lines or a fraction of
// the parent. Its zero value is absent.
type Extent struct{ v any }

// Ch is n character cells.
func Ch(n float64) Extent { return Extent{strconv.FormatFloat(n, 'f', -1, 64) + "ch"} }

// Lines is n text lines.
func Lines(n float64) Extent { return Extent{strconv.FormatFloat(n, 'f', -1, 64) + "lines"} }

// Frac is a fraction of the parent (0.5 is half).
func Frac(f float64) Extent { return Extent{f} }

// IsZero reports whether e is absent.
func (e Extent) IsZero() bool { return e.v == nil }

// MarshalJSON writes "<n>ch", "<n>lines" or a number.
func (e Extent) MarshalJSON() ([]byte, error) { return tern.Marshal(e.v) }

// Bounds are size bounds in width and height.
type Bounds struct {
	W Extent `json:"w,omitzero"`
	H Extent `json:"h,omitzero"`
}

// Basis is a flex basis: a fraction of the parent or the node's content
// size. Its zero value is absent.
type Basis struct{ v any }

// BasisFrac is a fraction of the parent (0.5 is half).
func BasisFrac(f float64) Basis { return Basis{f} }

// BasisContent keeps the node's own size without shrinking.
var BasisContent = Basis{"content"}

// IsZero reports whether b is absent.
func (b Basis) IsZero() bool { return b.v == nil }

// MarshalJSON writes a number or "content".
func (b Basis) MarshalJSON() ([]byte, error) { return tern.Marshal(b.v) }

// Gap is the space between children, and a spacer's height.
type Gap string

// Gaps.
const (
	GapNone Gap = "none"
	GapXS   Gap = "xs"
	GapSM   Gap = "sm"
	GapMD   Gap = "md"
	GapLG   Gap = "lg"
)

// Align is a cross-axis or text alignment.
type Align string

// Alignments.
const (
	AlignStart    Align = "start"
	AlignCenter   Align = "center"
	AlignEnd      Align = "end"
	AlignBaseline Align = "baseline"
	AlignStretch  Align = "stretch"
)

// Justify is a main-axis distribution.
type Justify string

// Distributions.
const (
	JustifyBetween Justify = "between"
	JustifyEnd     Justify = "end"
)

// Size is a size step of meters, charts, overlays and pickers.
type Size string

// Sizes.
const (
	SizeSM     Size = "sm"
	SizeMD     Size = "md"
	SizeLG     Size = "lg"
	SizeFull   Size = "full"
	SizeScreen Size = "screen"
)

// Wrap is how text wraps.
type Wrap string

// Wrapping.
const (
	WrapWord Wrap = "word"
	WrapChar Wrap = "char"
	WrapNone Wrap = "none"
)

// Truncate is where overflowing text is cut.
type Truncate string

// Truncation.
const (
	TruncateEnd    Truncate = "end"
	TruncateStart  Truncate = "start"
	TruncateMiddle Truncate = "middle"
)

// Preview is how much of a collapsed body shows. Its zero value is absent.
type Preview struct{ v any }

// PreviewAuto is a card's default preview (10 lines).
var PreviewAuto = Preview{"auto"}

// PreviewLines shows the first n lines.
func PreviewLines(n int) Preview { return Preview{map[string]int{"lines": n}} }

// PreviewTail shows the last n lines (a tool's body).
func PreviewTail(n int) Preview { return Preview{map[string]int{"tail": n}} }

// IsZero reports whether p is absent.
func (p Preview) IsZero() bool { return p.v == nil }

// MarshalJSON writes "auto" or {lines|tail: n}.
func (p Preview) MarshalJSON() ([]byte, error) { return tern.Marshal(p.v) }

// Chip is a badge in a tool's head, an agent row or a picker row.
type Chip struct {
	Text  string `json:"text"`
	Tone  Tone   `json:"tone,omitzero"`
	Title string `json:"title,omitzero"`
}
