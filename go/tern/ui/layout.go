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
func (c Card) Node() tern.Node { return Build(tern.KindCard, c, c.Children) }

// Section is a lighter disclosure group with no ring (kind section).
type Section struct {
	Common
	// Head is the head content.
	Head Rich `json:"head,omitzero"`
	// Collapsible lets the head flip the fold.
	Collapsible bool `json:"collapsible,omitzero"`
	// Collapsed is the starting fold state (needs Collapsible).
	Collapsed bool `json:"collapsed,omitzero"`
	// Children are placed by the section.
	Children []tern.Element `json:"-"`
}

// Node builds the section.
func (s Section) Node() tern.Node { return Build(tern.KindSection, s, s.Children) }

// Rule is a divider hairline, optionally labeled (kind rule).
type Rule struct {
	Common
	// Label is the centered label.
	Label Rich `json:"label,omitzero"`
}

// Node builds the rule.
func (r Rule) Node() tern.Node { return Build(tern.KindRule, r, nil) }

// Spacer is vertical space (kind spacer).
type Spacer struct {
	Common
	// Size is the height (default md).
	Size Gap `json:"size,omitzero"`
}

// Node builds the spacer.
func (s Spacer) Node() tern.Node { return Build(tern.KindSpacer, s, nil) }
