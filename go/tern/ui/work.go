package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

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
	// Confirm is the approval look while Status is Asking (default ConfirmCard).
	Confirm ToolConfirm `json:"confirm,omitzero"`
	// Ask is the approval panel's contents. Its buttons send action events
	// with act allow, deny or always and Value the selected scope.
	Ask *ToolAsk `json:"ask,omitzero"`
	// Approved is the resolved answer, shown as a head chip.
	Approved Approval `json:"approved,omitzero"`
	// Children are the body.
	Children []tern.Element `json:"-"`
}

// ToolConfirm is a tool's approval look.
type ToolConfirm string

// Approval looks.
const (
	ConfirmCard   ToolConfirm = "card"
	ConfirmStrip  ToolConfirm = "strip"
	ConfirmScopes ToolConfirm = "scopes"
)

// ToolAsk is a tool's approval panel.
type ToolAsk struct {
	// Text is the question.
	Text string `json:"text,omitzero"`
	// Allow is the Allow button's label (default "Allow").
	Allow string `json:"allow,omitzero"`
	// Deny is the Deny button's label (default "Deny").
	Deny string `json:"deny,omitzero"`
	// Always is the Always button's label (ConfirmCard; default "Always").
	Always Always `json:"always,omitzero"`
	// Scopes are ConfirmScopes' radios (default once, session, always).
	Scopes []AskScope `json:"scopes,omitzero"`
	// Scope is the selected scope's id; a change wins over the pick.
	Scope string `json:"scope,omitzero"`
}

// Always is the Always button of a tool's approval card: AlwaysSays, or
// NoAlways. Its zero value is absent ("Always").
type Always struct{ v any }

// NoAlways hides the Always button.
var NoAlways = Always{false}

// AlwaysSays labels the Always button.
func AlwaysSays(label string) Always { return Always{label} }

// IsZero reports whether a is absent.
func (a Always) IsZero() bool { return a.v == nil }

// MarshalJSON writes the label or false.
func (a Always) MarshalJSON() ([]byte, error) { return tern.Marshal(a.v) }

// AskScope is a radio of a tool's approval scopes.
type AskScope struct {
	ID string `json:"id"`
	// Label is the radio's text (default ID).
	Label string `json:"label,omitzero"`
}

// Approval is a tool's resolved approval.
type Approval string

// Approvals.
const (
	ApprovedOnce    Approval = "once"
	ApprovedSession Approval = "session"
	ApprovedAlways  Approval = "always"
	Denied          Approval = "denied"
)

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

// Message is one message of a conversation: a header, the body (its
// children), reactions, hover actions and a status line (kind message). A
// reaction flip sends change with Value the reaction's text and On; an
// action button sends an action event with its act.
type Message struct {
	CommonWithoutActions
	// Variant is MessageAuto (default), MessageBubble or MessageFlat.
	Variant MessageVariant `json:"variant,omitzero"`
	// Side is MessageStart (default) or MessageEnd, the local user.
	Side MessageSide `json:"side,omitzero"`
	// Author is the name in the header.
	Author string `json:"author,omitzero"`
	// Time is the time in the header ("14:02", "Yesterday").
	Time string `json:"time,omitzero"`
	// Avatar is drawn in the avatar slot: an Avatar, Icon or Image (child "avatar").
	Avatar tern.Element `json:"-"`
	// Continued says the same author sent the message before: no header,
	// no avatar.
	Continued bool `json:"continued,omitzero"`
	// Reactions are reaction chips.
	Reactions []Reaction `json:"reactions,omitzero"`
	// Actions are icon buttons shown on hover.
	Actions []IconAction `json:"actions,omitzero"`
	// Reveal is when the actions show: RevealHover (default) or RevealAlways.
	Reveal MessageReveal `json:"reveal,omitzero"`
	// Status is a quiet mono line under the message ("Read 14:03").
	Status string `json:"status,omitzero"`
	// Pending says it is still on its way: the status shimmers.
	Pending bool `json:"pending,omitzero"`
	// Children are the body, in order.
	Children []tern.Element `json:"-"`
}

// MessageVariant is a message's look.
type MessageVariant string

// Message looks.
const (
	// MessageAuto is an accent bubble for the local user, a plain body for others.
	MessageAuto MessageVariant = "auto"
	// MessageBubble is bubbles on both sides.
	MessageBubble MessageVariant = "bubble"
	// MessageFlat is rows from the left under the avatar and name.
	MessageFlat MessageVariant = "flat"
)

// MessageSide is the side a message sits on.
type MessageSide string

// Message sides.
const (
	MessageStart MessageSide = "start"
	MessageEnd   MessageSide = "end"
)

// MessageReveal is when a message's actions show.
type MessageReveal string

// Action reveals.
const (
	RevealHover  MessageReveal = "hover"
	RevealAlways MessageReveal = "always"
)

// Reaction is a reaction chip of a message.
type Reaction struct {
	// Text is an emoji or glyph, the chip's identity.
	Text string `json:"text"`
	// Count is how many reacted, with the local user (hidden at 0).
	Count int `json:"count,omitzero"`
	// On says the local user reacted.
	On bool `json:"on,omitzero"`
}

