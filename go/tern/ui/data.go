package ui

import "github.com/stencil-hq/tern-sdk/go/tern"

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

// Stats is a well of stat tiles, side by side (kind stats).
type Stats struct {
	Common
	// Items are the tiles, in order.
	Items []Stat `json:"items,omitzero"`
}

// Stat is one stat tile.
type Stat struct {
	// Label names the figure.
	Label Rich `json:"label"`
	// Value is the figure, large with tabular numbers.
	Value Rich `json:"value"`
	// Sub is a muted line under the value.
	Sub Rich `json:"sub,omitzero"`
	// Tone colors the value.
	Tone Tone `json:"tone,omitzero"`
	// Title is the tooltip.
	Title string `json:"title,omitzero"`
}

// Node builds the stats.
func (s Stats) Node() tern.Node { return Build(tern.KindStats, s, nil) }

// Table is a table with priority-hidden columns and meter cells (kind table).
type Table struct {
	Common
	// Cols are the columns, left to right.
	Cols []Column `json:"cols,omitzero"`
	// Rows are the rows, top to bottom.
	Rows []TableRow `json:"rows,omitzero"`
	// Selectable makes rows take clicks: select on click, activate on
	// double click, each with Item the row id (OnSelect, OnActivate).
	Selectable bool `json:"selectable,omitzero"`
	// Selected is the id of the row drawn selected, kept in view.
	Selected string `json:"selected,omitzero"`
	// Empty shows, muted, while there are no rows.
	Empty Rich `json:"empty,omitzero"`
	// Sort is the column the rows are sorted by, drawn with its arrow; a
	// click on a Sortable header sends an action event with act "sort" and
	// Value {col, dir}.
	Sort *TableSort `json:"sort,omitzero"`
	// Multi picks rows several at a time: a click toggles one, a
	// shift-click a range, and each change sends change with Value the ids.
	Multi bool `json:"multi,omitzero"`
	// Picked are the ids of the picked rows (with Multi); an empty non-nil
	// list clears them.
	Picked []string `json:"picked,omitzero"`
	// Variant is how picks and the sort look (default TableCheck).
	Variant TableVariant `json:"variant,omitzero"`
	// Bar are buttons in the selection bar; a click sends an action event
	// with the button's act and Value the picked ids.
	Bar []BarAction `json:"bar,omitzero"`
	// Hint is muted text after the count in the selection bar.
	Hint []Span `json:"hint,omitzero"`
}

// TableSort is a table's sorted column.
type TableSort struct {
	// Col is the column id.
	Col string `json:"col"`
	// Dir is SortAsc or SortDesc.
	Dir SortDir `json:"dir"`
}

// SortDir is a sort direction.
type SortDir string

// Sort directions.
const (
	SortAsc  SortDir = "asc"
	SortDesc SortDir = "desc"
)

// TableVariant is how a table draws picks and the sort.
type TableVariant string

// Table looks.
const (
	// TableCheck is a leading checkbox column under a tri-state header box.
	TableCheck TableVariant = "check"
	// TableBar marks picked rows with an accent bar at the leading edge, no
	// checkboxes.
	TableBar TableVariant = "bar"
)

// BarAction is a small button in a table's selection bar.
type BarAction struct {
	// Act is the action a click sends (default "click").
	Act      string        `json:"act,omitzero"`
	Text     string        `json:"text,omitzero"`
	Icon     IconName      `json:"icon,omitzero"`
	Keys     []string      `json:"keys,omitzero"`
	Variant  ButtonVariant `json:"variant,omitzero"`
	Disabled Disabled      `json:"disabled,omitzero"`
	Title    string        `json:"title,omitzero"`
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
	// Mono draws the column's cells in the mono face (ids, hashes, paths).
	Mono bool `json:"mono,omitzero"`
	// Sortable makes the header take clicks that send an action event with
	// act "sort".
	Sortable bool `json:"sortable,omitzero"`
}

// TableRow is a table row.
type TableRow struct {
	// ID is the row's identity, unique in the table.
	ID string `json:"id"`
	// Cells hold a Cell per column id.
	Cells map[string]Cell `json:"cells,omitzero"`
	// Disabled dims the row; it takes no select or activate.
	Disabled Disabled `json:"disabled,omitzero"`
	// Menu lists right-click actions; each sends an action event with
	// Item the row id.
	Menu []string `json:"menu,omitzero"`
	// Title is the row's tooltip.
	Title string `json:"title,omitzero"`
}

// Cell is a table cell: Rich, MeterCell, TextCell, BadgeCell, ButtonCell
// or SparkCell.
type Cell interface {
	cell()
}

// TextCell is text led by a status dot or an icon.
type TextCell struct {
	// Text is the cell's text.
	Text Rich `json:"text"`
	// Icon leads the text, colored by Tone.
	Icon IconName `json:"icon,omitzero"`
	// Dot is a status dot of this tone before the text.
	Dot Tone `json:"dot,omitzero"`
	// Tone colors the icon.
	Tone Tone `json:"tone,omitzero"`
	// Title is the tooltip.
	Title string `json:"title,omitzero"`
}

