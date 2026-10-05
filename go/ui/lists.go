package ui

import tern "github.com/stencil-hq/tern-sdk/go"

// List is a selectable list of Item children (kind list). Clicks on items
// send select, double clicks activate, both with the list as the target.
type List struct {
	CommonWithoutMax
	// Selected is the node id of the selected item.
	Selected string `json:"selected,omitzero"`
	// Filter marks its matches in each item's label.
	Filter string `json:"filter,omitzero"`
	// Empty shows when the list has no children.
	Empty Rich `json:"empty,omitzero"`
	// Max caps the height (MaxLines or MaxFrac); the list then scrolls.
	Max ListMax `json:"max,omitzero"`
	// Virtual virtualizes the rows even under 200 items.
	Virtual bool `json:"virtual,omitzero"`
	// Children are the list's items.
	Children []tern.Element `json:"-"`
}

// ListMax caps a list's height. Its zero value is absent.
type ListMax struct{ v any }

// MaxLines caps a list at n rows.
func MaxLines(n int) ListMax { return ListMax{map[string]int{"lines": n}} }

// MaxFrac caps a list at a fraction of the window height.
func MaxFrac(f float64) ListMax { return ListMax{f} }

// IsZero reports whether m is absent.
func (m ListMax) IsZero() bool { return m.v == nil }

// MarshalJSON writes {lines: n} or a number.
func (m ListMax) MarshalJSON() ([]byte, error) { return tern.Marshal(m.v) }

// Node builds the list.
func (l List) Node() tern.Node { return Build(tern.KindList, l, l.Children) }

// Item is one list row (kind item).
type Item struct {
	Common
	// Label is the row's text.
	Label Rich `json:"label,omitzero"`
	// Detail is dim text after the label.
	Detail Rich `json:"detail,omitzero"`
	// Value is right-aligned text.
	Value Rich `json:"value,omitzero"`
	// Icon is an icon name.
	Icon IconName `json:"icon,omitzero"`
	// Hint are key names drawn as keycaps at the end.
	Hint []string `json:"hint,omitzero"`
	// Disabled dims the row; it sends no select or activate.
	Disabled bool `json:"disabled,omitzero"`
}

// Node builds the item.
func (i Item) Node() tern.Node { return Build(tern.KindItem, i, nil) }

// Tabs is a tab strip (kind tabs); a click sends select with the tab's id.
type Tabs struct {
	Common
	// Items are the tabs in order.
	Items []Tab `json:"items,omitzero"`
	// Active is the id of the active tab.
	Active string `json:"active,omitzero"`
}

// Tab is one tab.
type Tab struct {
	ID    string `json:"id"`
	Label Rich   `json:"label,omitzero"`
	// Count is shown by a picker's tab row.
	Count *int `json:"count,omitzero"`
}

// Node builds the tabs.
func (t Tabs) Node() tern.Node { return Build(tern.KindTabs, t, nil) }

