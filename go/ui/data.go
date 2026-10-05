package ui

import tern "github.com/stencil-hq/tern-sdk/go"

// KV is an aligned key/value list (kind kv).
type KV struct {
	Common
	// Items are the pairs, in order.
	Items []Pair `json:"items,omitzero"`
	// Layout is grid (default) or inline.
	Layout KVLayout `json:"layout,omitzero"`
}

// Pair is one key/value pair.
type Pair struct {
	K Rich `json:"k"`
	V Rich `json:"v"`
}

// KVLayout is how a kv list lays out.
type KVLayout string

// KV layouts.
const (
	KVGrid   KVLayout = "grid"
	KVInline KVLayout = "inline"
)

// Node builds the kv.
func (k KV) Node() tern.Node { return Build(tern.KindKV, k, nil) }

// Table is a table with priority-hidden columns and meter cells (kind table).
type Table struct {
	Common
	// Cols are the columns, left to right.
	Cols []Column `json:"cols,omitzero"`
	// Rows are the rows, top to bottom.
	Rows []TableRow `json:"rows,omitzero"`
}

// Column is a table column.
type Column struct {
	// ID is the key of the column's cell in each row.
	ID string `json:"id"`
	// Head is the header text.
	Head Rich `json:"head,omitzero"`
	// Align is start (default), center or end.
	Align Align `json:"align,omitzero"`
	// Truncate is where a cell that doesn't fit is cut.
	Truncate Truncate `json:"truncate,omitzero"`
	// Priority: lower hides first when the table is narrow.
	Priority float64 `json:"priority,omitzero"`
	// Grow is the column's share of spare width.
	Grow float64 `json:"grow,omitzero"`
}

// TableRow is a table row.
type TableRow struct {
	// ID is the row's identity, unique in the table.
	ID string `json:"id"`
	// Cells hold a Rich or a MeterCell per column id.
	Cells map[string]Cell `json:"cells,omitzero"`
}

// Cell is a table cell: Rich text or a MeterCell.
type Cell interface {
	cell()
}

// MeterCell draws a bar in a table cell.
type MeterCell struct {
	// Value is the fill, 0–1 (default the sum of Parts).
	Value *float64 `json:"value,omitzero"`
	// Parts are stacked fills.
	Parts []Part `json:"parts,omitzero"`
	// Thresholds tint the fill at levels.
	Thresholds *Thresholds `json:"thresholds,omitzero"`
	// Tone tints the fill.
	Tone Tone `json:"tone,omitzero"`
	// Title is the tooltip.
	Title string `json:"title,omitzero"`
}

func (MeterCell) cell() {}

// MarshalJSON wraps the meter props in the table cell's meter discriminator.
func (m MeterCell) MarshalJSON() ([]byte, error) {
	type fields MeterCell
	return tern.Marshal(struct {
		Meter fields `json:"meter"`
	}{Meter: fields(m)})
}

// Part is one stacked fill of a meter.
type Part struct {
	// Value is the part's share of the whole.
	Value float64 `json:"value"`
	// Token is its color: a span or palette token, or "track" for a gap.
	Token string `json:"token,omitzero"`
	// Label names the part in the tooltip.
	Label string `json:"label,omitzero"`
	// Hatch stripes the part.
	Hatch bool `json:"hatch,omitzero"`
}

// Thresholds set data-level warn or bad at these values.
type Thresholds struct {
	Warn *float64 `json:"warn,omitzero"`
	Bad  *float64 `json:"bad,omitzero"`
}

// Node builds the table.
func (t Table) Node() tern.Node { return Build(tern.KindTable, t, nil) }

// Tree is a disclosure tree (kind tree).
type Tree struct {
	Common
	// Nodes are the top-level items.
	Nodes []TreeNode `json:"nodes,omitzero"`
}

// TreeNode is one tree item.
type TreeNode struct {
	// ID is the item's identity, unique in the tree.
	ID string `json:"id"`
	// Label is the row's text.
	Label Rich `json:"label,omitzero"`
	// Icon is shown before the label.
	Icon IconName `json:"icon,omitzero"`
	// Open shows the children.
	Open bool `json:"open,omitzero"`
	// Children are nested items.
	Children []TreeNode `json:"children,omitzero"`
}

// Node builds the tree.
func (t Tree) Node() tern.Node { return Build(tern.KindTree, t, nil) }

// Badge is a pill chip (kind badge).
type Badge struct {
	Common
	// Text is the label (plain text).
	Text string `json:"text,omitzero"`
}

// Node builds the badge.
func (b Badge) Node() tern.Node { return Build(tern.KindBadge, b, nil) }

// Kbd is keycaps (kind kbd).
type Kbd struct {
	Common
	// Keys are key names, in order ("cmd", "K").
	Keys []string `json:"keys,omitzero"`
}

// Node builds the kbd.
func (k Kbd) Node() tern.Node { return Build(tern.KindKbd, k, nil) }

// Icon is a named icon (kind icon).
type Icon struct {
	Common
	// Name is an icon name or alias.
	Name IconName `json:"name,omitzero"`
}

// Node builds the icon.
func (i Icon) Node() tern.Node { return Build(tern.KindIcon, i, nil) }

// Image is an image from a blob (kind image).
type Image struct {
	Common
	// Blob is the blob id (Session.Blob returns it).
	Blob string `json:"blob,omitzero"`
	// Builtin is an image Tern ships ("omp").
	Builtin string `json:"builtin,omitzero"`
	// Alt is the accessible name and placeholder.
	Alt string `json:"alt,omitzero"`
	// W and H are the natural size in px.
	W float64 `json:"w,omitzero"`
	H float64 `json:"h,omitzero"`
	// Path is read by the zoom viewer.
	Path string `json:"path,omitzero"`
}

// Node builds the image.
func (i Image) Node() tern.Node { return Build(tern.KindImage, i, nil) }
