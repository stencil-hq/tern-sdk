//! Structured prop values: table columns and rows, tree items, list and
//! picker entries, meter parts, tool and agent facts, checklist phases,
//! `prefs` pages, menu entries, and the items of the newer kinds (options,
//! crumbs, nav items, trace lines, stack frames, questions). Every field is
//! public; optional ones are omitted from the wire when `None` or empty.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use super::{
	icon::Icon,
	types::{
		AvatarVariant, ButtonVariant, ChoiceStyle, ColumnFormat, Disabled, ItemStatus, SortDir,
		Status, Text, TextAlign, Tone, Truncate,
	},
};

/// One `kv` pair.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct KvItem {
	/// The key.
	pub k: Text,
	/// The value.
	pub v: Text,
}

/// A `table` column.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Column {
	/// The key of this column's cell in each row.
	pub id:       String,
	/// Header text.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub head:     Option<Text>,
	/// Alignment of the header and cells.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub align:    Option<TextAlign>,
	/// Where a cell that doesn't fit is cut.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub truncate: Option<Truncate>,
	/// Lower hides first when the table is narrow.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub priority: Option<f64>,
	/// Share of the spare width.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub grow:     Option<f64>,
	/// Cells in the mono face (ids, hashes, numbers).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mono:     Option<bool>,
	/// Its header takes clicks and sends `action` `sort`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sortable: Option<bool>,
}

impl Column {
	/// A column with header `head`.
	pub fn new(id: impl Into<String>, head: impl Into<Text>) -> Self {
		Self { id: id.into(), head: Some(head.into()), ..Self::default() }
	}

	/// Sets the alignment.
	pub fn align(mut self, align: impl Into<TextAlign>) -> Self {
		self.align = Some(align.into());
		self
	}

	/// Sets the grow share.
	pub const fn grow(mut self, grow: f64) -> Self {
		self.grow = Some(grow);
		self
	}

	/// Sets the hide priority.
	pub const fn priority(mut self, priority: f64) -> Self {
		self.priority = Some(priority);
		self
	}

	/// Sets where cells are cut.
	pub fn truncate(mut self, truncate: impl Into<Truncate>) -> Self {
		self.truncate = Some(truncate.into());
		self
	}

	/// Draws the cells in the mono face.
	pub const fn mono(mut self, mono: bool) -> Self {
		self.mono = Some(mono);
		self
	}

	/// Makes the header sort the table on click.
	pub const fn sortable(mut self, sortable: bool) -> Self {
		self.sortable = Some(sortable);
		self
	}
}

/// A stacked part of a meter or meter cell.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Part {
	/// The part's share.
	pub value: f64,
	/// Its color token.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub token: Option<String>,
	/// Its tooltip label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// Diagonal hatching.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hatch: Option<bool>,
}

/// Fill levels that tint a meter.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Thresholds {
	/// `warn` at or above this.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub warn: Option<f64>,
	/// `bad` at or above this.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub bad:  Option<f64>,
}

/// A table cell drawn as a meter.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct MeterCell {
	/// The fill, 0–1.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub value:      Option<f64>,
	/// Stacked fills.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub parts:      Vec<Part>,
	/// Tint levels.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub thresholds: Option<Thresholds>,
	/// The fill's tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:       Option<Tone>,
	/// The tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:      Option<String>,
}

/// A table cell: text with a status dot or icon before it.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TextCell {
	/// The text.
	pub text:  Text,
	/// An icon before the text, colored by `tone`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:  Option<Icon>,
	/// A status dot before the text, in this tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dot:   Option<Tone>,
	/// The icon's color.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// The tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
}

impl TextCell {
	/// A cell of `text`.
	pub fn new(text: impl Into<Text>) -> Self {
		Self { text: text.into(), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the status dot's tone.
	pub fn dot(mut self, tone: impl Into<Tone>) -> Self {
		self.dot = Some(tone.into());
		self
	}

	/// Sets the icon's tone.
	pub fn tone(mut self, tone: impl Into<Tone>) -> Self {
		self.tone = Some(tone.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}
}

/// A table cell drawn as a sm `button`. Its click sends an `action` event
/// with `act` and the row as `item`, and doesn't select the row.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ButtonCell {
	/// The label.
	pub text:     String,
	/// The action name sent.
	pub act:      String,
	/// A leading icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Keycaps after the label.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub keys:     Vec<String>,
	/// The chrome.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub variant:  Option<ButtonVariant>,
	/// Dimmed and silent; a reason is the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Disabled>,
}

impl ButtonCell {
	/// A button labeled `text` sending `act`.
	pub fn new(text: impl Into<String>, act: impl Into<String>) -> Self {
		Self { text: text.into(), act: act.into(), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the keycaps.
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.keys = keys.into_iter().map(Into::into).collect();
		self
	}

	/// Sets the chrome.
	pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
		self.variant = Some(variant.into());
		self
	}

	/// Disables it (`true` or a reason).
	pub fn disabled(mut self, disabled: impl Into<Disabled>) -> Self {
		self.disabled = Some(disabled.into());
		self
	}
}

/// A table cell drawn as a sparkline.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct SparkCell {
	/// The values, oldest first.
	pub values: Vec<f64>,
	/// The line's tone (default accent).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:   Option<Tone>,
	/// The bottom of the scale (default the smallest value).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min:    Option<f64>,
	/// The top of the scale (default the largest value).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max:    Option<f64>,
	/// The tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:  Option<String>,
}

impl SparkCell {
	/// A sparkline of `values`.
	pub fn new(values: impl Into<Vec<f64>>) -> Self {
		Self { values: values.into(), ..Self::default() }
	}

	/// Sets the tone.
	pub fn tone(mut self, tone: impl Into<Tone>) -> Self {
		self.tone = Some(tone.into());
		self
	}

	/// Sets the scale.
	pub const fn range(mut self, min: f64, max: f64) -> Self {
		self.min = Some(min);
		self.max = Some(max);
		self
	}
}

/// A table cell: spans, a meter, decorated text, a badge, a button or a
/// sparkline.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
	/// Text.
	Text(Text),
	/// A meter.
	Meter(MeterCell),
	/// Text with a dot or icon.
	Rich(TextCell),
	/// A badge.
	Badge(BadgeSpec),
	/// A sm button.
	Button(ButtonCell),
	/// A sparkline.
	Spark(SparkCell),
}

