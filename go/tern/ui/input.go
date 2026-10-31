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
	// Typeahead says the program cannot read input yet: Tern keeps what is
	// typed into the field and sends it once this clears.
	Typeahead bool `json:"typeahead,omitzero"`
	// Variant is the look (default InputField with any addon, else InputWell).
	Variant InputVariant `json:"variant,omitzero"`
	// Buttons are attached buttons, in order; a click sends an action event
	// with the button's act.
	Buttons []FieldButton `json:"buttons,omitzero"`
}

// InputVariant is an input's or editor's look.
type InputVariant string

// Text field looks.
const (
	// InputWell is the tinted well.
	InputWell InputVariant = "well"
	// InputField is a form field with a hairline ring, holding the addons.
	InputField InputVariant = "field"
)

// FieldButton is a button attached to a text field.
type FieldButton struct {
	// Act is the action a click sends ("copy" copies the text; "x=y" sends
	// act x with value y); an entry without one is skipped.
	Act string `json:"act"`
	// Text is the label; without one the button is icon-only.
	Text string `json:"text,omitzero"`
	// Icon leads the label.
	Icon IconName `json:"icon,omitzero"`
	// Variant ButtonDefault is an attached hairline segment; the others are
	// an inset small button in the button kind's look.
	Variant ButtonVariant `json:"variant,omitzero"`
	// Keys are keycaps after the label (inset buttons).
	Keys []string `json:"keys,omitzero"`
	// Title is the tooltip, and an icon-only button's accessible name.
	Title string `json:"title,omitzero"`
	// Disabled fades the button; it sends nothing.
	Disabled Disabled `json:"disabled,omitzero"`
	// Busy puts a spinner in the icon's place (inset buttons); it sends nothing.
	Busy bool `json:"busy,omitzero"`
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

// Input is a single-line text field (kind input). Its addons send action
// events: clear, inc and dec, and each button's act.
type Input struct {
	Common
	Field
	// Icon leads the text.
	Icon IconName `json:"icon,omitzero"`
	// Keys are one keycap for the chord at the end while the field is idle
	// and empty (cmd, f draws ⌘F).
	Keys []string `json:"keys,omitzero"`
	// Mono keeps the terminal's mono face for a field's text in a sans region.
	Mono bool `json:"mono,omitzero"`
	// Clear draws a circled × at the end while there is text; it sends clear.
	Clear bool `json:"clear,omitzero"`
	// Prefix is a mono well before the text, attached to the ring.
	Prefix string `json:"prefix,omitzero"`
	// Suffix is a mono well after the text, attached to the ring.
	Suffix string `json:"suffix,omitzero"`
	// Unit is a mono unit after the text, inside the field.
	Unit string `json:"unit,omitzero"`
	// Stepper is an attached ⌃⌄ segment; it sends inc and dec.
	Stepper bool `json:"stepper,omitzero"`
	// Secret draws a • per UTF-16 unit of the text, with an eye that reveals it.
	Secret bool `json:"secret,omitzero"`
	// Reveal is a secret's shown state; a change wins over the eye.
	Reveal bool `json:"reveal,omitzero"`
}

// Node builds the input.
func (i Input) Node() tern.Node { return Build(tern.KindInput, i, nil) }

// Button is a push button (kind button). A click sends an action event
// with act Actions.Click, or "click" when no actions are set; OnClick
// handles it. It never takes focus: Keys show which key does the same.
type Button struct {
	Common
	// Text is the label. Empty with Icon is icon-only (set Title or Aria).
	Text string `json:"text,omitzero"`
	// Icon leads the label.
	Icon IconName `json:"icon,omitzero"`
	// Keys are keycaps drawn after the label, display only ("r", "cmd").
	Keys []string `json:"keys,omitzero"`
	// Variant is the chrome (default ButtonDefault).
	Variant ButtonVariant `json:"variant,omitzero"`
	// Size is SizeMD (28px, default) or SizeSM (22px, inline and rows).
	Size Size `json:"size,omitzero"`
	// On draws a pressed toggle.
	On bool `json:"on,omitzero"`
	// Disabled dims the button; it sends nothing.
	Disabled Disabled `json:"disabled,omitzero"`
	// Busy replaces the icon with a spinner; it sends nothing.
	Busy bool `json:"busy,omitzero"`
}

// ButtonVariant is a button's chrome.
type ButtonVariant string

// Button variants. Keep one primary per view.
const (
	ButtonDefault ButtonVariant = "default"
	ButtonPrimary ButtonVariant = "primary"
	ButtonQuiet   ButtonVariant = "quiet"
	ButtonDanger  ButtonVariant = "danger"
)

// Node builds the button.
func (b Button) Node() tern.Node { return Build(tern.KindButton, b, nil) }

// Disabled turns a control off: Off, or Because with the reason as the
// tooltip. Its zero value is absent (enabled).
type Disabled struct{ v any }

// Off is disabled without a reason.
var Off = Disabled{true}

// Because is disabled with reason as the tooltip.
func Because(reason string) Disabled { return Disabled{reason} }

// IsZero reports whether d is absent.
func (d Disabled) IsZero() bool { return d.v == nil }

// MarshalJSON writes true or the reason.
func (d Disabled) MarshalJSON() ([]byte, error) { return tern.Marshal(d.v) }

// Select is a value picked from a menu: a field, a quiet trigger or, with
// Search, a combobox (kind select). It never takes focus: a click on the
// trigger sends toggle (Open), a pick sends change with the option's value,
// then toggle. Drive it from the keys with Open, Active, Query and Cursor.
type Select struct {
	Common
	// Value is the current option's value; without one the trigger shows
	// Placeholder.
	Value string `json:"value,omitzero"`
	// Options are the options in order.
	Options []SelectOption `json:"options,omitzero"`
	// Variant is SelectField (default) or SelectQuiet.
	Variant SelectVariant `json:"variant,omitzero"`
	// Label is a muted label before the value, and the combobox's accessible name.
	Label string `json:"label,omitzero"`
	// Placeholder is the trigger's text without a value (default "Select…"),
	// and the search line's without a query (default "Search…").
	Placeholder string `json:"placeholder,omitzero"`
	// Size is SizeMD (default) or SizeSM.
	Size Size `json:"size,omitzero"`
	// Disabled dims the select; it opens nothing and sends nothing.
	Disabled Disabled `json:"disabled,omitzero"`
	// Open shows the menu; a change wins over the pointer's open or close.
	Open bool `json:"open,omitzero"`
	// Active is the highlighted option's value, the keyboard cursor
	// (default Value).
	Active string `json:"active,omitzero"`
	// Search makes the select a combobox.
	Search bool `json:"search,omitzero"`
	// Query is the search text; its hits are marked in the labels.
	Query string `json:"query,omitzero"`
	// Cursor is the caret in Query, in UTF-16 units (default the end).
	Cursor *int `json:"cursor,omitzero"`
	// Total is the size of the whole set: with a Query the count reads
	// "<shown> of <total>".
	Total *int `json:"total,omitzero"`
	// Empty is the menu's text when there are no options (default "No
	// matches" with a Query, else "No options").
	Empty string `json:"empty,omitzero"`
}

// SelectVariant is a select's look.
type SelectVariant string

// Select looks.
const (
	// SelectField is a card button with a hairline ring, for forms.
	SelectField SelectVariant = "field"
	// SelectQuiet is the value on a pill shown on hover, for heads and bars.
	SelectQuiet SelectVariant = "quiet"
)

// SelectOption is one option of a select.
type SelectOption struct {
	// Value is what Value, Active and change carry; an option without one
	// is skipped.
	Value string `json:"value"`
	// Label is what the option shows (default Value).
	Label string `json:"label,omitzero"`
	// Detail is muted mono text at the option's right.
	Detail string `json:"detail,omitzero"`
	// Icon leads the option, and the trigger while it is current.
	Icon IconName `json:"icon,omitzero"`
	// Group draws a section header over each run of options with the same group.
	Group string `json:"group,omitzero"`
	// Disabled draws the option faint; it is never picked.
	Disabled bool `json:"disabled,omitzero"`
}

// Node builds the select.
func (s Select) Node() tern.Node { return Build(tern.KindSelect, s, nil) }

// Switch is an on/off setting (kind switch). A click flips it at once and
// sends change with the new state as Value; mirror it into On.
type Switch struct {
	Common
	// On is whether the switch is on; a change wins over the user's flip.
	On bool `json:"on,omitzero"`
	// Variant is SwitchDefault, SwitchCompact or SwitchSegment.
	Variant SwitchVariant `json:"variant,omitzero"`
	// Label is the setting's name; a labeled switch fills its row.
	Label string `json:"label,omitzero"`
	// Hint is a muted line under the label.
	Hint string `json:"hint,omitzero"`
	// Labels are a segment's two sides, off first (default Off, On).
	Labels [2]string `json:"labels,omitzero"`
	// Disabled dims the switch; it flips nothing and sends nothing.
	Disabled Disabled `json:"disabled,omitzero"`
}

// SwitchVariant is a switch's look.
type SwitchVariant string

// Switch looks.
const (
	// SwitchDefault is a 36×22 switch; with a label, a settings row.
	SwitchDefault SwitchVariant = "default"
	// SwitchCompact is a small switch with its label to the right.
	SwitchCompact SwitchVariant = "compact"
	// SwitchSegment is an Off | On segmented control.
	SwitchSegment SwitchVariant = "segment"
)

// Node builds the switch.
func (s Switch) Node() tern.Node { return Build(tern.KindSwitch, s, nil) }

// Slider is a value, or a range, dragged along a track (kind slider). The
// release sends change with the new value: a number, or a [lo, hi] list.
// Its Min and Max are the value's ends and shadow Common's size bounds.
type Slider struct {
	Common
	// Value is the value (SliderAt) or a range (SliderRange); default Min.
	Value SliderValue `json:"value,omitzero"`
	// Min is the low end.
	Min float64 `json:"min,omitzero"`
	// Max is the high end (default 100).
	Max *float64 `json:"max,omitzero"`
	// Step is the distance between values, counted from Min (default a
	// hundredth of the span's order of magnitude).
	Step float64 `json:"step,omitzero"`
	// Label is the name at the left (over the track in SliderAccent).
	Label string `json:"label,omitzero"`
	// Unit is written after the number (SliderNumber).
	Unit string `json:"unit,omitzero"`
	// Format is how the value reads (default SliderNumber).
	Format SliderFormat `json:"format,omitzero"`
	// Ticks are tick marks under the track.
	Ticks Ticks `json:"ticks,omitzero"`
	// Variant is SliderDefault or SliderAccent.
	Variant SliderVariant `json:"variant,omitzero"`
	// Disabled dims the slider; it does not move and sends nothing.
	Disabled Disabled `json:"disabled,omitzero"`
}

// SliderValue is a slider's value or range. Its zero value is absent.
type SliderValue struct{ v any }

// SliderAt is one value.
func SliderAt(v float64) SliderValue { return SliderValue{v} }

// SliderRange is a range with two thumbs.
func SliderRange(lo, hi float64) SliderValue { return SliderValue{[2]float64{lo, hi}} }

// IsZero reports whether v is absent.
func (v SliderValue) IsZero() bool { return v.v == nil }

// MarshalJSON writes a number or [lo, hi].
func (v SliderValue) MarshalJSON() ([]byte, error) { return tern.Marshal(v.v) }

// SliderFormat is how a slider's value reads.
type SliderFormat string

// Slider formats.
const (
	// SliderNumber is the value with the step's decimals, then the unit.
	SliderNumber SliderFormat = "number"
	// SliderPercent is a fraction as a percentage (0.9 reads 90%).
	SliderPercent SliderFormat = "percent"
	// SliderDuration is milliseconds as 200 ms, 1.2 s, 1 min 30 s.
	SliderDuration SliderFormat = "duration"
)

// Ticks are a slider's tick marks. Its zero value is absent (the variant's
// default).
type Ticks struct{ v any }

// Tick marks.
var (
	// NoTicks draws none.
	NoTicks = Ticks{false}
	// EveryStep puts one at every step (up to 100).
	EveryStep = Ticks{true}
)

// TickParts divides the track into n equal parts.
func TickParts(n int) Ticks { return Ticks{n} }

// TicksAt puts one at each value.
func TicksAt(values ...float64) Ticks { return Ticks{values} }

// IsZero reports whether t is absent.
func (t Ticks) IsZero() bool { return t.v == nil }

// MarshalJSON writes a bool, a count or a list.
func (t Ticks) MarshalJSON() ([]byte, error) { return tern.Marshal(t.v) }

// SliderVariant is a slider's look.
type SliderVariant string

// Slider looks.
const (
	// SliderDefault is one row: label, a neutral track, the value.
	SliderDefault SliderVariant = "default"
	// SliderAccent is the label and value over an accent track with ticks
	// and a value bubble.
	SliderAccent SliderVariant = "accent"
)

// Node builds the slider.
func (s Slider) Node() tern.Node { return Build(tern.KindSlider, s, nil) }

// FormField is a labelled form row: the label, its controls (the
// children) and a line under them that is the hint until Error or Warning
// says something is wrong (kind field). It sends no events of its own.
type FormField struct {
	Common
	// Label is the field's name.
	Label Rich `json:"label,omitzero"`
	// Hint is the help line, hidden while an error or warning shows.
	Hint Rich `json:"hint,omitzero"`
	// Error rings the control in red and says why; it wins over Warning.
	Error Problem `json:"error,omitzero"`
	// Warning rings the control in amber and says why.
	Warning Problem `json:"warning,omitzero"`
	// Required draws a * after the label (display only).
	Required bool `json:"required,omitzero"`
	// LabelWidth is FieldGrid's label column (default Ch(12)).
	LabelWidth Extent `json:"labelWidth,omitzero"`
	// Variant is FieldStacked (default), FieldRow or FieldGrid.
	Variant FormFieldVariant `json:"variant,omitzero"`
	// Children are the controls, in order.
	Children []tern.Element `json:"-"`
}

// Problem is a field's error or warning: Flagged, or Says with the line to
// show. Its zero value is absent (cleared).
type Problem struct{ v any }

// Flagged sets the state and the ring without a line.
var Flagged = Problem{true}

// Says sets the state with text as the line under the control.
func Says(text Rich) Problem { return Problem{text} }

// IsZero reports whether p is absent.
func (p Problem) IsZero() bool { return p.v == nil }

// MarshalJSON writes true or the text.
func (p Problem) MarshalJSON() ([]byte, error) { return tern.Marshal(p.v) }

// FormFieldVariant is a field's look.
type FormFieldVariant string

// Field looks.
const (
	// FieldStacked is the label over the control over the note.
	FieldStacked FormFieldVariant = "stacked"
	// FieldRow is a settings row: label and hint left, control right.
	FieldRow FormFieldVariant = "row"
	// FieldGrid is a right-aligned label column before the control.
	FieldGrid FormFieldVariant = "grid"
)

// Node builds the field.
func (f FormField) Node() tern.Node { return Build(tern.KindField, f, f.Children) }

// Otp is a one-time code, one cell per character (kind otp). The program
// owns the keys and answers with Text; a click on a cell sends an action
// event with act "focus" and Value the cell.
type Otp struct {
	Common
	// Text is the characters typed so far, one per cell.
	Text string `json:"text,omitzero"`
	// Length is the number of cells, 1–12 (default 6).
	Length int `json:"length,omitzero"`
	// Group is cells per group (default 3); 0 draws one group.
	Group *int `json:"group,omitzero"`
	// Cursor is the active cell (default the end of Text).
	Cursor Caret `json:"cursor,omitzero"`
	// Variant is OtpCells (default), OtpJoined or OtpUnderline.
	Variant OtpVariant `json:"variant,omitzero"`
	// Mask hides what was typed.
	Mask Mask `json:"mask,omitzero"`
	// State is OtpError or OtpSuccess.
	State OtpState `json:"state,omitzero"`
	// Hint is a quiet line under the cells, in the state's color.
	Hint string `json:"hint,omitzero"`
	// Disabled dims the code; clicks send nothing.
	Disabled Disabled `json:"disabled,omitzero"`
}

// Caret is an otp's active cell: CaretAt, or NoCaret. Its zero value is
// absent (after the text).
type Caret struct{ v any }

// NoCaret shows no active cell.
var NoCaret = Caret{false}

// CaretAt makes cell i (0-based) active.
func CaretAt(i int) Caret { return Caret{i} }

// IsZero reports whether c is absent.
func (c Caret) IsZero() bool { return c.v == nil }

// MarshalJSON writes the cell or false.
func (c Caret) MarshalJSON() ([]byte, error) { return tern.Marshal(c.v) }

// Mask hides an otp's characters: MaskDots, or MaskWith a character. Its
// zero value is absent (shown).
type Mask struct{ v any }

// MaskDots draws a dot in each filled cell.
var MaskDots = Mask{true}

// MaskWith draws the first character of s in each filled cell.
func MaskWith(s string) Mask { return Mask{s} }

// IsZero reports whether m is absent.
func (m Mask) IsZero() bool { return m.v == nil }

// MarshalJSON writes true or the character.
func (m Mask) MarshalJSON() ([]byte, error) { return tern.Marshal(m.v) }

// OtpVariant is an otp's look.
type OtpVariant string

// Otp looks.
const (
	OtpCells     OtpVariant = "cells"
	OtpJoined    OtpVariant = "joined"
	OtpUnderline OtpVariant = "underline"
)

// OtpState is an otp's verdict.
type OtpState string

// Otp states.
const (
	// OtpError rings the cells in red and shakes the code once.
	OtpError OtpState = "error"
	// OtpSuccess rings the cells in green.
	OtpSuccess OtpState = "success"
)

// Node builds the otp.
func (o Otp) Node() tern.Node { return Build(tern.KindOtp, o, nil) }

// Buttons are Button children drawn as one control: joined, split or a
// toolbar well (kind buttons). Each child sends its own action.
type Buttons struct {
	Common
	// Variant is ButtonsJoined (default), ButtonsSplit or ButtonsToolbar.
	Variant ButtonsVariant `json:"variant,omitzero"`
	// Size is the children's size, SizeMD or SizeSM, over their own.
	Size Size `json:"size,omitzero"`
	// Label is the group's accessible name.
	Label string `json:"label,omitzero"`
	// Children are the buttons, in order.
	Children []tern.Element `json:"-"`
}

// ButtonsVariant is a button group's look.
type ButtonsVariant string

// Button group looks.
const (
	// ButtonsJoined rings the buttons as one, with dividers.
	ButtonsJoined ButtonsVariant = "joined"
	// ButtonsSplit is an action and its menu part.
	ButtonsSplit ButtonsVariant = "split"
	// ButtonsToolbar is a soft well of bare buttons.
	ButtonsToolbar ButtonsVariant = "toolbar"
)

// Node builds the buttons.
func (b Buttons) Node() tern.Node { return Build(tern.KindButtons, b, b.Children) }

// Calendar is a month grid, or presets beside a week strip (kind
// calendar). A pick sends change with the day (or the range); a preset
// sends select; ‹ › send an action event with act "month" and the month.
// Its Min and Max are days and shadow Common's size bounds.
type Calendar struct {
	Common
	// Variant is CalendarMonth (default) or CalendarWeek.
	Variant CalendarVariant `json:"variant,omitzero"`
	// Value is the selection: Day or Days.
	Value Dates `json:"value,omitzero"`
	// Range picks a range (two clicks) instead of a day.
	Range bool `json:"range,omitzero"`
	// Month is the month shown, "YYYY-MM" (default Value's, else Today's).
	Month string `json:"month,omitzero"`
	// Today is the day ringed as today, "YYYY-MM-DD" (default the system's).
	Today string `json:"today,omitzero"`
	// Min is the first day that can be picked.
	Min string `json:"min,omitzero"`
	// Max is the last day that can be picked.
	Max string `json:"max,omitzero"`
	// WeekStart is the month grid's first column (default WeekMon).
	WeekStart WeekStart `json:"weekStart,omitzero"`
	// Marks are days carrying a dot.
	Marks []DayMark `json:"marks,omitzero"`
	// Cursor is the day your keys are on, drawn with a focus ring.
	Cursor string `json:"cursor,omitzero"`
	// Presets are CalendarWeek's presets column.
	Presets []Preset `json:"presets,omitzero"`
	// Preset is the active preset's id.
	Preset string `json:"preset,omitzero"`
	// Note is a muted line under the days.
	Note string `json:"note,omitzero"`
}

// CalendarVariant is a calendar's look.
type CalendarVariant string

// Calendar looks.
const (
	CalendarMonth CalendarVariant = "month"
	CalendarWeek  CalendarVariant = "week"
)

// Dates is a calendar's or date's selection. Its zero value is absent.
type Dates struct{ v any }

// Day is one day, "YYYY-MM-DD".
func Day(day string) Dates { return Dates{day} }

// Days is a range from one day to another; an empty to is a range waiting
// for its end.
func Days(from, to string) Dates {
	if to == "" {
		return Dates{[]any{from, nil}}
	}
	return Dates{[]string{from, to}}
}

// IsZero reports whether d is absent.
func (d Dates) IsZero() bool { return d.v == nil }

// MarshalJSON writes the day or [from, to].
func (d Dates) MarshalJSON() ([]byte, error) { return tern.Marshal(d.v) }

// WeekStart is a month grid's first column.
type WeekStart string

// Week starts.
const (
	WeekMon WeekStart = "mon"
	WeekSun WeekStart = "sun"
)

// DayMark is a day carrying a dot.
type DayMark struct {
	// Date is the day, "YYYY-MM-DD".
	Date string `json:"date"`
	// Tone colors the dot (default the accent).
	Tone Tone `json:"tone,omitzero"`
}

// Preset is a calendar's preset.
type Preset struct {
	ID string `json:"id"`
	// Label is the preset's text (default ID).
	Label string `json:"label,omitzero"`
}

// Node builds the calendar.
func (c Calendar) Node() tern.Node { return Build(tern.KindCalendar, c, nil) }

// Date is a date field with a calendar popover (kind date). A click sends
// toggle (Collapsed false when it opened); a pick sends change, then toggle.
// Its Min and Max are days and shadow Common's size bounds.
type Date struct {
	Common
	// Value is the selection: Day or Days.
	Value Dates `json:"value,omitzero"`
	// Range picks a range; the popover stays open until the second end.
	Range bool `json:"range,omitzero"`
	// Open shows the popover; a change wins over the user's.
	Open bool `json:"open,omitzero"`
	// Placeholder shows without a value (default "Pick a date").
	Placeholder string `json:"placeholder,omitzero"`
	// Month is the popover's month, "YYYY-MM".
	Month string `json:"month,omitzero"`
	// Today is the day ringed as today, "YYYY-MM-DD".
	Today string `json:"today,omitzero"`
	// Min is the first day that can be picked.
	Min string `json:"min,omitzero"`
	// Max is the last day that can be picked.
	Max string `json:"max,omitzero"`
	// WeekStart is the month grid's first column (default WeekMon).
	WeekStart WeekStart `json:"weekStart,omitzero"`
	// Marks are days carrying a dot.
	Marks []DayMark `json:"marks,omitzero"`
	// Cursor is the day your keys are on.
	Cursor string `json:"cursor,omitzero"`
	// Note is a muted line under the days.
	Note string `json:"note,omitzero"`
}

// Node builds the date.
func (d Date) Node() tern.Node { return Build(tern.KindDate, d, nil) }