func (TextCell) cell() {}

// BadgeCell is a pill chip in a table cell.
type BadgeCell struct {
	Text  string `json:"text"`
	Tone  Tone   `json:"tone,omitzero"`
	Title string `json:"title,omitzero"`
}

func (BadgeCell) cell() {}

// MarshalJSON wraps the badge in the cell's badge discriminator.
func (b BadgeCell) MarshalJSON() ([]byte, error) {
	type fields BadgeCell
	return tern.Marshal(struct {
		Badge fields `json:"badge"`
	}{Badge: fields(b)})
}

// ButtonCell is a small button in a table cell. A click sends an action
// event with act Act and Item the row id; it does not select the row.
type ButtonCell struct {
	Text     string        `json:"text"`
	Act      string        `json:"act"`
	Icon     IconName      `json:"icon,omitzero"`
	Keys     []string      `json:"keys,omitzero"`
	Variant  ButtonVariant `json:"variant,omitzero"`
	Disabled Disabled      `json:"disabled,omitzero"`
}

func (ButtonCell) cell() {}

// MarshalJSON wraps the button in the cell's button discriminator.
func (b ButtonCell) MarshalJSON() ([]byte, error) {
	type fields ButtonCell
	return tern.Marshal(struct {
		Button fields `json:"button"`
	}{Button: fields(b)})
}

// SparkCell is a sparkline in a table cell.
type SparkCell struct {
	// Values are the points, oldest first.
	Values []float64 `json:"values"`
	// Tone colors the line (default accent).
	Tone Tone `json:"tone,omitzero"`
	// Min and Max fix the scale (default the values' range).
	Min *float64 `json:"min,omitzero"`
	Max *float64 `json:"max,omitzero"`
	// Title is the tooltip.
	Title string `json:"title,omitzero"`
}

func (SparkCell) cell() {}