impl Serialize for Cell {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		fn one<S: Serializer>(s: S, key: &str, value: &impl Serialize) -> Result<S::Ok, S::Error> {
			let mut map = s.serialize_map(Some(1))?;
			map.serialize_entry(key, value)?;
			map.end()
		}
		match self {
			Self::Text(t) => t.serialize(s),
			Self::Rich(c) => c.serialize(s),
			Self::Meter(m) => one(s, "meter", m),
			Self::Badge(b) => one(s, "badge", b),
			Self::Button(b) => one(s, "button", b),
			Self::Spark(c) => one(s, "spark", c),
		}
	}
}

impl<T: Into<Text>> From<T> for Cell {
	fn from(t: T) -> Self {
		Self::Text(t.into())
	}
}

impl From<MeterCell> for Cell {
	fn from(m: MeterCell) -> Self {
		Self::Meter(m)
	}
}

impl From<TextCell> for Cell {
	fn from(c: TextCell) -> Self {
		Self::Rich(c)
	}
}

impl From<BadgeSpec> for Cell {
	fn from(b: BadgeSpec) -> Self {
		Self::Badge(b)
	}
}

impl From<ButtonCell> for Cell {
	fn from(b: ButtonCell) -> Self {
		Self::Button(b)
	}
}

impl From<SparkCell> for Cell {
	fn from(c: SparkCell) -> Self {
		Self::Spark(c)
	}
}

/// A `table` row.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TableRow {
	/// Row identity, unique in the table.
	pub id:       String,
	/// One cell per column id.
	pub cells:    BTreeMap<String, Cell>,
	/// Dimmed, can't be selected or activated; a reason is the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Disabled>,
	/// Right-click actions, each sent as an `action` with the row as `item`.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub menu:     Vec<String>,
	/// The row's tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:    Option<String>,
}

impl TableRow {
	/// An empty row.
	pub fn new(id: impl Into<String>) -> Self {
		Self { id: id.into(), ..Self::default() }
	}

	/// Disables the row (`true` or a reason).
	pub fn disabled(mut self, disabled: impl Into<Disabled>) -> Self {
		self.disabled = Some(disabled.into());
		self
	}

	/// Adds a right-click action.
	pub fn menu(mut self, act: impl Into<String>) -> Self {
		self.menu.push(act.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Sets the cell of column `col`.
	pub fn cell(mut self, col: impl Into<String>, cell: impl Into<Cell>) -> Self {
		self.cells.insert(col.into(), cell.into());
		self
	}
}

/// A `tree` item.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TreeItem {
	/// Identity, unique in the tree.
	pub id:       String,
	/// The row's text.
	pub label:    Text,
	/// An icon before the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Whether the children show.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub open:     Option<bool>,
	/// Nested items.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub children: Vec<Self>,
}

impl TreeItem {
	/// An item labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<Text>) -> Self {
		Self { id: id.into(), label: label.into(), ..Self::default() }
	}

	/// Adds a child item.
	pub fn child(mut self, item: Self) -> Self {
		self.children.push(item);
		self
	}

	/// Sets whether the children show.
	pub const fn open(mut self, open: bool) -> Self {
		self.open = Some(open);
		self
	}
}

/// A tab of a `tabs` strip.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Tab {
	/// The tab's id.
	pub id:    String,
	/// Its label.
	pub label: Text,
}

/// A code line mark.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct CodeMark {
	/// The line, in the gutter's numbering.
	pub line:   u64,
	/// The tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:   Option<Tone>,
	/// A score 0–1 on the heat ramp: the overview's bar, the fold sparkline,
	/// and the row's wash without `wash`; with no `tone`, no bar.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub heat:   Option<f64>,
	/// Character ranges `[from, to)` marked on the line.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub ranges: Vec<[u64; 2]>,
	/// Scores 0–1 washing the row on the heat ramp: one flat, several a
	/// gradient across it, left to right; in place of `heat`'s wash.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub wash:   Vec<f64>,
}

/// A run of lines a code block draws as one row until it is opened.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CodeFold {
	/// The first folded line, in the gutter's numbering.
	pub from:      u64,
	/// The last folded line, in the gutter's numbering.
	pub to:        u64,
	/// The folded row's text, in the code's face (default `… [lines
	/// {from}-{to}]`, `… [line {from}]` for one line).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:     Option<String>,
	/// What the fold hides, after a sparkline of its lines' heat.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub note:      Option<Text>,
	/// `error` tints the row; any tone colors the note.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:      Option<Tone>,
	/// Drawn open; a click on the folded row (or an open fold's head)
	/// toggles it in place.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub open:      Option<bool>,
	/// A card shown while the pointer rests on the folded row: an id-less
	/// node tree, as [`Node::to_json`](super::Node::to_json) gives.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hovercard: Option<Value>,
}

/// A diff hunk given directly.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hunk {
	/// The old side's first line.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub old_start: Option<u64>,
	/// The new side's first line.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub new_start: Option<u64>,
	/// Lines starting with `+`, `-` or a space.
	pub lines:     Vec<String>,
}

/// A styled range of an editor's text (UTF-16 offsets).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Decor {
	/// Start.
	pub from: usize,
	/// End.
	pub to:   usize,
	/// Style tokens.
	pub s:    String,
	/// An effect.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fx:   Option<super::types::Fx>,
}

/// A shimmer's sweep colors, as span tokens.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ShimmerPalette {
	/// The low color.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub low:  Option<String>,
	/// The middle color.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mid:  Option<String>,
	/// The high color.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub high: Option<String>,
}

/// A tick on a meter.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct MeterMark {
	/// Position, 0–1.
	pub at:    f64,
	/// Tick color.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// Tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	/// An icon on the track.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:  Option<Icon>,
}

/// One bar of a `bars` or `spark` chart, one point of a one-series `line`
/// or `area` chart, or one segment of a `donut`.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Point {
	/// The value.
	pub value: f64,
	/// A label (`line`/`area`: the point's x label; `donut`: its legend row).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// A tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	/// This bar's fill instead of the chart's `token`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// `donut`: this segment's color token.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub token: Option<String>,
}

/// One line of a multi-series `line` or `area` chart.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Series {
	/// Names its legend chip and its part of the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:  Option<String>,
	/// Its color token (default the next chart color).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub token:  Option<String>,
	/// One value per x position; `None` is a gap where the line breaks.
	pub values: Vec<Option<f64>>,
}