// Node builds the message.
func (m Message) Node() tern.Node {
	return slotted(Build(tern.KindMessage, m, m.Children), "avatar", m.Avatar)
}

// Trace is an agent's steps, a chain of thought or a reasoning block with
// sources (kind trace). A child keyed by a line's id is that line's
// content; the rest are the body. Flipping a line or the head sends toggle
// (Open, Item the line). Its Title is content rather than a tooltip.
type Trace struct {
	CommonWithoutTitle
	// Variant is TraceRail (default), TraceSummary or TraceCard.
	Variant TraceVariant `json:"variant,omitzero"`
	// Lines are the steps, in order.
	Lines []TraceLine `json:"lines,omitzero"`
	// Status is the whole trace's: pending (default), running, done or error.
	Status RunState `json:"status,omitzero"`
	// Title is the head text.
	Title Rich `json:"title,omitzero"`
	// Summary is TraceSummary's head text (default derived from Status).
	Summary Rich `json:"summary,omitzero"`
	// Icon is the head glyph when not running or failed.
	Icon IconName `json:"icon,omitzero"`
	// Age is the ms elapsed when sent; a live timer while running.
	Age *float64 `json:"age,omitzero"`
	// Took is the total time in ms once settled.
	Took *float64 `json:"took,omitzero"`
	// Open is whether the head's fold is open (default false on
	// TraceSummary, else true).
	Open *bool `json:"open,omitzero"`
	// Sources are the Sources row at the foot.
	Sources []Source `json:"sources,omitzero"`
	// Children are the lines' content and the body.
	Children []tern.Element `json:"-"`
}

// TraceVariant is a trace's look.
type TraceVariant string

// Trace looks.
const (
	// TraceRail is lines on a vertical rail, a live log.
	TraceRail TraceVariant = "rail"
	// TraceSummary is one line that unfolds into the steps.
	TraceSummary TraceVariant = "summary"
	// TraceCard is a reasoning card: a head, prose, source chips.
	TraceCard TraceVariant = "card"
)

// TraceLine is one step of a trace.
type TraceLine struct {
	// ID keys the line's content child and is the toggle's Item (default its index).
	ID string `json:"id,omitzero"`
	// Icon is its glyph, tinted by status.
	Icon IconName `json:"icon,omitzero"`
	// Title is the step in words.
	Title Rich `json:"title,omitzero"`
	// Target is what it acted on, in mono.
	Target Rich `json:"target,omitzero"`
	// Detail is a dim line under the row.
	Detail Rich `json:"detail,omitzero"`
	// Status is pending (default), running, done or error.
	Status RunState `json:"status,omitzero"`
	// Age is the ms elapsed when sent; a live timer while running.
	Age *float64 `json:"age,omitzero"`
	// Took is its time in ms once settled.
	Took *float64 `json:"took,omitzero"`
	// Open shows its content.
	Open bool `json:"open,omitzero"`
	// Sources are chips under the line.
	Sources []Source `json:"sources,omitzero"`
}

// Source is a source chip of a trace.
type Source struct {
	Title string `json:"title,omitzero"`
	Href  string `json:"href,omitzero"`
	// Site is the chip's text (default the href's host, www. dropped).
	Site string `json:"site,omitzero"`
}

// Node builds the trace.
func (t Trace) Node() tern.Node { return Build(tern.KindTrace, t, t.Children) }

// Stacktrace is an error with its stack (kind stacktrace). A click on a
// frame sends select and a double click activate, each with Item the
// frame's index; a double click also opens the frame's file.
type Stacktrace struct {
	Common
	// Error is the error the head shows.
	Error *StackError `json:"error,omitzero"`
	// Frames are the stack, innermost first.
	Frames []StackFrame `json:"frames,omitzero"`
	// Selected is the selected frame's index (default the first app frame).
	Selected *int `json:"selected,omitzero"`
	// Lang is the excerpts' language (default by the first excerpt's file).
	Lang string `json:"lang,omitzero"`
	// Variant is StackFrames (default), StackSplit or StackMono.
	Variant StacktraceVariant `json:"variant,omitzero"`
}

// StackError is a stacktrace's error.
type StackError struct {
	Type    string `json:"type,omitzero"`
	Message string `json:"message,omitzero"`
}

// StackFrame is one frame of a stacktrace.
type StackFrame struct {
	// Fn is the function (<anonymous> when empty).
	Fn string `json:"fn,omitzero"`
	// File is the file path; a relative one opens against the pane's directory.
	File string `json:"file,omitzero"`
	// Line is the 1-based line.
	Line int `json:"line,omitzero"`
	// Col is the 1-based column.
	Col int `json:"col,omitzero"`
	// App says the frame is your code (default true); library frames fold.
	App *bool `json:"app,omitzero"`
	// Code is a source excerpt for StackSplit.
	Code *FrameCode `json:"code,omitzero"`
}

// FrameCode is a source excerpt of a stack frame.
type FrameCode struct {
	// Start is the first line's number (default 1).
	Start int `json:"start,omitzero"`
	// Lines are the excerpt's lines.
	Lines []string `json:"lines"`
	// Line is the line to mark (default the frame's).
	Line int `json:"line,omitzero"`
}

