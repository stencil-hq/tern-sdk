package ui

import (
	"encoding/json"
	"strconv"

	"github.com/stencil-hq/tern-sdk/go/tern"
)

// Status is a status bar strip of Seg children (kind status).
type Status struct {
	Common
	// Transparent draws no fill.
	Transparent bool `json:"transparent,omitzero"`
	// Children are the strip's segments.
	Children []tern.Element `json:"-"`
}

// Node builds the status.
func (s Status) Node() tern.Node { return Build(tern.KindStatus, s, s.Children) }

// Seg is a status bar segment (kind seg).
type Seg struct {
	Common
	// Text is plain text, used when Spans is absent.
	Text string `json:"text,omitzero"`
	// Spans are styled text.
	Spans []Span `json:"spans,omitzero"`
	// Icon is an icon name.
	Icon IconName `json:"icon,omitzero"`
	// Side "right" puts the segment in the strip's right group.
	Side string `json:"side,omitzero"`
	// Priority is the drop order when the strip is narrow (lower first).
	Priority float64 `json:"priority,omitzero"`
	// Children are drawn between the icon and the text.
	Children []tern.Element `json:"-"`
}

// Node builds the seg.
func (s Seg) Node() tern.Node { return Build(tern.KindSeg, s, s.Children) }

// Toast is a transient notice (kind toast); Common.Tone picks its look
// (error, info or accent, else success).
type Toast struct {
	Common
	// Text is the message.
	Text string `json:"text,omitzero"`
	// Sub is a mono detail after it.
	Sub string `json:"sub,omitzero"`
	// TTL takes it down after this many ms.
	TTL *float64 `json:"ttl,omitzero"`
}

// Node builds the toast.
func (t Toast) Node() tern.Node { return Build(tern.KindToast, t, nil) }

// Overlay is a floating panel or sheet in the layer region (kind overlay).
type Overlay struct {
	Common
	// Anchor is the placement (default center).
	Anchor Anchor `json:"anchor,omitzero"`
	// Side docks the overlay to an edge of the layer: a full-height sheet
	// (SideLeft, SideRight) or a drawer (SideBottom). It wins over Anchor.
	Side OverlaySide `json:"side,omitzero"`
	// Variant is, with Side, OverlayEdge (default, flush) or OverlayInset
	// (floating 8px off the edge).
	Variant OverlayVariant `json:"variant,omitzero"`
	// Size is sm, md, lg or full; with Side, the sheet's width or the
	// drawer's height.
	Size Size `json:"size,omitzero"`
	// SizeFrac is, with Side, the size as a fraction of the layer in (0, 1],
	// sent as size in place of Size.
	SizeFrac float64 `json:"-"`
	// Modal dims the backdrop and takes the pointer.
	Modal bool `json:"modal,omitzero"`
	// Head is the title row.
	Head Rich `json:"head,omitzero"`
	// Icon is the head icon.
	Icon IconName `json:"icon,omitzero"`
	// Meta is drawn muted after the head text.
	Meta Rich `json:"meta,omitzero"`
	// End is drawn at the head's end (child "end").
	End tern.Element `json:"-"`
	// Foot is drawn in a footer bar under the body: the sheet's buttons
	// (child "foot").
	Foot tern.Element `json:"-"`
	// Children are drawn in order.
	Children []tern.Element `json:"-"`
}

// OverlaySide is the layer edge an overlay docks to.
type OverlaySide string

// Overlay sides.
const (
	SideLeft   OverlaySide = "left"
	SideBottom OverlaySide = "bottom"
)

// SideRight docks an overlay to the layer's right edge; as a string, it
// puts a segment in the status strip's right group.
const SideRight = "right"

// OverlayVariant is how a docked overlay sits at its edge.
type OverlayVariant string

// Docked overlay looks.
const (
	OverlayEdge  OverlayVariant = "edge"
	OverlayInset OverlayVariant = "inset"
)

// Anchor places an overlay. Its zero value is absent (center).
type Anchor struct{ v any }

// Overlay anchors.
var (
	AnchorCenter = Anchor{"center"}
	AnchorTop    = Anchor{"top"}
	AnchorBottom = Anchor{"bottom"}
)

// AtNode places an overlay beside node id, on side "below" (default) or "above".
func AtNode(id, side string) Anchor {
	m := map[string]string{"node": id}
	if side != "" {
		m["side"] = side
	}
	return Anchor{m}
}

// AtCaret places an overlay below the caret of editor or input id.
func AtCaret(id string) Anchor { return Anchor{map[string]string{"caret": id}} }

// IsZero reports whether a is absent.
func (a Anchor) IsZero() bool { return a.v == nil }

// MarshalJSON writes the anchor name or object.
func (a Anchor) MarshalJSON() ([]byte, error) { return tern.Marshal(a.v) }

// Node builds the overlay.
func (o Overlay) Node() tern.Node {
	n := Build(tern.KindOverlay, o, o.Children)
	if o.SizeFrac != 0 && o.Props["size"] == nil {
		if n.Props == nil {
			n.Props = map[string]any{}
		}
		n.Props["size"] = json.Number(strconv.FormatFloat(o.SizeFrac, 'f', -1, 64))
	}
	return slotted(n, "end", o.End, "foot", o.Foot)
}