impl Series {
	/// A series of `values`.
	pub fn new(values: impl Into<Vec<Option<f64>>>) -> Self {
		Self { values: values.into(), ..Self::default() }
	}

	/// Sets the legend label.
	pub fn label(mut self, label: impl Into<String>) -> Self {
		self.label = Some(label.into());
		self
	}

	/// Sets the color token.
	pub fn token(mut self, token: impl Into<String>) -> Self {
		self.token = Some(token.into());
		self
	}
}

/// A dashed reference line (a target, a limit) across a `bars`, `spark`,
/// `line` or `area` chart. Outside the domain it is clamped to its edge.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ChartMark {
	/// Where the line sits, in value units.
	pub value: f64,
	/// The line's color (default muted foreground).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// The line's tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
}

/// A heatmap column label.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ChartCol {
	/// The 0-based column.
	pub at:    u32,
	/// The label.
	pub label: String,
}

/// Where an overlay floats.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Anchor {
	/// Centered.
	Center,
	/// At the top.
	Top,
	/// At the bottom.
	Bottom,
	/// Beside node `id`, below it unless `above`.
	Node {
		/// The node.
		id:    String,
		/// Above instead of below.
		above: bool,
	},
	/// Below the caret of field `id`.
	Caret(String),
}

impl Serialize for Anchor {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeMap as _;
		match self {
			Self::Center => s.serialize_str("center"),
			Self::Top => s.serialize_str("top"),
			Self::Bottom => s.serialize_str("bottom"),
			Self::Node { id, above } => {
				let mut m = s.serialize_map(Some(2))?;
				m.serialize_entry("node", id)?;
				m.serialize_entry("side", if *above { "above" } else { "below" })?;
				m.end()
			},
			Self::Caret(id) => {
				let mut m = s.serialize_map(Some(1))?;
				m.serialize_entry("caret", id)?;
				m.end()
			},
		}
	}
}

/// A badge in a tool's or agent's head, or after a picker row's label.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct BadgeSpec {
	/// The text.
	pub text:  String,
	/// The tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// A tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
}

impl BadgeSpec {
	/// A badge reading `text`.
	pub fn new(text: impl Into<String>) -> Self {
		Self { text: text.into(), ..Self::default() }
	}

	/// Sets the tone.
	pub fn tone(mut self, tone: impl Into<Tone>) -> Self {
		self.tone = Some(tone.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}
}

/// One tile of a `stats` well.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StatItem {
	/// What is counted.
	pub label: Text,
	/// The figure.
	pub value: Text,
	/// A note under the figure.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sub:   Option<Text>,
	/// Colors the figure.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:  Option<Tone>,
	/// The tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
}

impl StatItem {
	/// A tile showing `value` under `label`.
	pub fn new(label: impl Into<Text>, value: impl Into<Text>) -> Self {
		Self { label: label.into(), value: value.into(), ..Self::default() }
	}

	/// Sets the note under the figure.
	#[expect(clippy::should_implement_trait, reason = "named after the wire prop `sub`")]
	pub fn sub(mut self, sub: impl Into<Text>) -> Self {
		self.sub = Some(sub.into());
		self
	}

	/// Sets the figure's tone.
	pub fn tone(mut self, tone: impl Into<Tone>) -> Self {
		self.tone = Some(tone.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}
}

/// A hover button on a tool.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ToolButton {
	/// The action id.
	pub id:    String,
	/// The label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// Keycaps.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub keys:  Vec<String>,
}

/// The clamp of a collapsed tool body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolPreview {
	/// The first lines.
	Lines(u32),
	/// The last lines.
	Tail(u32),
}

impl Serialize for ToolPreview {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeMap as _;
		let mut m = s.serialize_map(Some(1))?;
		match *self {
			Self::Lines(n) => m.serialize_entry("lines", &n)?,
			Self::Tail(n) => m.serialize_entry("tail", &n)?,
		}
		m.end()
	}
}

/// Lines a collapsed card shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preview {
	/// Ten lines.
	Auto,
	/// This many lines.
	Lines(u32),
}

impl Serialize for Preview {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeMap as _;
		match *self {
			Self::Auto => s.serialize_str("auto"),
			Self::Lines(n) => {
				let mut m = s.serialize_map(Some(1))?;
				m.serialize_entry("lines", &n)?;
				m.end()
			},
		}
	}
}

/// An agent's stats.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStats {
	/// Tool calls.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tools:         Option<u64>,
	/// Requests.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub requests:      Option<u64>,
	/// Fraction done, 0–1.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub done:          Option<f64>,
	/// Context used, 0–1.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub context:       Option<f64>,
	/// The context meter's label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub context_label: Option<String>,
	/// Tokens used.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tokens:        Option<u64>,
	/// Cost in dollars.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cost:          Option<f64>,
	/// Elapsed ms when sent.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age:           Option<i64>,
	/// Final duration in ms.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub took:          Option<u64>,
}

/// The tool an agent runs now.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AgentTool {
	/// The tool's name.
	pub name:   String,
	/// What it does.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub intent: Option<String>,
	/// Elapsed ms when sent.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age:    Option<i64>,
}

/// An agent's retry countdown.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Retry {
	/// The attempt.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub attempt: Option<u32>,
	/// Attempts allowed.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub max:     Option<u32>,
	/// The delay in ms.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub delay:   Option<u64>,
	/// Elapsed ms when sent.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age:     Option<i64>,
	/// The error retried.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub error:   Option<String>,
}

/// A checklist item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ChecklistItem {
	/// Item id.
	pub id:     String,
	/// The text.
	pub text:   Text,
	/// The state.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status: Option<ItemStatus>,
	/// A line under the item.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub note:   Option<Text>,
}

/// A checklist phase.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Phase {
	/// Phase id.
	pub id:        String,
	/// The title.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:     Option<Text>,
	/// The items.
	pub items:     Vec<ChecklistItem>,
	/// The initial fold.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub collapsed: Option<bool>,
}

