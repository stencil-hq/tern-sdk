package ui

import tern "github.com/stencil-hq/tern-sdk/go"

// Tool is one step of work: a head drawn from data over a body of children
// (kind tool).
type Tool struct {
	CommonWithoutTitle
	// Name picks the icon (bash, read, edit, …).
	Name string `json:"name,omitzero"`
	// Title is the verb ("Deploy").
	Title Rich `json:"title,omitzero"`
	// Target is the primary argument.
	Target Rich `json:"target,omitzero"`
	// TargetKind is how a string Target is drawn.
	TargetKind TargetKind `json:"targetKind,omitzero"`
	// Meta are short facts after the target.
	Meta []Rich `json:"meta,omitzero"`
	// Badges are chips in the head.
	Badges []Chip `json:"badges,omitzero"`
	// Note is a short state note before the timer.
	Note Rich `json:"note,omitzero"`
	// Exit is a non-zero exit code chip.
	Exit int `json:"exit,omitzero"`
	// Status is the state (default pending).
	Status RunState `json:"status,omitzero"`
	// Age is the ms elapsed when sent; a live timer while running.
	Age *float64 `json:"age,omitzero"`
	// Took is the final duration in ms.
	Took *float64 `json:"took,omitzero"`
	// Intent is the head's tooltip.
	Intent string `json:"intent,omitzero"`
	// Frame is card (default) or inline.
	Frame ToolFrame `json:"frame,omitzero"`
	// Collapsible lets the head toggle the body.
	Collapsible bool `json:"collapsible,omitzero"`
	// Collapsed is the initial collapse state.
	Collapsed *bool `json:"collapsed,omitzero"`
	// Preview clamps the collapsed body (PreviewLines or PreviewTail).
	Preview Preview `json:"preview,omitzero"`
	// Tools are hover action buttons.
	Tools []PickerAction `json:"tools,omitzero"`
	// Children are the body.
	Children []tern.Element `json:"-"`
}

// TargetKind is how a tool's target is drawn.
type TargetKind string

// Target kinds.
const (
	TargetCommand TargetKind = "command"
	TargetPath    TargetKind = "path"
	TargetPattern TargetKind = "pattern"
	TargetQuery   TargetKind = "query"
	TargetText    TargetKind = "text"
)

// ToolFrame is a tool's frame.
type ToolFrame string

// Tool frames.
const (
	FrameCard   ToolFrame = "card"
	FrameInline ToolFrame = "inline"
)

// Node builds the tool.
func (t Tool) Node() tern.Node { return Build(tern.KindTool, t, t.Children) }

// Agent is one subagent row (kind agent).
type Agent struct {
	Common
	// Name is the display name.
	Name string `json:"name,omitzero"`
	// Agent is the type, the first badge.
	Agent string `json:"agent,omitzero"`
	// Badges are more chips.
	Badges []Chip `json:"badges,omitzero"`
	// Task is the one-line task.
	Task Rich `json:"task,omitzero"`
	// Status is the state (default pending).
	Status AgentState `json:"status,omitzero"`
	// Model is the model chip.
	Model string `json:"model,omitzero"`
	// Thinking is a palette token for the model chip's dot.
	Thinking string `json:"thinking,omitzero"`
	// Stats are the row's facts; without them there is no timer.
	Stats *AgentStats `json:"stats,omitzero"`
	// Tool is the tool running now.
	Tool *AgentTool `json:"tool,omitzero"`
	// Retry is a retry countdown (wins over Tool).
	Retry *AgentRetry `json:"retry,omitzero"`
	// Depth is the nesting depth.
	Depth int `json:"depth,omitzero"`
	// Collapsible lets the row toggle the children.
	Collapsible bool `json:"collapsible,omitzero"`
	// Collapsed is the initial collapse state.
	Collapsed *bool `json:"collapsed,omitzero"`
	// Children are nested under the row.
	Children []tern.Element `json:"-"`
}

// AgentStats are an agent row's facts.
type AgentStats struct {
	Tools        int      `json:"tools,omitzero"`
	Requests     int      `json:"requests,omitzero"`
	Done         *float64 `json:"done,omitzero"`
	Context      *float64 `json:"context,omitzero"`
	ContextLabel string   `json:"contextLabel,omitzero"`
	Tokens       *float64 `json:"tokens,omitzero"`
	Cost         float64  `json:"cost,omitzero"`
	Age          *float64 `json:"age,omitzero"`
	Took         *float64 `json:"took,omitzero"`
}

