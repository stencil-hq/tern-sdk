//! Structured prop values: table columns and rows, tree items, list and
//! picker entries, meter parts, tool and agent facts, checklist phases and
//! `prefs` pages. Every field is public; optional ones are omitted from the
//! wire when `None` or empty.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer, ser::SerializeMap};
use serde_json::Value;

use super::{
	icon::Icon,
	types::{ChoiceStyle, ColumnFormat, ItemStatus, Text, TextAlign, Tone, Truncate},
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

/// A table cell: text or a meter.
#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
	/// Text.
	Text(Text),
	/// A meter.
	Meter(MeterCell),
}

impl Serialize for Cell {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Text(t) => t.serialize(s),
			Self::Meter(m) => {
				let mut map = s.serialize_map(Some(1))?;
				map.serialize_entry("meter", m)?;
				map.end()
			},
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

/// A `table` row.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct TableRow {
	/// Row identity, unique in the table.
	pub id:    String,
	/// One cell per column id.
	pub cells: BTreeMap<String, Cell>,
}

impl TableRow {
	/// An empty row.
	pub fn new(id: impl Into<String>) -> Self {
		Self { id: id.into(), cells: BTreeMap::new() }
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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct CodeMark {
	/// The line, in the gutter's numbering.
	pub line:   u64,
	/// The tone.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub tone:   Option<Tone>,
	/// Character ranges `[from, to)` marked on the line.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub ranges: Vec<[u64; 2]>,
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

/// One bar of a `bars` or `spark` chart.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct Point {
	/// The value.
	pub value: f64,
	/// A label.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub label: Option<String>,
	/// A tooltip.
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