/// A picker catalog item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PickerItem {
	/// Item id.
	pub id:       String,
	/// Row text.
	pub label:    Text,
	/// Text after the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail:   Option<Text>,
	/// Lead icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Picks the lead icon from a role.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub role:     Option<String>,
	/// An initials avatar `{text, seed?}`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mark:     Option<Value>,
	/// A status dot.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dot:      Option<Tone>,
	/// Monospace label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mono:     Option<bool>,
	/// Column values by column id.
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	pub facts:    BTreeMap<String, Value>,
	/// Badges after the label.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub badges:   Vec<BadgeSpec>,
	/// Role chips `{text, on?, auto?, dot?}`.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub chips:    Vec<Value>,
	/// The row's tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:     Option<Tone>,
	/// `true` or a reason: dims the row.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Value>,
	/// Match ranges `[from, to)` in the label (UTF-16).
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub hits:     Vec<[usize; 2]>,
	/// `timeline` node style.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub node:     Option<String>,
	/// `tree` depth.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub depth:    Option<u32>,
	/// `tree` chevron state.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub open:     Option<bool>,
	/// Row tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:    Option<String>,
}

impl PickerItem {
	/// An item labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<Text>) -> Self {
		Self { id: id.into(), label: label.into(), ..Self::default() }
	}
}

/// A picker `order` entry: an item id or a group header.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OrderEntry {
	/// An item id.
	Item(String),
	/// A group header.
	Group {
		/// The group's key.
		group: String,
		/// Its label.
		label: String,
		/// A count.
		count: Option<u64>,
	},
}

impl Serialize for OrderEntry {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeMap as _;
		match self {
			Self::Item(id) => s.serialize_str(id),
			Self::Group { group, label, count } => {
				let mut m = s.serialize_map(None)?;
				m.serialize_entry("group", group)?;
				m.serialize_entry("label", label)?;
				if let Some(count) = count {
					m.serialize_entry("count", count)?;
				}
				m.end()
			},
		}
	}
}

/// A picker fact column.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PickerColumn {
	/// The `facts` key.
	pub id:       String,
	/// Head label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub head:     Option<String>,
	/// Format.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub format:   Option<ColumnFormat>,
	/// Lower hides first.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub priority: Option<f64>,
	/// Minimum width in characters.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub min:      Option<u32>,
}

/// A picker confirm strip.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Confirm {
	/// The question.
	pub text:  String,
	/// The action sent (default `confirm`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub act:   Option<String>,
	/// The button label (default `Delete`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
}

/// A picker scope.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Scope {
	/// Scope id.
	pub id:       String,
	/// Label.
	pub label:    String,
	/// Group header.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub group:    Option<String>,
	/// Initials avatar `{text, seed?}`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mark:     Option<Value>,
	/// Icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Status dot.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dot:      Option<Tone>,
	/// Count.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub count:    Option<u64>,
	/// `true` or a reason.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Value>,
}

/// A picker tab.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PickerTab {
	/// Tab id.
	pub id:    String,
	/// Label.
	pub label: String,
	/// Count.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub count: Option<u64>,
}

/// A switch chip of a picker strip.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StripItem {
	/// Chip id.
	pub id:    String,
	/// Label.
	pub label: String,
	/// Checked.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub on:    Option<bool>,
	/// Dot palette name.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dot:   Option<String>,
}

/// A picker's chip strip.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Strip {
	/// Label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:    Option<String>,
	/// Chips.
	pub items:    Vec<StripItem>,
	/// Outlined chip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub selected: Option<String>,
}

/// A picker action bar button.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PickerAction {
	/// Action id.
	pub id:       String,
	/// Label (default the id).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:    Option<String>,
	/// Keycaps.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub keys:     Vec<String>,
	/// The primary action.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub primary:  Option<bool>,
	/// At the right end.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub end:      Option<bool>,
	/// Destructive.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub danger:   Option<bool>,
	/// Toggled on.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub on:       Option<bool>,
	/// `true` or a reason.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Value>,
}

/// A `prefs` nav page.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PrefsPage {
	/// Page id.
	pub id:       String,
	/// Label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:    Option<String>,
	/// Group heading.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub group:    Option<String>,
	/// Icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Why it is disabled.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<String>,
	/// Rows changed from default.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub changed:  Option<u32>,
}

/// A `prefs` section.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct PrefsSection {
	/// Section id.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id:    Option<String>,
	/// Title.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	/// The page it belongs to (search results).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub page:  Option<String>,
	/// The rows.
	pub rows:  Vec<PrefsRow>,
}

/// A `prefs` row.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefsRow {
	/// Row id.
	pub id:            String,
	/// Label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:         Option<String>,
	/// A line under the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub hint:          Option<String>,
	/// A warning.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub warning:       Option<String>,
	/// Why it is disabled.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled:      Option<String>,
	/// Changed from default.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub changed:       Option<bool>,
	/// The default, named in the changed dot's tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub default_label: Option<String>,
	/// The control.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub control:       Option<Control>,
}

/// An option of a `prefs` choice or multi control.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ChoiceOption {
	/// The value.
	pub value:  String,
	/// The label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:  Option<String>,
	/// A tooltip or second line.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail: Option<String>,
}

/// A `prefs` row's control, by `k`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "k", rename_all = "lowercase")]
pub enum Control {
	/// A toggle.
	Switch {
		/// On.
		on: bool,
	},
	/// One of several options.
	Choice {
		/// The current value.
		value:   String,
		/// The options.
		options: Vec<ChoiceOption>,
		/// How it draws.
		#[serde(skip_serializing_if = "Option::is_none")]
		style:   Option<ChoiceStyle>,
		/// Monospace labels.
		#[serde(skip_serializing_if = "Option::is_none")]
		mono:    Option<bool>,
	},
	/// A stepper.
	Number {
		/// The value.
		value:  f64,
		/// Minimum.
		#[serde(skip_serializing_if = "Option::is_none")]
		min:    Option<f64>,
		/// Maximum.
		#[serde(skip_serializing_if = "Option::is_none")]
		max:    Option<f64>,
		/// Step (default 1).
		#[serde(skip_serializing_if = "Option::is_none")]
		step:   Option<f64>,
		/// Unit.
		#[serde(skip_serializing_if = "Option::is_none")]
		unit:   Option<String>,
		/// Labels by value.
		#[serde(skip_serializing_if = "BTreeMap::is_empty")]
		labels: BTreeMap<String, String>,
	},
	/// A text field look.
	Text {
		/// The value.
		value:       String,
		/// Placeholder.
		#[serde(skip_serializing_if = "Option::is_none")]
		placeholder: Option<String>,
		/// Dots instead of text.
		#[serde(skip_serializing_if = "Option::is_none")]
		secret:      Option<bool>,
		/// Monospace.
		#[serde(skip_serializing_if = "Option::is_none")]
		mono:        Option<bool>,
	},
	/// Keycap chords.
	Keys {
		/// Chords, each a list of key names.
		keys: Vec<Vec<String>>,
	},
	/// Toggle chips or an ordered list.
	Multi {
		/// Enabled values.
		values:  Vec<String>,
		/// The options.
		options: Vec<ChoiceOption>,
		/// A reorderable list.
		#[serde(skip_serializing_if = "Option::is_none")]
		ordered: Option<bool>,
	},
	/// A button.
	Action {
		/// The action (default `edit`).
		#[serde(skip_serializing_if = "Option::is_none")]
		act:   Option<String>,
		/// The label.
		#[serde(skip_serializing_if = "Option::is_none")]
		label: Option<String>,
	},
}