// AgentTool is the tool an agent runs now.
type AgentTool struct {
	Name   string   `json:"name"`
	Intent string   `json:"intent,omitzero"`
	Age    *float64 `json:"age,omitzero"`
}

// AgentRetry is an agent's retry countdown.
type AgentRetry struct {
	Attempt int      `json:"attempt,omitzero"`
	Max     int      `json:"max,omitzero"`
	Delay   float64  `json:"delay,omitzero"`
	Age     *float64 `json:"age,omitzero"`
	Error   string   `json:"error,omitzero"`
}

// Node builds the agent.
func (a Agent) Node() tern.Node { return Build(tern.KindAgent, a, a.Children) }

// Checklist is a todo list in phases (kind checklist).
type Checklist struct {
	Common
	// Phases are the list.
	Phases []Phase `json:"phases,omitzero"`
	// Mode is full (default), hud or reminder.
	Mode ChecklistMode `json:"mode,omitzero"`
	// Note follows the count in reminder mode.
	Note Rich `json:"note,omitzero"`
}

// Phase is one phase of a checklist.
type Phase struct {
	ID        string `json:"id,omitzero"`
	Title     Rich   `json:"title,omitzero"`
	Items     []Todo `json:"items"`
	Collapsed *bool  `json:"collapsed,omitzero"`
}

// Todo is one checklist item.
type Todo struct {
	ID     string    `json:"id"`
	Text   Rich      `json:"text,omitzero"`
	Status TodoState `json:"status,omitzero"`
	Note   Rich      `json:"note,omitzero"`
}

// TodoState is a checklist item's state.
type TodoState string

// Todo states.
const (
	TodoPending TodoState = "pending"
	TodoActive  TodoState = "active"
	TodoDone    TodoState = "done"
	TodoDropped TodoState = "dropped"
	TodoBlocked TodoState = "blocked"
)

// ChecklistMode is a checklist's presentation.
type ChecklistMode string

// Checklist modes.
const (
	ChecklistFull     ChecklistMode = "full"
	ChecklistHUD      ChecklistMode = "hud"
	ChecklistReminder ChecklistMode = "reminder"
)

// Node builds the checklist.
func (c Checklist) Node() tern.Node { return Build(tern.KindChecklist, c, nil) }

// Prefs is a settings page or sheet drawn from data (kind prefs); its
// children are placed by role.
type Prefs struct {
	CommonWithoutTitle
	// Title is the brand title (default "Settings"); not a tooltip.
	Title string `json:"title,omitzero"`
	// Pages are the nav rows.
	Pages []PrefsPage `json:"pages,omitzero"`
	// Page is the current page id.
	Page string `json:"page,omitzero"`
	// Lead is a paragraph under the page title.
	Lead string `json:"lead,omitzero"`
	// Sections are the document's sections.
	Sections []PrefsSection `json:"sections,omitzero"`
	// Query is the search text.
	Query string `json:"query,omitzero"`
	// Cursor is the caret in the search field.
	Cursor *int `json:"cursor,omitzero"`
	// Focus is the row or section id with the focus ring.
	Focus string `json:"focus,omitzero"`
	// Editing is the row being edited with the keyboard.
	Editing *PrefsEditing `json:"editing,omitzero"`
	// Children are placed by role.
	Children []tern.Element `json:"-"`
}

// PrefsPage is a nav row.
type PrefsPage struct {
	ID       string   `json:"id"`
	Label    string   `json:"label,omitzero"`
	Group    string   `json:"group,omitzero"`
	Icon     IconName `json:"icon,omitzero"`
	Disabled string   `json:"disabled,omitzero"`
	Changed  int      `json:"changed,omitzero"`
}

// PrefsSection is a section of rows.
type PrefsSection struct {
	ID    string     `json:"id,omitzero"`
	Title string     `json:"title,omitzero"`
	Page  string     `json:"page,omitzero"`
	Rows  []PrefsRow `json:"rows"`
}

// PrefsRow is a settings row with its control.
type PrefsRow struct {
	ID           string       `json:"id"`
	Label        string       `json:"label,omitzero"`
	Hint         string       `json:"hint,omitzero"`
	Warning      string       `json:"warning,omitzero"`
	Disabled     string       `json:"disabled,omitzero"`
	Changed      bool         `json:"changed,omitzero"`
	DefaultLabel string       `json:"defaultLabel,omitzero"`
	Control      PrefsControl `json:"control"`
}