// MarshalJSON wraps the sparkline in the cell's spark discriminator.
func (s SparkCell) MarshalJSON() ([]byte, error) {
	type fields SparkCell
	return tern.Marshal(struct {
		Spark fields `json:"spark"`
	}{Spark: fields(s)})
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

// JSON is a JSON value Tern draws natively as a foldable tree (kind json).
// Check the hello reply's kinds for json and fall back to Tree otherwise.
type JSON struct {
	Common
	// Value is any JSON-marshalable value; absent reads as null.
	Value any `json:"value,omitzero"`
	// Open is how many levels of containers start expanded.
	Open int `json:"open,omitzero"`
}

// Node builds the json node.
func (j JSON) Node() tern.Node { return Build(tern.KindJSON, j, nil) }

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

// Media is a file Tern loads by path and shows in its media viewer (kind media).
type Media struct {
	Common
	// Path is the file: absolute, or relative to the pane's working directory.
	Path string `json:"path"`
	// Rev reads the file from git instead of disk (HEAD, a sha, ":0" for the index).
	Rev string `json:"rev,omitzero"`
	// Link makes media nodes sharing it pan and zoom together.
	Link string `json:"link,omitzero"`
}

// Node builds the media node.
func (m Media) Node() tern.Node { return Build(tern.KindMedia, m, nil) }

// Avatar is a person's, bot's or provider's mark: initials on a seeded
// gradient, a tinted squircle tile, an icon or a picture, with a status dot
// (kind avatar). Common.Title is its tooltip and accessible name (default
// Text).
type Avatar struct {
	Common
	// Variant is AvatarCircle (default) or AvatarSquare.
	Variant AvatarVariant `json:"variant,omitzero"`
	// Text is the name the mark stands for; its first two characters are
	// the initials.
	Text string `json:"text,omitzero"`
	// Seed picks the colors (default Text): pass a stable id.
	Seed string `json:"seed,omitzero"`
	// Icon is drawn instead of the initials (an icon name or provider:<id>).
	Icon IconName `json:"icon,omitzero"`
	// Blob is a picture's blob id, cropped to the face.
	Blob string `json:"blob,omitzero"`
	// Size is SizeXS, SizeSM, SizeMD (default) or SizeLG.
	Size Size `json:"size,omitzero"`
	// Status is a dot in the tone's color at the bottom right.
	Status Tone `json:"status,omitzero"`
}

// AvatarVariant is an avatar's look.
type AvatarVariant string

// Avatar looks.
const (
	// AvatarCircle is initials on a two-hue gradient, for people and providers.
	AvatarCircle AvatarVariant = "circle"
	// AvatarSquare is a tinted squircle tile, for bots, orgs and repositories.
	AvatarSquare AvatarVariant = "square"
)

// Node builds the avatar.
func (a Avatar) Node() tern.Node { return Build(tern.KindAvatar, a, nil) }

// Face is an avatar in a stack, or a picker's initials mark (which draws
// only Text and Seed).
type Face struct {
	// Text is the name the mark stands for; its initials are drawn.
	Text string `json:"text"`
	// Seed picks the colors (default Text).
	Seed string `json:"seed,omitzero"`
	// Variant is AvatarCircle (default) or AvatarSquare.
	Variant AvatarVariant `json:"variant,omitzero"`
	// Icon is drawn instead of the initials.
	Icon IconName `json:"icon,omitzero"`
	// Blob is a picture's blob id.
	Blob string `json:"blob,omitzero"`
	// Status is a status tone (not drawn in a stack).
	Status Tone `json:"status,omitzero"`
	// Title is the tooltip (default Text).
	Title string `json:"title,omitzero"`
}

// Avatars is an overlapping stack of avatars with a +N overflow chip
// (kind avatars). Its Max is a count and shadows Common's size bound.
type Avatars struct {
	Common
	// Items are the avatars, front first.
	Items []Face `json:"items,omitzero"`
	// Max is how many show before the +N chip (default all).
	Max int `json:"max,omitzero"`
	// Size is every avatar's size (default SizeMD).
	Size Size `json:"size,omitzero"`
	// Label is muted text after the stack, and the group's accessible name.
	Label string `json:"label,omitzero"`
}

// Node builds the avatars.
func (a Avatars) Node() tern.Node { return Build(tern.KindAvatars, a, nil) }

// Attachment is a file attached to a message or a form: a chip, a tile or
// a pill, with its upload state (kind attachment). Its buttons send action
// events: remove, retry, and each of Actions' acts. Common.Href is opened by
// a click.
type Attachment struct {
	CommonWithoutActions
	// Name is the file name, also the tooltip and accessible name.
	Name string `json:"name,omitzero"`
	// Variant is AttachmentChip (default), AttachmentTile or AttachmentPill.
	Variant AttachmentVariant `json:"variant,omitzero"`
	// Ext is the extension shown (default from Name, else Mime).
	Ext string `json:"ext,omitzero"`
	// Mime is the MIME type; it picks the family when the extension says nothing.
	Mime string `json:"mime,omitzero"`
	// Size is the size in bytes, shown in the default meta.
	Size *int64 `json:"size,omitzero"`
	// Meta replaces the meta line ("EXT · size").
	Meta string `json:"meta,omitzero"`
	// Blob is an image blob id, the thumbnail of an image file.
	Blob string `json:"blob,omitzero"`
	// Path is a file opened by a click when there is no Href.
	Path string `json:"path,omitzero"`
	// State is the upload state (default AttachmentIdle).
	State AttachmentState `json:"state,omitzero"`
	// Progress is 0–1 while busy; without it the line sweeps.
	Progress *float64 `json:"progress,omitzero"`
	// Error is the meta line in AttachmentError (default "Upload failed").
	Error string `json:"error,omitzero"`
	// Removable adds a × that sends remove.
	Removable bool `json:"removable,omitzero"`
	// Actions are icon buttons before the ×, in order.
	Actions []IconAction `json:"actions,omitzero"`
}

// AttachmentVariant is an attachment's look.
type AttachmentVariant string

// Attachment looks.
const (
	AttachmentChip AttachmentVariant = "chip"
	AttachmentTile AttachmentVariant = "tile"
	AttachmentPill AttachmentVariant = "pill"
)

// AttachmentState is an attachment's upload state.
type AttachmentState string

// Upload states.
const (
	AttachmentIdle       AttachmentState = "idle"
	AttachmentUploading  AttachmentState = "uploading"
	AttachmentProcessing AttachmentState = "processing"
	AttachmentError      AttachmentState = "error"
	AttachmentDone       AttachmentState = "done"
)

// Node builds the attachment.
func (a Attachment) Node() tern.Node { return Build(tern.KindAttachment, a, nil) }

// Carousel shows its children one slide at a time (kind carousel). A move
// by the user sends change with the index now shown.
type Carousel struct {
	Common
	// Index is the slide shown, from 0; a change wins over the user's.
	Index int `json:"index,omitzero"`
	// Loop wraps around past either end.
	Loop bool `json:"loop,omitzero"`
	// Variant is CarouselPeek (default), CarouselArrows or CarouselStrip.
	Variant CarouselVariant `json:"variant,omitzero"`
	// H is the slides' height: Px (default 200), Lines or Ch.
	H Extent `json:"h,omitzero"`
	// Children are the slides, in order.
	Children []tern.Element `json:"-"`
}

// CarouselVariant is a carousel's look.
type CarouselVariant string

// Carousel looks.
const (
	CarouselPeek   CarouselVariant = "peek"
	CarouselArrows CarouselVariant = "arrows"
	CarouselStrip  CarouselVariant = "strip"
)

// Node builds the carousel.
func (c Carousel) Node() tern.Node { return Build(tern.KindCarousel, c, c.Children) }