/// The `prefs` row being edited with the keyboard.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Editing {
	/// The row.
	pub row:    String,
	/// The highlighted option.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub option: Option<String>,
	/// A text row's draft.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub draft:  Option<String>,
	/// The caret in the draft (UTF-16).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub cursor: Option<usize>,
}

/// An entry of a `menu`, of a `menubar` menu or of `actions.menu`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuEntry {
	/// An action labeled from its id (`open-file` → "Open file").
	Act(String),
	/// An item: label, icon, keys, a check or radio, a submenu.
	Item(MenuItem),
	/// A dim section title.
	Header(String),
	/// A separator line.
	Separator,
}

impl MenuEntry {
	/// A section title.
	pub fn header(title: impl Into<String>) -> Self {
		Self::Header(title.into())
	}
}

impl Serialize for MenuEntry {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Act(act) => s.serialize_str(act),
			Self::Item(item) => item.serialize(s),
			Self::Header(title) => {
				let mut map = s.serialize_map(Some(1))?;
				map.serialize_entry("header", title)?;
				map.end()
			},
			Self::Separator => s.serialize_str("-"),
		}
	}
}

impl From<&str> for MenuEntry {
	fn from(act: &str) -> Self {
		Self::Act(act.to_owned())
	}
}

impl From<String> for MenuEntry {
	fn from(act: String) -> Self {
		Self::Act(act)
	}
}

impl From<MenuItem> for MenuEntry {
	fn from(item: MenuItem) -> Self {
		Self::Item(item)
	}
}

/// A menu row. Choosing it sends `action` with `id` as `act` (and the new
/// `checked` of a check or radio row).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MenuItem {
	/// What the `action` event sends as `act`.
	pub id:       String,
	/// The row's text (default from `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:    Option<String>,
	/// A leading icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// Shortcut keys drawn at the right (display only).
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub keys:     Vec<String>,
	/// A second line under the label (a rich row).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail:   Option<String>,
	/// Makes it a check row, checked while `true`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub checked:  Option<bool>,
	/// Makes it a radio row of this group.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub radio:    Option<String>,
	/// Dimmed, not choosable; a reason is the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Disabled>,
	/// A destructive action, drawn in `--bad`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub danger:   Option<bool>,
	/// A submenu (the row itself sends nothing).
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub items:    Vec<MenuEntry>,
}

impl MenuItem {
	/// A row sending `id`, labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the shortcut keys.
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.keys = keys.into_iter().map(Into::into).collect();
		self
	}

	/// Sets the second line.
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Makes it a check row.
	pub const fn checked(mut self, checked: bool) -> Self {
		self.checked = Some(checked);
		self
	}

	/// Makes it a radio row of `group`.
	pub fn radio(mut self, group: impl Into<String>) -> Self {
		self.radio = Some(group.into());
		self
	}

	/// Disables it (`true` or a reason).
	pub fn disabled(mut self, disabled: impl Into<Disabled>) -> Self {
		self.disabled = Some(disabled.into());
		self
	}

	/// Draws it as destructive.
	pub const fn danger(mut self, danger: bool) -> Self {
		self.danger = Some(danger);
		self
	}

	/// Adds an entry to its submenu.
	pub fn item(mut self, entry: impl Into<MenuEntry>) -> Self {
		self.items.push(entry.into());
		self
	}
}

/// A menu of a `menubar`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct MenubarMenu {
	/// Names the menu in events and in `open`.
	pub id:    String,
	/// Its title (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// Its entries.
	pub items: Vec<MenuEntry>,
}

impl MenubarMenu {
	/// A menu titled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Adds an entry.
	pub fn item(mut self, entry: impl Into<MenuEntry>) -> Self {
		self.items.push(entry.into());
		self
	}
}

/// A button that sends `act`: in a table's selection bar (`bar`, sent with
/// the picked ids as `value`), or attached to an `input` or `editor`
/// (`buttons`; `"copy"` copies the text).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct ActButton {
	/// The action a click sends.
	pub act:      String,
	/// The label (without one the button is icon-only).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub text:     Option<String>,
	/// A leading icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// The chrome.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub variant:  Option<ButtonVariant>,
	/// Keycaps after the label.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub keys:     Vec<String>,
	/// The tooltip (and an icon-only button's accessible name).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:    Option<String>,
	/// Faded and inert; a reason is the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<Disabled>,
	/// A spinner in the icon's place; inert.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub busy:     Option<bool>,
}

impl ActButton {
	/// A button labeled `text` sending `act`.
	pub fn new(act: impl Into<String>, text: impl Into<String>) -> Self {
		Self { act: act.into(), text: Some(text.into()), ..Self::default() }
	}

	/// An icon-only button sending `act`.
	pub fn icon_only(act: impl Into<String>, icon: impl Into<Icon>) -> Self {
		Self { act: act.into(), icon: Some(icon.into()), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the chrome.
	pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
		self.variant = Some(variant.into());
		self
	}

	/// Sets the keycaps.
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.keys = keys.into_iter().map(Into::into).collect();
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Disables it (`true` or a reason).
	pub fn disabled(mut self, disabled: impl Into<Disabled>) -> Self {
		self.disabled = Some(disabled.into());
		self
	}

	/// Shows a spinner; inert.
	pub const fn busy(mut self, busy: bool) -> Self {
		self.busy = Some(busy);
		self
	}
}

/// An icon button of a `message` or an `attachment` (`actions`).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct IconAction {
	/// The action a click runs.
	pub act:   String,
	/// The icon (default the act's own).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:  Option<Icon>,
	/// The tooltip and accessible name (default the act, capitalized).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
}