// Picker is a searchable picker sheet (kind picker); its children are the
// preview.
type Picker struct {
	PickerCommon
	// Size is md, lg (default) or screen.
	Size Size `json:"size,omitzero"`
	// Layout is the row template.
	Layout PickerLayout `json:"layout,omitzero"`
	// Preview is where the preview pane sits.
	Preview PickerPane `json:"preview,omitzero"`
	// Title is the heading (not a tooltip).
	Title Rich `json:"title,omitzero"`
	// Subtitle is the heading's second part.
	Subtitle Rich `json:"subtitle,omitzero"`
	// Icon is the head icon.
	Icon IconName `json:"icon,omitzero"`
	// Query is the search text; nil hides the search.
	Query *string `json:"query,omitzero"`
	// Cursor is the caret in Query, in UTF-16 units (default the end).
	Cursor *int `json:"cursor,omitzero"`
	// Placeholder is the search placeholder.
	Placeholder string `json:"placeholder,omitzero"`
	// Noun is what the rows are.
	Noun string `json:"noun,omitzero"`
	// Focus is which part the keys drive.
	Focus PickerFocus `json:"focus,omitzero"`
	// State is ready, loading or error.
	State PickerState `json:"state,omitzero"`
	// Message is the error card text.
	Message Rich `json:"message,omitzero"`
	// Empty is the text when there are no rows.
	Empty Rich `json:"empty,omitzero"`
	// Total is the catalog size.
	Total *int `json:"total,omitzero"`
	// Items is the catalog.
	Items []PickerItem `json:"items,omitzero"`
	// ItemsAdd adds or replaces catalog items by id.
	ItemsAdd []PickerItem `json:"itemsAdd,omitzero"`
	// ItemsDel removes catalog items by id.
	ItemsDel []string `json:"itemsDel,omitzero"`
	// Order is what to show: item ids and group headers.
	Order []OrderEntry `json:"order,omitzero"`
	// Selected is the selected item id.
	Selected string `json:"selected,omitzero"`
	// Current are item ids marked current.
	Current []string `json:"current,omitzero"`
	// Hits are match ranges per item id, UTF-16 [from, to).
	Hits map[string][][2]int `json:"hits,omitzero"`
	// Columns are fact columns, at most 8.
	Columns []PickerColumn `json:"columns,omitzero"`
	// Confirm is a confirm strip over the selected row.
	Confirm *Confirm `json:"confirm,omitzero"`
	// Scopes is a scope column at the left.
	Scopes []Scope `json:"scopes,omitzero"`
	// Scope is the active scope id.
	Scope string `json:"scope,omitzero"`
	// Tabs is a tab row over the list.
	Tabs []Tab `json:"tabs,omitzero"`
	// Tab is the active tab id.
	Tab string `json:"tab,omitzero"`
	// Strip is a row of switch chips above the action bar.
	Strip *Strip `json:"strip,omitzero"`
	// Actions is the action bar, replacing the common pointer actions prop.
	Actions []PickerAction `json:"actions,omitzero"`
	// Children are the preview.
	Children []tern.Element `json:"-"`
}

// PickerLayout is a picker's row template.
type PickerLayout string

// Picker layouts.
const (
	LayoutRows     PickerLayout = "rows"
	LayoutCards    PickerLayout = "cards"
	LayoutTimeline PickerLayout = "timeline"
	LayoutTree     PickerLayout = "tree"
)

// PickerPane is where a picker's preview pane sits.
type PickerPane string

// Preview panes.
const (
	PaneSide  PickerPane = "side"
	PaneBelow PickerPane = "below"
	PaneNone  PickerPane = "none"
)

// PickerFocus is which part of a picker the keys drive.
type PickerFocus string

// Picker focus targets.
const (
	FocusList    PickerFocus = "list"
	FocusScopes  PickerFocus = "scopes"
	FocusTabs    PickerFocus = "tabs"
	FocusStrip   PickerFocus = "strip"
	FocusPreview PickerFocus = "preview"
)

// PickerState is ready, loading or error.
type PickerState string

// Picker states.
const (
	StateReady   PickerState = "ready"
	StateLoading PickerState = "loading"
	StateError   PickerState = "error"
)

// PickerItem is one catalog row of a picker.
type PickerItem struct {
	// ID is required.
	ID string `json:"id"`
	// Label is the row text.
	Label Rich `json:"label,omitzero"`
	// Detail follows the label.
	Detail Rich `json:"detail,omitzero"`
	// Icon is the lead icon.
	Icon IconName `json:"icon,omitzero"`
	// Role picks the lead icon when Icon is unset.
	Role string `json:"role,omitzero"`
	// Mark is an initials avatar.
	Mark *Avatar `json:"mark,omitzero"`
	// Dot is a status dot tone.
	Dot Tone `json:"dot,omitzero"`
	// Mono draws the label monospace.
	Mono bool `json:"mono,omitzero"`
	// Facts are column values: numbers or Rich.
	Facts map[string]any `json:"facts,omitzero"`
	// Badges follow the label (up to 3 shown).
	Badges []Chip `json:"badges,omitzero"`
	// Chips are role chips after the badges.
	Chips []RoleChip `json:"chips,omitzero"`
	// Tone is data-tone on the row.
	Tone Tone `json:"tone,omitzero"`
	// Disabled dims the row; a string is the reason.
	Disabled any `json:"disabled,omitzero"`
	// Hits are match ranges in the label.
	Hits [][2]int `json:"hits,omitzero"`
	// Node is a timeline node style.
	Node string `json:"node,omitzero"`
	// Depth is a tree's indent.
	Depth int `json:"depth,omitzero"`
	// Open is a tree chevron; nil is a leaf.
	Open *bool `json:"open,omitzero"`
	// Title is the row tooltip.
	Title string `json:"title,omitzero"`
}

