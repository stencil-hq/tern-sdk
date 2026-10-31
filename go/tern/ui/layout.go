package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

// Col is a vertical stack (kind col).
type Col struct {
	Common
	// Gap is the space between children (absent is none).
	Gap Gap `json:"gap,omitzero"`
	// Align is the cross-axis alignment (absent stretches).
	Align Align `json:"align,omitzero"`
	// Justify is the main-axis distribution.
	Justify Justify `json:"justify,omitzero"`
	// Wrap adds class wrap.
	Wrap bool `json:"wrap,omitzero"`
	// Children are drawn in order.
	Children []tern.Element `json:"-"`
}

// Node builds the col.
func (c Col) Node() tern.Node { return Build(tern.KindCol, c, c.Children) }

// Row is a horizontal flex row (kind row).
type Row struct {
	Common
	// Gap is the space between children (absent is none).
	Gap Gap `json:"gap,omitzero"`
	// Align is align-items (absent is center).
	Align Align `json:"align,omitzero"`
	// Justify is the main-axis distribution.
	Justify Justify `json:"justify,omitzero"`
	// Wrap lets children flow onto more lines.
	Wrap bool `json:"wrap,omitzero"`
	// Children are drawn in order.
	Children []tern.Element `json:"-"`
}

// Node builds the row.
func (r Row) Node() tern.Node { return Build(tern.KindRow, r, r.Children) }

// Card is a rounded card with a head and a body, optionally folding (kind card).
type Card struct {
	Common
	// Head is the head content.
	Head Rich `json:"head,omitzero"`
	// HeadNode is a node drawn as the head, in place of Head (child "head").
	HeadNode tern.Element `json:"-"`
	// Icon is the head icon, over the one the role implies.
	Icon IconName `json:"icon,omitzero"`
	// Meta is drawn muted after the head text.
	Meta Rich `json:"meta,omitzero"`
	// End is drawn at the head's end, never folding the card: buttons,
	// segmented tabs, badges (child "end").
	End tern.Element `json:"-"`
	// Status shows a status chip.
	Status RunState `json:"status,omitzero"`
	// Collapsible lets the head flip the fold.
	Collapsible bool `json:"collapsible,omitzero"`
	// Collapsed is the starting fold state (needs Collapsible).
	Collapsed bool `json:"collapsed,omitzero"`
	// Preview is what shows while collapsed (PreviewAuto or PreviewLines).
	Preview Preview `json:"preview,omitzero"`
	// Selected draws an accent ring.
	Selected bool `json:"selected,omitzero"`
	// Inset lets children fill the card edge to edge.
	Inset bool `json:"inset,omitzero"`
	// Variant "bare" drops the ring, fill and rounding.
	Variant string `json:"variant,omitzero"`
	// Children are placed by the card.
	Children []tern.Element `json:"-"`
}

// CardBare is the card variant without ring, fill or rounding.
const CardBare = "bare"

// Node builds the card.
func (c Card) Node() tern.Node {
	return slotted(Build(tern.KindCard, c, c.Children), "head", c.HeadNode, "end", c.End)
}

// slotted is n with each non-nil element of pairs (name, element) as the
// slot of that name.
func slotted(n tern.Node, pairs ...any) tern.Node {
	for i := 0; i+1 < len(pairs); i += 2 {
		name, _ := pairs[i].(string)
		el, _ := pairs[i+1].(tern.Element)
		if el == nil {
			continue
		}
		if n.Slots == nil {
			n.Slots = map[string]tern.Element{}
		}
		n.Slots[name] = el
	}
	return n
}

// Section is a lighter disclosure group with no ring (kind section).
type Section struct {
	Common
	// Head is the head content.
	Head Rich `json:"head,omitzero"`
	// HeadNode is a node drawn as the head, in place of Head (child "head").
	HeadNode tern.Element `json:"-"`
	// Collapsible lets the head flip the fold.
	Collapsible bool `json:"collapsible,omitzero"`
	// Collapsed is the starting fold state (needs Collapsible).
	Collapsed bool `json:"collapsed,omitzero"`
	// Children are placed by the section.
	Children []tern.Element `json:"-"`
}

// Node builds the section.
func (s Section) Node() tern.Node {
	return slotted(Build(tern.KindSection, s, s.Children), "head", s.HeadNode)
}

// Rule is a divider hairline, optionally labeled, or a marker in a
// timeline: a framed note row or a pill (kind rule). Common.Tone colors the
// icon, or tints the pill.
type Rule struct {
	Common
	// Label is the label.
	Label Rich `json:"label,omitzero"`
	// Variant is RuleLine (default), RuleNote or RulePin.
	Variant RuleVariant `json:"variant,omitzero"`
	// Icon leads the label.
	Icon IconName `json:"icon,omitzero"`
	// Time is a time or short stamp beside the label.
	Time string `json:"time,omitzero"`
	// End is drawn at the end: after the label, at a note's right, beside
	// the pill (child "end").
	End tern.Element `json:"-"`
}

// RuleVariant is a rule's look.
type RuleVariant string

// Rule looks.
const (
	// RuleLine is a hairline with a centered label.
	RuleLine RuleVariant = "line"
	// RuleNote is a framed row: icon, label, time at the right.
	RuleNote RuleVariant = "note"
	// RulePin is a pill on the hairline.
	RulePin RuleVariant = "pin"
)

// Node builds the rule.
func (r Rule) Node() tern.Node { return slotted(Build(tern.KindRule, r, nil), "end", r.End) }

// Spacer is vertical space (kind spacer).
type Spacer struct {
	Common
	// Size is the height (default md).
	Size Gap `json:"size,omitzero"`
}

// Node builds the spacer.
func (s Spacer) Node() tern.Node { return Build(tern.KindSpacer, s, nil) }