impl IconAction {
	/// A button running `act`.
	pub fn new(act: impl Into<String>) -> Self {
		Self { act: act.into(), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}
}

/// The column a `table`'s rows are sorted by.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Sort {
	/// The column's id.
	pub col: String,
	/// The direction.
	pub dir: SortDir,
}

impl Sort {
	/// Sorted by `col`, ascending.
	pub fn asc(col: impl Into<String>) -> Self {
		Self { col: col.into(), dir: SortDir::Asc }
	}

	/// Sorted by `col`, descending.
	pub fn desc(col: impl Into<String>) -> Self {
		Self { col: col.into(), dir: SortDir::Desc }
	}
}

/// A scope of a tool's approval (how long the yes lasts).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AskScope {
	/// Sent as the approval actions' `value`.
	pub id:    String,
	/// The radio's label (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
}

impl AskScope {
	/// A scope labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()) }
	}
}

/// The Always button of a tool's approval card.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AskAlways {
	/// Shown with this label.
	Label(String),
	/// Hidden.
	Hidden,
}

impl Serialize for AskAlways {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Label(label) => s.serialize_str(label),
			Self::Hidden => s.serialize_bool(false),
		}
	}
}

impl From<&str> for AskAlways {
	fn from(label: &str) -> Self {
		Self::Label(label.to_owned())
	}
}

impl From<String> for AskAlways {
	fn from(label: String) -> Self {
		Self::Label(label)
	}
}

/// The contents of a tool's approval panel (`ask`), drawn while its status
/// is `ask`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Ask {
	/// The question.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub text:   Option<String>,
	/// The Allow button's label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub allow:  Option<String>,
	/// The Deny button's label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub deny:   Option<String>,
	/// The Always button (`card` only).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub always: Option<AskAlways>,
	/// The radios of the `scopes` look (default once, session, always).
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub scopes: Vec<AskScope>,
	/// The scope selected, sent as the actions' `value`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub scope:  Option<String>,
}

impl Ask {
	/// A panel asking `text`.
	pub fn new(text: impl Into<String>) -> Self {
		Self { text: Some(text.into()), ..Self::default() }
	}

	/// Sets the Allow button's label.
	pub fn allow(mut self, label: impl Into<String>) -> Self {
		self.allow = Some(label.into());
		self
	}

	/// Sets the Deny button's label.
	pub fn deny(mut self, label: impl Into<String>) -> Self {
		self.deny = Some(label.into());
		self
	}

	/// Sets the Always button's label, or hides it.
	pub fn always(mut self, always: impl Into<AskAlways>) -> Self {
		self.always = Some(always.into());
		self
	}

	/// Adds a scope radio.
	pub fn scope_option(mut self, scope: AskScope) -> Self {
		self.scopes.push(scope);
		self
	}

	/// Selects scope `id`.
	pub fn scope(mut self, id: impl Into<String>) -> Self {
		self.scope = Some(id.into());
		self
	}
}

/// An option of a `select`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct SelectOption {
	/// What `value`, `active` and `change` carry.
	pub value:    String,
	/// What the option (and the trigger, when current) shows (default the
	/// value).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:    Option<String>,
	/// Muted mono text at the option's right.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail:   Option<String>,
	/// A leading icon (`provider:<id>` for a provider's logo).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:     Option<Icon>,
	/// A section header over each run of options of the same group.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub group:    Option<String>,
	/// Drawn faint and never picked.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub disabled: Option<bool>,
}

impl SelectOption {
	/// An option `value` labeled `label`.
	pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
		Self { value: value.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Sets the detail.
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the group.
	pub fn group(mut self, group: impl Into<String>) -> Self {
		self.group = Some(group.into());
		self
	}

	/// Disables it.
	pub const fn disabled(mut self, disabled: bool) -> Self {
		self.disabled = Some(disabled);
		self
	}
}

impl From<&str> for SelectOption {
	fn from(value: &str) -> Self {
		Self { value: value.to_owned(), ..Self::default() }
	}
}

impl From<String> for SelectOption {
	fn from(value: String) -> Self {
		Self { value, ..Self::default() }
	}
}

/// One avatar of an `avatars` stack.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct AvatarItem {
	/// The name: its first two characters are the initials.
	pub text:    String,
	/// The shape.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub variant: Option<AvatarVariant>,
	/// Picks the colors (default `text`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub seed:    Option<String>,
	/// An icon drawn instead of the initials.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:    Option<Icon>,
	/// A picture's blob id.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub blob:    Option<String>,
	/// A status tone (not drawn in a stack).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status:  Option<Tone>,
	/// The tooltip and accessible name (default `text`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:   Option<String>,
}

impl AvatarItem {
	/// An avatar of `text`.
	pub fn new(text: impl Into<String>) -> Self {
		Self { text: text.into(), ..Self::default() }
	}

	/// Sets the shape.
	pub fn variant(mut self, variant: impl Into<AvatarVariant>) -> Self {
		self.variant = Some(variant.into());
		self
	}

	/// Sets the color seed.
	pub fn seed(mut self, seed: impl Into<String>) -> Self {
		self.seed = Some(seed.into());
		self
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the picture's blob.
	pub fn blob(mut self, blob: impl Into<String>) -> Self {
		self.blob = Some(blob.into());
		self
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}
}

/// A crumb of a `breadcrumb`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Crumb {
	/// Identity: the `select` event's `item`.
	pub id:    String,
	/// The text (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// An icon before the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:  Option<Icon>,
	/// A URL or path ⌘-click opens.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub href:  Option<String>,
}

impl Crumb {
	/// A crumb labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the link.
	pub fn href(mut self, href: impl Into<String>) -> Self {
		self.href = Some(href.into());
		self
	}
}

/// A day of a `calendar` carrying a dot.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct DayMark {
	/// The day, `YYYY-MM-DD`.
	pub date: String,
	/// The dot's tone (default the accent).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone: Option<Tone>,
}

