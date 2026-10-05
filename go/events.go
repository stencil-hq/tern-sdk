package tern

import (
	"bytes"
	"encoding/json"
)

// Reply is a terminal → program r message, the answer to a q.
type Reply struct {
	// R is the reply's name: "hello", "blobs" or one newer than the SDK.
	R string
	// Hello is set when R is "hello".
	Hello *HelloReply
	// Blobs is set when R is "blobs".
	Blobs *BlobsReply
	// Raw is the reply's JSON object as received.
	Raw json.RawMessage
}

// Cell is a cell size in pixels.
type Cell struct {
	W float64 `json:"w"`
	H float64 `json:"h"`
}

// HelloReply is the terminal's answer to hello: what it speaks and supports.
type HelloReply struct {
	V            int      `json:"v"`
	Term         string   `json:"term"`
	Ver          string   `json:"ver"`
	Kinds        []string `json:"kinds"`
	Features     []string `json:"features"`
	APC          int      `json:"apc"`
	Credits      int      `json:"credits"`
	Cols         int      `json:"cols"`
	Cell         Cell     `json:"cell"`
	Dark         bool     `json:"dark"`
	ReduceMotion bool     `json:"reduceMotion"`
	Hour12       bool     `json:"hour12"` // a 12-hour clock (3:05 PM); false (24-hour) when unsaid
}

// BlobsReply lists the blobs Tern holds, in the order asked.
type BlobsReply struct {
	Have []string `json:"have"`
}

// decodeReply decodes an r body; ok is false when it is not a JSON object.
func decodeReply(body []byte) (*Reply, bool) {
	if !isObject(body) {
		return nil, false
	}
	raw := json.RawMessage(bytes.Clone(body))
	var head struct {
		R json.RawMessage `json:"r"`
	}
	_ = json.Unmarshal(raw, &head)
	r := &Reply{Raw: raw}
	_ = json.Unmarshal(head.R, &r.R)
	switch r.R {
	case "hello":
		var h HelloReply
		if json.Unmarshal(raw, &h) == nil {
			r.Hello = &h
		}
	case "blobs":
		var b BlobsReply
		if json.Unmarshal(raw, &b) == nil {
			r.Blobs = &b
		}
	}
	return r, true
}

// isObject reports whether body is one valid JSON object.
func isObject(body []byte) bool {
	t := bytes.TrimLeft(body, " \t\r\n")
	return len(t) > 0 && t[0] == '{' && json.Valid(body)
}

// Input is what a session's input loop yields: a Key or an Event.
type Input interface {
	isInput()
}

// Event is a terminal → program e message. Switch on its concrete type:
// *Ack, *Resize, *Theme, *Motion, *Visible, *Toggle, *Select, *Activate,
// *Action, *Change, *Focus, *Edit, *Undo, *Send, *ErrorEvent, *Gone or
// *UnknownEvent.
type Event interface {
	Input
	// Type is the event's ev field ("ack", "action", …).
	Type() string
	// Surface is the event's sf field, "" when it names no surface.
	Surface() string
	// Target is the node id the event is about, "" when none.
	Target() string
	// Raw is the event's JSON object as received.
	Raw() json.RawMessage
	base() *EventBase
}

// EventBase holds the fields every event has.
type EventBase struct {
	Ev  string `json:"ev"`
	SF  string `json:"sf,omitempty"`
	raw json.RawMessage
}

func (*EventBase) isInput() {}

// Type is the event's ev field.
func (b *EventBase) Type() string { return b.Ev }

// Surface is the event's sf field.
func (b *EventBase) Surface() string { return b.SF }

// Target is "" for events not about one node.
func (b *EventBase) Target() string { return "" }

// Raw is the event's JSON object as received.
func (b *EventBase) Raw() json.RawMessage { return b.raw }

func (b *EventBase) base() *EventBase { return b }

// Values are the named controls of the form around an event's node.
type Values map[string]any

// String is the value of a radio group or text-valued control ("" when absent).
func (v Values) String(name string) string {
	s, _ := v[name].(string)
	return s
}

// Bool is the value of a lone checkbox.
func (v Values) Bool(name string) bool {
	b, _ := v[name].(bool)
	return b
}

// Strings is the checked values of checkboxes sharing a name.
func (v Values) Strings(name string) []string {
	list, _ := v[name].([]any)
	out := make([]string, 0, len(list))
	for _, x := range list {
		if s, ok := x.(string); ok {
			out = append(out, s)
		}
	}
	return out
}

// Ack acknowledges every frame up to S as drawn.
type Ack struct {
	EventBase
	S int64 `json:"s"`
}

// Resize reports the pane's width in columns and its cell size.
type Resize struct {
	EventBase
	Cols    int  `json:"cols"`
	Cell    Cell `json:"cell"`
	Visible bool `json:"visible"`
}