// Empty is an empty state: an icon, a title, one muted line and the
// actions that fill it, its children (kind empty). Its Title is content
// rather than a tooltip; Common.Tone error or warning tints it.
type Empty struct {
	CommonWithoutTitle
	// Variant is EmptyCenter (default), EmptyInline or EmptyDrop.
	Variant EmptyVariant `json:"variant,omitzero"`
	// Icon is the icon (default by look); new(IconName("")) draws none.
	Icon *IconName `json:"icon,omitzero"`
	// Title is what is missing, in a few words.
	Title Rich `json:"title,omitzero"`
	// Text is one muted line: why, or what to do.
	Text Rich `json:"text,omitzero"`
	// Frame draws an EmptyInline row in a hairline card.
	Frame bool `json:"frame,omitzero"`
	// Over lights an EmptyDrop zone up while a drag is over it.
	Over bool `json:"over,omitzero"`
	// Children are the actions, usually one button.
	Children []tern.Element `json:"-"`
}

// EmptyVariant is an empty state's look.
type EmptyVariant string

// Empty state looks.
const (
	// EmptyCenter is an icon in a round well, title, line, actions under, centered.
	EmptyCenter EmptyVariant = "center"
	// EmptyInline is one row: icon, title, muted text, actions at the end.
	EmptyInline EmptyVariant = "inline"
	// EmptyDrop is a dashed zone that takes a dropped file.
	EmptyDrop EmptyVariant = "drop"
)

// Node builds the empty state.
func (e Empty) Node() tern.Node { return Build(tern.KindEmpty, e, e.Children) }

// Split lays its children out as panes with handles between them that the
// user drags (kind split). Releasing a moved handle sends change with every
// pane's share. Its Min is the panes' minimums and shadows Common's size
// bound.
type Split struct {
	Common
	// Variant is SplitHairline (default), SplitGutter or SplitTiles.
	Variant SplitVariant `json:"variant,omitzero"`
	// Dir is SplitRow (default, side by side) or SplitCol (stacked).
	Dir SplitDir `json:"dir,omitzero"`
	// Sizes are each pane's share, in child order (default equal); a change
	// wins over the user's drags.
	Sizes []float64 `json:"sizes,omitzero"`
	// Min are each pane's minimum along the split: Ch, Lines or Frac; a
	// zero Extent is none.
	Min []Extent `json:"min,omitzero"`
	// H is the split's height: Lines, Ch or Frac of its parent (default the
	// panes' content in a row, Lines(12) in a col).
	H Extent `json:"h,omitzero"`
	// Active is the handle your keys move (0 is the one after the first pane).
	Active *int `json:"active,omitzero"`
	// Children are the panes, in order.
	Children []tern.Element `json:"-"`
}

// SplitVariant is a split's look.
type SplitVariant string

// Split looks.
const (
	// SplitHairline is one framed box with a 1px divider.
	SplitHairline SplitVariant = "hairline"
	// SplitGutter is a card per pane with a dotted grip between.
	SplitGutter SplitVariant = "gutter"
	// SplitTiles is a card per pane; the gap is the handle.
	SplitTiles SplitVariant = "tiles"
)

// SplitDir is a split's direction.
type SplitDir string

// Split directions.
const (
	SplitRow SplitDir = "row"
	SplitCol SplitDir = "col"
)

// Node builds the split.
func (s Split) Node() tern.Node { return Build(tern.KindSplit, s, s.Children) }

// Scroll is a box that scrolls its children, optionally stuck to its end
// with a "↓ N new" pill (kind scroll). It sends no events.
type Scroll struct {
	Common
	// Variant is ScrollBar (default) or ScrollFade.
	Variant ScrollVariant `json:"variant,omitzero"`
	// Stick "end" keeps to the end while content grows (StickEnd).
	Stick string `json:"stick,omitzero"`
	// Pinned is whether a stuck view follows its end (default true with
	// Stick); a change wins over the user's scrolling.
	Pinned *bool `json:"pinned,omitzero"`
	// Bar is when the thumb shows: BarAuto (default) or BarAlways.
	Bar ScrollBarMode `json:"bar,omitzero"`
	// Gap is the space between the children.
	Gap Gap `json:"gap,omitzero"`
	// Noun is what the counts name ("4 more lines", "2 new entries").
	Noun Noun `json:"noun,omitzero"`
	// Children are drawn in order.
	Children []tern.Element `json:"-"`
}

// StickEnd keeps a scroll at its end while content grows.
const StickEnd = "end"

// ScrollVariant is a scroll's look.
type ScrollVariant string

// Scroll looks.
const (
	// ScrollBar is an overlay thumb that shows while the view scrolls.
	ScrollBar ScrollVariant = "bar"
	// ScrollFade fades the edges where content continues, with an "N more ↓" chip.
	ScrollFade ScrollVariant = "fade"
)

// ScrollBarMode is when a scroll's thumb shows.
type ScrollBarMode string

// Thumb modes.
const (
	BarAuto   ScrollBarMode = "auto"
	BarAlways ScrollBarMode = "always"
)

// Noun is what a scroll's counts name: One, and Many when the plural is not
// One with an s. Its zero value is absent.
type Noun struct {
	One, Many string
}

// IsZero reports whether n is absent.
func (n Noun) IsZero() bool { return n.One == "" }

// MarshalJSON writes the noun, or [one, many].
func (n Noun) MarshalJSON() ([]byte, error) {
	if n.Many == "" {
		return tern.Marshal(n.One)
	}
	return tern.Marshal([2]string{n.One, n.Many})
}

// Node builds the scroll.
func (s Scroll) Node() tern.Node { return Build(tern.KindScroll, s, s.Children) }