impl DayMark {
	/// A dot on `date` in `tone`.
	pub fn new(date: impl Into<String>, tone: impl Into<Tone>) -> Self {
		Self { date: date.into(), tone: Some(tone.into()) }
	}
}

impl From<&str> for DayMark {
	fn from(date: &str) -> Self {
		Self { date: date.to_owned(), tone: None }
	}
}

impl From<String> for DayMark {
	fn from(date: String) -> Self {
		Self { date, tone: None }
	}
}

/// A preset of a `week` calendar.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Preset {
	/// The `select` event's `item` and the `preset` prop's value.
	pub id:    String,
	/// The text (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
}

impl Preset {
	/// A preset labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()) }
	}
}

/// An item of a `nav`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NavItem {
	/// The `select` event's `item`, and `active`'s value.
	pub id:    String,
	/// The text (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// An icon before the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:  Option<Icon>,
	/// A small mono pill after the label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub count: Option<String>,
	/// Sub-items: the item opens a menu or panel instead of being picked.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub items: Vec<NavSubItem>,
}

impl NavItem {
	/// An item labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the count pill.
	pub fn count(mut self, count: impl Into<String>) -> Self {
		self.count = Some(count.into());
		self
	}

	/// Adds a sub-item.
	pub fn item(mut self, item: NavSubItem) -> Self {
		self.items.push(item);
		self
	}
}

/// A sub-item of a `nav` item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct NavSubItem {
	/// The `select` event's `item`.
	pub id:     String,
	/// The text (default the `id`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:  Option<String>,
	/// A muted note (`pills`) or description line (`links`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail: Option<String>,
	/// Leads the menu row, or fills the panel entry's tile.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:   Option<Icon>,
}

impl NavSubItem {
	/// A sub-item labeled `label`.
	pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
		Self { id: id.into(), label: Some(label.into()), ..Self::default() }
	}

	/// Sets the detail.
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}
}

/// A reaction chip of a `message`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Reaction {
	/// An emoji or glyph: the reaction's identity.
	pub text:  String,
	/// How many reacted, with the local user (hidden at 0).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub count: Option<u32>,
	/// The local user reacted.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub on:    Option<bool>,
}

impl Reaction {
	/// `text` reacted `count` times.
	pub fn new(text: impl Into<String>, count: u32) -> Self {
		Self { text: text.into(), count: Some(count), on: None }
	}

	/// Marks the local user's own reaction.
	pub const fn on(mut self, on: bool) -> Self {
		self.on = Some(on);
		self
	}
}

/// A source chip of a `trace` or one of its lines.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Source {
	/// The tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title: Option<String>,
	/// Opened by a click.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub href:  Option<String>,
	/// The name shown (default the `href`'s host, `www.` dropped).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub site:  Option<String>,
}

impl Source {
	/// A source at `href`.
	pub fn new(href: impl Into<String>) -> Self {
		Self { href: Some(href.into()), ..Self::default() }
	}

	/// Sets the tooltip.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Sets the site name.
	pub fn site(mut self, site: impl Into<String>) -> Self {
		self.site = Some(site.into());
		self
	}
}

/// A step of a `trace`. A child keyed by its `id` is its content.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct TraceLine {
	/// Identity (default its index): keys its content child and its open
	/// state, and is the `toggle` event's `item`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id:      Option<String>,
	/// Its glyph, tinted by status.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:    Option<Icon>,
	/// The step in words.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:   Option<Text>,
	/// What it acted on, in mono.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub target:  Option<Text>,
	/// A dim line under the row.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail:  Option<Text>,
	/// `pending`, `running`, `done` or `error`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub status:  Option<Status>,
	/// Elapsed ms when sent.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub age:     Option<u64>,
	/// Its time in ms once settled.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub took:    Option<u64>,
	/// Whether its content shows.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub open:    Option<bool>,
	/// Chips under the line.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub sources: Vec<Source>,
}

impl TraceLine {
	/// A step `id` reading `title`.
	pub fn new(id: impl Into<String>, title: impl Into<Text>) -> Self {
		Self { id: Some(id.into()), title: Some(title.into()), ..Self::default() }
	}

	/// Sets the glyph.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets what it acted on.
	pub fn target(mut self, target: impl Into<Text>) -> Self {
		self.target = Some(target.into());
		self
	}

	/// Sets the dim line under it.
	pub fn detail(mut self, detail: impl Into<Text>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Sets the status.
	pub fn status(mut self, status: impl Into<Status>) -> Self {
		self.status = Some(status.into());
		self
	}

	/// Sets the elapsed ms.
	pub const fn age(mut self, age: u64) -> Self {
		self.age = Some(age);
		self
	}

	/// Sets the final time in ms.
	pub const fn took(mut self, took: u64) -> Self {
		self.took = Some(took);
		self
	}

	/// Sets whether its content shows.
	pub const fn open(mut self, open: bool) -> Self {
		self.open = Some(open);
		self
	}

	/// Adds a source chip.
	pub fn source(mut self, source: Source) -> Self {
		self.sources.push(source);
		self
	}
}

/// The error of a `stacktrace`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct StackError {
	/// The error's type (`type` on the wire).
	#[serde(rename = "type", skip_serializing_if = "Option::is_none")]
	pub kind:    Option<String>,
	/// The message.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub message: Option<String>,
}

impl StackError {
	/// An error of type `kind` saying `message`.
	pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
		Self { kind: Some(kind.into()), message: Some(message.into()) }
	}
}

/// A frame of a `stacktrace`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Frame {
	/// The function (`fn` on the wire; `<anonymous>` when empty).
	#[serde(rename = "fn")]
	pub func: String,
	/// The file path; a relative one opens against the pane's directory.
	pub file: String,
	/// The 1-based line.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub line: Option<u32>,
	/// The 1-based column.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub col:  Option<u32>,
	/// Your code (default) or a library's (`false`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub app:  Option<bool>,
	/// A source excerpt for `split`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub code: Option<FrameCode>,
}

impl Frame {
	/// A frame of `func` in `file`.
	pub fn new(func: impl Into<String>, file: impl Into<String>) -> Self {
		Self { func: func.into(), file: file.into(), ..Self::default() }
	}