// Theme reports that the appearance switched.
type Theme struct {
	EventBase
	Dark bool `json:"dark"`
}

// Motion reports that Reduce Motion was toggled.
type Motion struct {
	EventBase
	Reduce bool `json:"reduce"`
}

// Visible reports that the pane was hidden or shown.
type Visible struct {
	EventBase
	Visible bool `json:"visible"`
}

// Toggle reports that the user folded or unfolded a node.
type Toggle struct {
	EventBase
	ID        string `json:"id"`
	Collapsed bool   `json:"collapsed"`
	Key       string `json:"key,omitempty"`
}

// Select reports a select action; for a list's item ID is the list.
type Select struct {
	EventBase
	ID     string `json:"id"`
	Item   string `json:"item"`
	Values Values `json:"values,omitempty"`
}

// Activate reports an activate action; for a list's item ID is the list.
type Activate struct {
	EventBase
	ID     string `json:"id"`
	Item   string `json:"item"`
	Values Values `json:"values,omitempty"`
}

// Action reports a program-named pointer action.
type Action struct {
	EventBase
	ID     string   `json:"id"`
	Act    string   `json:"act"`
	Value  string   `json:"value,omitempty"`
	Mods   []string `json:"mods,omitempty"`
	Values Values   `json:"values,omitempty"`
}

// Change reports an el checkbox or radio flip, or a prefs row change.
type Change struct {
	EventBase
	ID      string `json:"id"`
	Value   any    `json:"value"`
	Checked bool   `json:"checked"`
	Name    string `json:"name,omitempty"`
	Item    string `json:"item,omitempty"`
	Values  Values `json:"values,omitempty"`
}

// Focus asks the program to move its focus to field ID.
type Focus struct {
	EventBase
	ID string `json:"id"`
}

// Edit asks the program to replace UTF-16 [From, To) of field ID with Text
// and put the caret at Cursor; Len is the text length Tern saw.
type Edit struct {
	EventBase
	ID     string `json:"id"`
	From   int    `json:"from"`
	To     int    `json:"to"`
	Text   string `json:"text"`
	Cursor int    `json:"cursor"`
	Len    int    `json:"len"`
}

// Undo asks the program to undo the last change to field ID.
type Undo struct {
	EventBase
	ID string `json:"id"`
}

// Send asks the program to submit Text through composer ID.
type Send struct {
	EventBase
	ID   string `json:"id"`
	Text string `json:"text"`
}

// ErrorEvent reports a rejected op, message, stylesheet or el tag.
type ErrorEvent struct {
	EventBase
	S     *int64 `json:"s,omitempty"`
	Op    *int   `json:"op,omitempty"`
	Msg   string `json:"msg"`
	Sheet string `json:"sheet,omitempty"`
	ID    string `json:"id,omitempty"`
}

// Gone reports nodes or surfaces Tern dropped.
type Gone struct {
	EventBase
	IDs []string `json:"ids"`
}

// UnknownEvent is an event newer than the SDK, or one whose fields did not
// decode; Raw holds it whole.
type UnknownEvent struct {
	EventBase
}

// Target is the node the event is about.
func (e *Toggle) Target() string { return e.ID }

// Target is the node the event is about (the list for a list item).
func (e *Select) Target() string { return e.ID }

// Target is the node the event is about (the list for a list item).
func (e *Activate) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Action) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Change) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Focus) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Edit) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Undo) Target() string { return e.ID }

// Target is the node the event is about.
func (e *Send) Target() string { return e.ID }

// DecodeEvent decodes an e body; ok is false when it is not a JSON object.
// An unknown ev, or fields of the wrong type, give an *UnknownEvent.
func DecodeEvent(body []byte) (Event, bool) {
	if !isObject(body) {
		return nil, false
	}
	raw := json.RawMessage(bytes.Clone(body))
	var head EventBase
	_ = json.Unmarshal(raw, &head)
	var ev Event
	switch head.Ev {
	case "ack":
		ev = &Ack{}
	case "resize":
		ev = &Resize{}
	case "theme":
		ev = &Theme{}
	case "motion":
		ev = &Motion{}
	case "visible":
		ev = &Visible{}
	case "toggle":
		ev = &Toggle{}
	case "select":
		ev = &Select{}
	case "activate":
		ev = &Activate{}
	case "action":
		ev = &Action{}
	case "change":
		ev = &Change{}
	case "focus":
		ev = &Focus{}
	case "edit":
		ev = &Edit{}
	case "undo":
		ev = &Undo{}
	case "send":
		ev = &Send{}
	case "error":
		ev = &ErrorEvent{}
	case "gone":
		ev = &Gone{}
	}
	if ev == nil || json.Unmarshal(raw, ev) != nil {
		ev = &UnknownEvent{EventBase: head}
	}
	ev.base().raw = raw
	return ev, true
}