// Avatar is an initials avatar on a gradient.
type Avatar struct {
	Text string `json:"text"`
	Seed string `json:"seed,omitzero"`
}

// RoleChip is a chip after a picker row's badges.
type RoleChip struct {
	Text string `json:"text"`
	On   bool   `json:"on,omitzero"`
	Auto bool   `json:"auto,omitzero"`
	Dot  string `json:"dot,omitzero"`
}

// OrderEntry is an item id, or a group header when Group is set.
type OrderEntry struct {
	ID    string
	Group string
	Label Rich
	Count *int
}

// MarshalJSON writes the id, or the header object.
func (o OrderEntry) MarshalJSON() ([]byte, error) {
	if o.Group == "" {
		return tern.Marshal(o.ID)
	}
	return tern.Marshal(struct {
		Group string `json:"group"`
		Label Rich   `json:"label"`
		Count *int   `json:"count,omitzero"`
	}{o.Group, o.Label, o.Count})
}

// PickerColumn is a fact column.
type PickerColumn struct {
	ID       string       `json:"id"`
	Head     string       `json:"head,omitzero"`
	Format   ColumnFormat `json:"format,omitzero"`
	Priority *float64     `json:"priority,omitzero"`
	Min      int          `json:"min,omitzero"`
}

// ColumnFormat styles a fact column.
type ColumnFormat string

// Column formats.
const (
	FormatText    ColumnFormat = "text"
	FormatNum     ColumnFormat = "num"
	FormatPrice   ColumnFormat = "price"
	FormatBar     ColumnFormat = "bar"
	FormatTime    ColumnFormat = "time"
	FormatElapsed ColumnFormat = "elapsed"
	FormatDim     ColumnFormat = "dim"
)

// Confirm is a confirm strip over a picker's selected row.
type Confirm struct {
	Text  Rich   `json:"text"`
	Act   string `json:"act,omitzero"`
	Label string `json:"label,omitzero"`
}

// Scope is one entry of a picker's scope column.
type Scope struct {
	ID       string   `json:"id"`
	Label    Rich     `json:"label,omitzero"`
	Group    string   `json:"group,omitzero"`
	Mark     *Avatar  `json:"mark,omitzero"`
	Icon     IconName `json:"icon,omitzero"`
	Dot      Tone     `json:"dot,omitzero"`
	Count    *int     `json:"count,omitzero"`
	Disabled any      `json:"disabled,omitzero"`
}

// Strip is a picker's row of switch chips.
type Strip struct {
	Label    Rich        `json:"label,omitzero"`
	Items    []StripChip `json:"items"`
	Selected string      `json:"selected,omitzero"`
}

// StripChip is one chip of a strip.
type StripChip struct {
	ID    string `json:"id"`
	Label Rich   `json:"label,omitzero"`
	On    bool   `json:"on,omitzero"`
	Dot   string `json:"dot,omitzero"`
}

// PickerAction is a button of a picker's action bar or a tool's head.
type PickerAction struct {
	ID       string   `json:"id"`
	Label    string   `json:"label,omitzero"`
	Keys     []string `json:"keys,omitzero"`
	Primary  bool     `json:"primary,omitzero"`
	End      bool     `json:"end,omitzero"`
	Danger   bool     `json:"danger,omitzero"`
	On       bool     `json:"on,omitzero"`
	Disabled any      `json:"disabled,omitzero"`
}

// Node builds the picker.
func (p Picker) Node() tern.Node { return Build(tern.KindPicker, p, p.Children) }