	/// Sets the line and column.
	pub const fn at(mut self, line: u32, col: u32) -> Self {
		self.line = Some(line);
		self.col = Some(col);
		self
	}

	/// Sets the line.
	pub const fn line(mut self, line: u32) -> Self {
		self.line = Some(line);
		self
	}

	/// Marks it as your code or a library's.
	pub const fn app(mut self, app: bool) -> Self {
		self.app = Some(app);
		self
	}

	/// Sets the source excerpt.
	pub fn code(mut self, code: FrameCode) -> Self {
		self.code = Some(code);
		self
	}
}

/// A source excerpt of a `stacktrace` frame.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct FrameCode {
	/// The first line's number (default 1).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub start: Option<u32>,
	/// The lines.
	pub lines: Vec<String>,
	/// The line to mark (default the frame's `line`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub line:  Option<u32>,
}

impl FrameCode {
	/// `lines` from line `start`.
	pub fn new<L: Into<String>>(start: u32, lines: impl IntoIterator<Item = L>) -> Self {
		Self { start: Some(start), lines: lines.into_iter().map(Into::into).collect(), line: None }
	}

	/// Sets the line to mark.
	pub const fn line(mut self, line: u32) -> Self {
		self.line = Some(line);
		self
	}
}

/// A prompt of a `suggestions` node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Suggestion {
	/// What `activate` names (default the `text`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id:     Option<String>,
	/// The prompt as shown.
	pub text:   String,
	/// `rows`: muted after the text; `cards`: the line under it; `chips`:
	/// the tooltip.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail: Option<String>,
	/// The leading icon.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub icon:   Option<Icon>,
	/// `rows`: the keycaps at the right (default the item's number; empty
	/// for none).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub keys:   Option<Vec<String>>,
}

impl Suggestion {
	/// A prompt reading `text`.
	pub fn new(text: impl Into<String>) -> Self {
		Self { text: text.into(), ..Self::default() }
	}

	/// Sets the id `activate` names.
	pub fn id(mut self, id: impl Into<String>) -> Self {
		self.id = Some(id.into());
		self
	}

	/// Sets the detail.
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}

	/// Sets the icon.
	pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
		self.icon = Some(icon.into());
		self
	}

	/// Sets the keycaps (`rows`).
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self.keys = Some(keys.into_iter().map(Into::into).collect());
		self
	}
}

/// A question of a `questions` node.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Question {
	/// Its events' `item` and its key in `answers`.
	pub id:          String,
	/// The question.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub prompt:      Option<String>,
	/// A line under the prompt.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub description: Option<String>,
	/// The choices.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub choices:     Vec<QuestionChoice>,
	/// Several picks: the answer is a list of values.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub multi:       Option<bool>,
	/// Takes a free-text answer from the `input` child keyed `<id>.other`.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub other:       Option<OtherAnswer>,
	/// No Skip; Next and Submit wait for its answer.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub required:    Option<bool>,
}

impl Question {
	/// Question `id` asking `prompt`.
	pub fn new(id: impl Into<String>, prompt: impl Into<String>) -> Self {
		Self { id: id.into(), prompt: Some(prompt.into()), ..Self::default() }
	}

	/// Sets the line under the prompt.
	pub fn description(mut self, description: impl Into<String>) -> Self {
		self.description = Some(description.into());
		self
	}

	/// Adds a choice.
	pub fn choice(mut self, choice: impl Into<QuestionChoice>) -> Self {
		self.choices.push(choice.into());
		self
	}

	/// Takes several picks.
	pub const fn multi(mut self, multi: bool) -> Self {
		self.multi = Some(multi);
		self
	}

	/// Takes a free-text answer (`true`, or its placeholder).
	pub fn other(mut self, other: impl Into<OtherAnswer>) -> Self {
		self.other = Some(other.into());
		self
	}

	/// Requires an answer.
	pub const fn required(mut self, required: bool) -> Self {
		self.required = Some(required);
		self
	}
}

/// A choice of a question.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct QuestionChoice {
	/// The answer it gives.
	pub value:  String,
	/// The text (default the `value`).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label:  Option<String>,
	/// A second line on a choice card (`steps` only).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub detail: Option<String>,
}

impl QuestionChoice {
	/// A choice `value` labeled `label`.
	pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
		Self { value: value.into(), label: Some(label.into()), detail: None }
	}

	/// Sets the second line.
	pub fn detail(mut self, detail: impl Into<String>) -> Self {
		self.detail = Some(detail.into());
		self
	}
}

impl From<&str> for QuestionChoice {
	fn from(value: &str) -> Self {
		Self { value: value.to_owned(), ..Self::default() }
	}
}

impl From<String> for QuestionChoice {
	fn from(value: String) -> Self {
		Self { value, ..Self::default() }
	}
}

/// A question's free-text answer: on, or on with a placeholder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OtherAnswer {
	/// On or off.
	Flag(bool),
	/// On, with this placeholder.
	Placeholder(String),
}

impl Serialize for OtherAnswer {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Flag(on) => s.serialize_bool(*on),
			Self::Placeholder(placeholder) => {
				let mut map = s.serialize_map(Some(1))?;
				map.serialize_entry("placeholder", placeholder)?;
				map.end()
			},
		}
	}
}

impl From<bool> for OtherAnswer {
	fn from(on: bool) -> Self {
		Self::Flag(on)
	}
}

impl From<&str> for OtherAnswer {
	fn from(placeholder: &str) -> Self {
		Self::Placeholder(placeholder.to_owned())
	}
}

impl From<String> for OtherAnswer {
	fn from(placeholder: String) -> Self {
		Self::Placeholder(placeholder)
	}
}

/// An answer of a `questions` node: a choice's value or text, or the values
/// of a `multi` question.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QuestionAnswer {
	/// One value, or text.
	One(String),
	/// Several values.
	Many(Vec<String>),
}

impl Serialize for QuestionAnswer {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::One(value) => s.serialize_str(value),
			Self::Many(values) => values.serialize(s),
		}
	}
}

impl From<&str> for QuestionAnswer {
	fn from(value: &str) -> Self {
		Self::One(value.to_owned())
	}
}

impl From<String> for QuestionAnswer {
	fn from(value: String) -> Self {
		Self::One(value)
	}
}

impl From<Vec<String>> for QuestionAnswer {
	fn from(values: Vec<String>) -> Self {
		Self::Many(values)
	}
}