// PrefsEditing is the row being edited with the keyboard.
type PrefsEditing struct {
	Row    string `json:"row"`
	Option string `json:"option,omitzero"`
	Draft  string `json:"draft,omitzero"`
	Cursor *int   `json:"cursor,omitzero"`
}

// PrefsControl is a row's control: PrefSwitch, PrefChoice, PrefNumber,
// PrefText, PrefKeys, PrefMulti or PrefAction.
type PrefsControl interface {
	control()
}

// PrefSwitch is a toggle.
type PrefSwitch struct {
	On bool `json:"on"`
}

// PrefChoice is segments or a popup menu.
type PrefChoice struct {
	Value   string       `json:"value"`
	Options []PrefOption `json:"options"`
	Style   ChoiceStyle  `json:"style,omitzero"`
	Mono    bool         `json:"mono,omitzero"`
}

// PrefOption is one option of a choice or multi control.
type PrefOption struct {
	Value  string `json:"value"`
	Label  string `json:"label,omitzero"`
	Detail string `json:"detail,omitzero"`
}

// ChoiceStyle is how a choice draws.
type ChoiceStyle string

// Choice styles.
const (
	ChoiceAuto      ChoiceStyle = "auto"
	ChoiceSegmented ChoiceStyle = "segmented"
	ChoiceMenu      ChoiceStyle = "menu"
)

// PrefNumber is a stepper.
type PrefNumber struct {
	Value  float64           `json:"value"`
	Min    *float64          `json:"min,omitzero"`
	Max    *float64          `json:"max,omitzero"`
	Step   float64           `json:"step,omitzero"`
	Unit   string            `json:"unit,omitzero"`
	Labels map[string]string `json:"labels,omitzero"`
}

// PrefText is a text field look.
type PrefText struct {
	Value       string `json:"value"`
	Placeholder string `json:"placeholder,omitzero"`
	Secret      bool   `json:"secret,omitzero"`
	Mono        bool   `json:"mono,omitzero"`
}

// PrefKeys shows key chords.
type PrefKeys struct {
	Keys [][]string `json:"keys"`
}

// PrefMulti is toggle chips or an ordered list.
type PrefMulti struct {
	Values  []string     `json:"values"`
	Options []PrefOption `json:"options"`
	Ordered bool         `json:"ordered,omitzero"`
}

// PrefAction is a small button.
type PrefAction struct {
	Act   string `json:"act,omitzero"`
	Label string `json:"label,omitzero"`
}

func (PrefSwitch) control() {}
func (PrefChoice) control() {}
func (PrefNumber) control() {}
func (PrefText) control()   {}
func (PrefKeys) control()   {}
func (PrefMulti) control()  {}
func (PrefAction) control() {}

// MarshalJSON writes the control with k "switch".
func (c PrefSwitch) MarshalJSON() ([]byte, error) {
	type fields PrefSwitch
	return withK("switch", fields(c))
}

// MarshalJSON writes the control with k "choice".
func (c PrefChoice) MarshalJSON() ([]byte, error) {
	type fields PrefChoice
	return withK("choice", fields(c))
}

// MarshalJSON writes the control with k "number".
func (c PrefNumber) MarshalJSON() ([]byte, error) {
	type fields PrefNumber
	return withK("number", fields(c))
}

// MarshalJSON writes the control with k "text".
func (c PrefText) MarshalJSON() ([]byte, error) {
	type fields PrefText
	return withK("text", fields(c))
}

// MarshalJSON writes the control with k "keys".
func (c PrefKeys) MarshalJSON() ([]byte, error) {
	type fields PrefKeys
	return withK("keys", fields(c))
}

// MarshalJSON writes the control with k "multi".
func (c PrefMulti) MarshalJSON() ([]byte, error) {
	type fields PrefMulti
	return withK("multi", fields(c))
}

// MarshalJSON writes the control with k "action".
func (c PrefAction) MarshalJSON() ([]byte, error) {
	type fields PrefAction
	return withK("action", fields(c))
}

// withK writes v, a struct, as an object led by "k": k.
func withK(k string, v any) ([]byte, error) {
	b, err := tern.Marshal(v)
	if err != nil {
		return nil, err
	}
	sep := ""
	if len(b) > 2 {
		sep = ","
	}
	return append([]byte(`{"k":"`+k+`"`+sep), b[1:]...), nil
}

// Node builds the prefs.
func (p Prefs) Node() tern.Node { return Build(tern.KindPrefs, p, p.Children) }