// StacktraceVariant is a stacktrace's look.
type StacktraceVariant string

// Stacktrace looks.
const (
	// StackFrames is an error card over the app frames, library runs folded.
	StackFrames StacktraceVariant = "frames"
	// StackSplit is a frame list beside the selected frame's code.
	StackSplit StacktraceVariant = "split"
	// StackMono is a mono well of "at fn (file:line:col)" lines.
	StackMono StacktraceVariant = "mono"
)

// Node builds the stacktrace.
func (s Stacktrace) Node() tern.Node { return Build(tern.KindStacktrace, s, nil) }

// Suggestions are prompts the user can pick (kind suggestions). A click
// sends activate with Item the suggestion's id (else its text).
type Suggestions struct {
	Common
	// Variant is SuggestChips (default), SuggestRows or SuggestCards.
	Variant SuggestionsVariant `json:"variant,omitzero"`
	// Items are the suggestions, in order.
	Items []Suggestion `json:"items,omitzero"`
}

// Suggestion is one suggestion.
type Suggestion struct {
	// ID is what activate names (default Text).
	ID string `json:"id,omitzero"`
	// Text is the prompt as shown; an item without it is skipped.
	Text string `json:"text"`
	// Detail is muted after the text, or under it on a card.
	Detail string `json:"detail,omitzero"`
	// Icon leads the item.
	Icon IconName `json:"icon,omitzero"`
	// Keys are SuggestRows' keycaps (default the item's number); an empty
	// non-nil list draws none.
	Keys []string `json:"keys,omitzero"`
}

// SuggestionsVariant is a suggestions' look.
type SuggestionsVariant string

// Suggestion looks.
const (
	// SuggestChips is wrapping pills with a sparkle.
	SuggestChips SuggestionsVariant = "chips"
	// SuggestRows is full-width rows with keycaps.
	SuggestRows SuggestionsVariant = "rows"
	// SuggestCards is two columns of cards: the text over a muted reason.
	SuggestCards SuggestionsVariant = "cards"
)

// Node builds the suggestions.
func (s Suggestions) Node() tern.Node { return Build(tern.KindSuggestions, s, nil) }

// Questions is a questionnaire: steps, inline chips or a folding list
// (kind questions). A pick sends change with Item the question and Value
// the answer; Next, Back and Skip send action events with Item the
// question; Submit sends an action event with act "submit" and Value every
// answer; a click on another question of QuestionsList sends select.
type Questions struct {
	Common
	// Items are the questions.
	Items []Question `json:"items,omitzero"`
	// Step is the current question's index.
	Step int `json:"step,omitzero"`
	// Answers are the answers by question id: a value, a list of values
	// (Multi) or text (Other).
	Answers map[string]any `json:"answers,omitzero"`
	// Variant is QuestionsSteps (default), QuestionsInline or QuestionsList.
	Variant QuestionsVariant `json:"variant,omitzero"`
	// Hint is QuestionsInline's hint line (default derived); new("") hides it.
	Hint *string `json:"hint,omitzero"`
	// Submit is the submit label (default "Submit").
	Submit string `json:"submit,omitzero"`
	// Children are the questions' Other inputs, keyed "<id>.other".
	Children []tern.Element `json:"-"`
}

// Question is one question of a questionnaire.
type Question struct {
	// ID is the events' Item and the key in Answers; a question without
	// one is skipped.
	ID string `json:"id"`
	// Prompt is the question.
	Prompt string `json:"prompt,omitzero"`
	// Description is a line under the prompt.
	Description string `json:"description,omitzero"`
	// Choices are the choices.
	Choices []Choice `json:"choices,omitzero"`
	// Multi takes several picks: the answer is a list of values.
	Multi bool `json:"multi,omitzero"`
	// Other takes a free-text answer from the input child keyed "<id>.other".
	Other *Other `json:"other,omitzero"`
	// Required has no Skip; Next and Submit wait for its answer.
	Required bool `json:"required,omitzero"`
}

// Choice is one choice of a question.
type Choice struct {
	Value string `json:"value"`
	// Label is the choice's text (default Value).
	Label string `json:"label,omitzero"`
	// Detail is a second line on a choice card (QuestionsSteps).
	Detail string `json:"detail,omitzero"`
}

// Other is a question's free-text answer.
type Other struct {
	Placeholder string `json:"placeholder,omitzero"`
}

// QuestionsVariant is a questionnaire's look.
type QuestionsVariant string

// Questionnaire looks.
const (
	// QuestionsSteps is one card per question with Back, Skip and Next.
	QuestionsSteps QuestionsVariant = "steps"
	// QuestionsInline is the prompt and keyed chips, in the flow.
	QuestionsInline QuestionsVariant = "inline"
	// QuestionsList is every question; answered ones fold.
	QuestionsList QuestionsVariant = "list"
)

// Node builds the questions.
func (q Questions) Node() tern.Node { return Build(tern.KindQuestions, q, q.Children) }
