//! Layer 3: nodes. Typed builders for every kind (except `block`, which is
//! Tern's own), every documented prop, the common props, styled spans and
//! handlers that map events to the program's messages.
//!
//! Every builder converts into [`Node`]; containers take children with
//! `.child(…)` / `.children(…)`. Any prop can also be set untyped with
//! `.prop(name, value)`, and any kind built with [`node`].
//!
//! # Example
//! ```
//! use tern_sdk::ui::{self, CardStatus, View};
//!
//! #[derive(Clone)]
//! enum Msg {
//! 	Rerun,
//! }
//!
//! let view: View<Msg> = View::new().main([ui::card()
//! 	.head("Deploy")
//! 	.status(CardStatus::Running)
//! 	.on_menu("rerun", Msg::Rerun)
//! 	.child(ui::md("Uploading **api-gateway**"))
//! 	.child(ui::progress().value(0.4).label("40%"))]);
//! assert!(view.regions()[0].is_some());
//! ```

mod data;
pub mod html;
mod icon;
pub(crate) mod node;
mod types;

use std::time::Instant;

pub use data::*;
pub use icon::Icon;
pub use node::{Node, View};
pub use types::*;

use self::node::{
	actions_methods, base_methods, children_methods, event_methods, kind_type, max_method, props,
	title_method,
};

/// The status chip of a card (the same values as tools).
pub type CardStatus = Status;

/// Builds `Vec<Span>` from strings and spans.
///
/// # Example
/// ```
/// use tern_sdk::{spans, ui};
///
/// let label = spans!["Waiting for ", ui::span("CI").style("strong")];
/// assert_eq!(label.len(), 2);
/// ```
#[macro_export]
macro_rules! spans {
	($($s:expr),* $(,)?) => {
		vec![$($crate::ui::Span::from($s)),*]
	};
}

/// Builds `Vec<Node<M>>` from builders of different kinds.
///
/// # Example
/// ```
/// use tern_sdk::{nodes, ui, ui::View};
///
/// let view: View = View::new().main(nodes![ui::md("# Hi"), ui::rule(), ui::badge("new")]);
/// assert_eq!(view.regions()[0].unwrap().child_nodes().len(), 3);
/// ```
#[macro_export]
macro_rules! nodes {
	($($n:expr),* $(,)?) => {
		vec![$($crate::ui::Node::from($n)),*]
	};
}

/// A styled span of text.
pub fn span(t: impl Into<String>) -> Span {
	Span::new(t)
}

/// A node of any kind (the untyped escape hatch).
pub fn node<M>(kind: impl Into<String>) -> Node<M> {
	Node::new(kind)
}

/// Defines a kind's builder with the common props.
macro_rules! common_kind {
	($(#[$m:meta])* $name:ident) => {
		kind_type!($(#[$m])* $name);

		impl<M> $name<M> {
			base_methods!();
			title_method!();
			max_method!();
			actions_methods!();
			event_methods!();
		}
	};
}

/// Defines a container kind's builder with the common props.
macro_rules! container_kind {
	($(#[$m:meta])* $name:ident) => {
		common_kind!($(#[$m])* $name);

		impl<M> $name<M> {
			children_methods!();
		}
	};
}

/// A new builder of `kind`.
fn new<M, T: From<Node<M>>>(kind: &str) -> T {
	T::from(Node::new(kind))
}

/// Implements `From<Node>` for builder types (used by [`new`]).
macro_rules! from_node {
	($($name:ident),*) => {
		$(impl<M> From<Node<M>> for $name<M> {
			fn from(node: Node<M>) -> Self {
				Self(node)
			}
		})*
	};
}

from_node!(
	Col, Row, Card, Section, Rule, Spacer, TextNode, Md, Code, Diff, Ansi, Rows, Math, Kv, Table,
	Tree, Badge, Kbd, IconNode, Image, List, Item, Tabs, Picker, Spinner, Shimmer, Elapsed, Rate,
	Progress, Meter, Chart, Effort, Editor, Input, StatusBar, Seg, Toast, Overlay, Tool, Agent,
	Checklist, Prefs
);

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

container_kind!(
	/// `col`: a vertical stack.
	Col
);

impl<M> Col<M> {
	props! {
		/// Space between children.
		gap(Gap) = "gap";
		/// Cross-axis alignment.
		align(Align) = "align";
		/// Main-axis distribution.
		justify(Justify) = "justify";
		/// Adds class `wrap`.
		wrap(bool) = "wrap";
	}
}

/// A vertical stack.
pub fn col<M>() -> Col<M> {
	new("col")
}

container_kind!(
	/// `row`: a horizontal flex row.
	Row
);

impl<M> Row<M> {
	props! {
		/// Space between children.
		gap(Gap) = "gap";
		/// Cross-axis alignment.
		align(Align) = "align";
		/// Main-axis distribution.
		justify(Justify) = "justify";
		/// Children flow onto more lines when they don't fit.
		wrap(bool) = "wrap";
	}
}

/// A horizontal flex row.
pub fn row<M>() -> Row<M> {
	new("row")
}

container_kind!(
	/// `card`: a rounded card with a head and a body, optionally folding.
	Card
);

impl<M> Card<M> {
	props! {
		/// Head content: text, spans, or a child's id.
		head(Text) = "head";
		/// The status chip.
		status(Status) = "status";
		/// The head flips the fold.
		collapsible(bool) = "collapsible";
		/// The starting fold state.
		collapsed(bool) = "collapsed";
		/// Lines shown while collapsed.
		preview(Preview) = "preview";
		/// An accent ring and halo.
		selected(bool) = "selected";
		/// The body has no padding.
		inset(bool) = "inset";
		/// The bare variant.
		variant(CardVariant) = "variant";
	}
}

/// A card.
pub fn card<M>() -> Card<M> {
	new("card")
}

container_kind!(
	/// `section`: a lighter disclosure group.
	Section
);

impl<M> Section<M> {
	props! {
		/// Head content.
		head(Text) = "head";
		/// The head flips the fold.
		collapsible(bool) = "collapsible";
		/// The starting fold state.
		collapsed(bool) = "collapsed";
	}
}

/// A section.
pub fn section<M>() -> Section<M> {
	new("section")
}

common_kind!(
	/// `rule`: a divider hairline.
	Rule
);

impl<M> Rule<M> {
	props! {
		/// A centered label.
		label(Text) = "label";
	}
}

/// A divider.
pub fn rule<M>() -> Rule<M> {
	new("rule")
}

common_kind!(
	/// `spacer`: vertical space.
	Spacer
);

impl<M> Spacer<M> {
	props! {
		/// The height.
		size(Gap) = "size";
	}
}

/// Vertical space.
pub fn spacer<M>() -> Spacer<M> {
	new("spacer")
}

// ---------------------------------------------------------------------------
// Text and code
// ---------------------------------------------------------------------------

common_kind!(
	/// `text`: styled spans, wrapped or truncated.
	TextNode
);

impl<M> TextNode<M> {
	props! {
		/// How it wraps.
		wrap(Wrap) = "wrap";
		/// Where it is cut.
		truncate(Truncate) = "truncate";
		/// Clamp to this many lines.
		lines(u32) = "lines";
		/// Caps the line length.
		measure(Measure) = "measure";
	}

	/// Replaces the text (plain or spans).
	pub fn text(mut self, text: impl Into<Text>) -> Self {
		self.0.set_text(text.into());
		self
	}
}

/// Text, plain or styled.
pub fn text<M>(text: impl Into<Text>) -> TextNode<M> {
	let mut node = Node::new("text");
	node.set_text(text.into());
	TextNode(node)
}

common_kind!(
	/// `md`: Markdown.
	Md
);

impl<M> Md<M> {
	props! {
		/// The Markdown source.
		text(String) = "text";
		/// The source is still arriving.
		stream(bool) = "stream";
		/// Spans drawn in place of their literal text.
		marks(Vec<Span>) = "marks";
	}
}

/// Markdown.
pub fn md<M>(source: impl Into<String>) -> Md<M> {
	new::<M, Md<M>>("md").text(source)
}

common_kind!(
	/// `code`: highlighted code.
	Code
);

impl<M> Code<M> {
	props! {
		/// The code.
		text(String) = "text";
		/// The grammar by language name or extension.
		lang(String) = "lang";
		/// A file path: picks the grammar and shows a header.
		path(String) = "path";
		/// Show line numbers.
		numbers(bool) = "numbers";
		/// The first line's number.
		start(i64) = "start";
		/// Marked lines.
		marks(Vec<CodeMark>) = "marks";
		/// Soft-wrap long lines.
		wrap(bool) = "wrap";
	}
}

/// Highlighted code.
pub fn code<M>(source: impl Into<String>) -> Code<M> {
	new::<M, Code<M>>("code").text(source)
}

common_kind!(
	/// `diff`: a unified or split diff.
	Diff
);

impl<M> Diff<M> {
	props! {
		/// A unified diff.
		text(String) = "text";
		/// Hunks given directly.
		hunks(Vec<Hunk>) = "hunks";
		/// Picks the grammar.
		path(String) = "path";
		/// The grammar by language name.
		lang(String) = "lang";
		/// How it lays out.
		mode(DiffMode) = "mode";
	}
}

/// A diff.
pub fn diff<M>() -> Diff<M> {
	new("diff")
}

common_kind!(
	/// `ansi`: output with escape sequences.
	Ansi
);

impl<M> Ansi<M> {
	props! {
		/// The output.
		text(String) = "text";
		/// Wrap at this many columns.
		cols(u32) = "cols";
		/// Keep the tail in view.
		follow(bool) = "follow";
	}

	/// Clamps to `lines` lines under a fade with a "more lines" button.
	pub fn preview(mut self, lines: u32) -> Self {
		self
			.0
			.set("preview", &serde_json::json!({ "lines": lines }));
		self
	}
}

/// Terminal output.
pub fn ansi<M>(output: impl Into<String>) -> Ansi<M> {
	new::<M, Ansi<M>>("ansi").text(output)
}

common_kind!(
	/// `rows`: pre-rendered ANSI rows (the migration fallback).
	Rows
);

impl<M> Rows<M> {
	props! {
		/// The rows.
		lines(Vec<String>) = "lines";
		/// The exact width in columns.
		cols(u32) = "cols";
	}
}

/// Pre-rendered rows.
pub fn rows<M>(lines: impl Into<Vec<String>>) -> Rows<M> {
	new::<M, Rows<M>>("rows").lines(lines)
}

common_kind!(
	/// `math`: TeX math.
	Math
);

impl<M> Math<M> {
	props! {
		/// The TeX, without delimiters.
		text(String) = "text";
		/// Display style.
		display(bool) = "display";
	}
}

/// TeX math.
pub fn math<M>(tex: impl Into<String>) -> Math<M> {
	new::<M, Math<M>>("math").text(tex)
}

// ---------------------------------------------------------------------------
// Data
// ---------------------------------------------------------------------------

common_kind!(
	/// `kv`: an aligned key/value list.
	Kv
);

impl<M> Kv<M> {
	props! {
		/// All pairs.
		items(Vec<KvItem>) = "items";
		/// How it lays out.
		layout(KvLayout) = "layout";
	}

	/// Adds a pair.
	pub fn item(mut self, k: impl Into<Text>, v: impl Into<Text>) -> Self {
		push(&mut self.0, "items", &KvItem { k: k.into(), v: v.into() });
		self
	}
}

/// A key/value list.
pub fn kv<M>() -> Kv<M> {
	new("kv")
}

common_kind!(
	/// `table`: a table with priority-hidden columns and meter cells.
	Table
);

impl<M> Table<M> {
	props! {
		/// All columns.
		cols(Vec<Column>) = "cols";
		/// All rows.
		rows(Vec<TableRow>) = "rows";
	}

	/// Adds a column.
	pub fn col(mut self, col: Column) -> Self {
		push(&mut self.0, "cols", &col);
		self
	}

	/// Adds a row.
	pub fn row(mut self, row: TableRow) -> Self {
		push(&mut self.0, "rows", &row);
		self
	}
}

/// A table.
pub fn table<M>() -> Table<M> {
	new("table")
}

common_kind!(
	/// `tree`: a disclosure tree.
	Tree
);

impl<M> Tree<M> {
	props! {
		/// The top-level items.
		nodes(Vec<TreeItem>) = "nodes";
	}

	/// Adds a top-level item.
	pub fn item(mut self, item: TreeItem) -> Self {
		push(&mut self.0, "nodes", &item);
		self
	}
}

/// A disclosure tree.
pub fn tree<M>() -> Tree<M> {
	new("tree")
}

common_kind!(
	/// `badge`: a pill chip.
	Badge
);

impl<M> Badge<M> {
	props! {
		/// The label (plain text).
		text(String) = "text";
	}
}

/// A pill chip.
pub fn badge<M>(text: impl Into<String>) -> Badge<M> {
	new::<M, Badge<M>>("badge").text(text)
}

common_kind!(
	/// `kbd`: keycaps.
	Kbd
);

impl<M> Kbd<M> {
	props! {
		/// Key names, in order.
		keys(Vec<String>) = "keys";
	}
}

/// Keycaps (`["cmd", "K"]`).
pub fn kbd<M, K: Into<String>>(keys: impl IntoIterator<Item = K>) -> Kbd<M> {
	new::<M, Kbd<M>>("kbd").keys(keys.into_iter().map(Into::into).collect::<Vec<_>>())
}

common_kind!(
	/// `icon`: a named icon.
	IconNode
);

impl<M> IconNode<M> {
	props! {
		/// The icon.
		name(Icon) = "name";
	}
}

/// A named icon.
pub fn icon<M>(name: impl Into<Icon>) -> IconNode<M> {
	new::<M, IconNode<M>>("icon").name(name)
}

common_kind!(
	/// `image`: an image from a blob.
	Image
);

impl<M> Image<M> {
	props! {
		/// A blob id.
		blob(String) = "blob";
		/// An image Tern ships (`"omp"`).
		builtin(String) = "builtin";
		/// The accessible name.
		alt(String) = "alt";
		/// Natural width in px.
		w(f64) = "w";
		/// Natural height in px.
		h(f64) = "h";
		/// A file the zoom viewer may open.
		path(String) = "path";
	}
}

/// An image.
pub fn image<M>() -> Image<M> {
	new("image")
}

// ---------------------------------------------------------------------------
// Lists, tabs and pickers
// ---------------------------------------------------------------------------

kind_type!(
	/// `list`: a selectable list of `item`s.
	List
);

impl<M> List<M> {
	base_methods!();

	title_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The selected item's id.
		selected(String) = "selected";
		/// A query marked in each item's label.
		filter(String) = "filter";
		/// The line shown when empty.
		empty(Text) = "empty";
		/// Virtualize the rows.
		virtualize(bool) = "virtual";
	}

	/// Caps the height at `lines` rows (the list scrolls itself).
	pub fn max_lines(mut self, lines: u32) -> Self {
		self.0.set("max", &serde_json::json!({ "lines": lines }));
		self
	}

	/// Caps the height at a fraction of the window's.
	pub fn max_fraction(mut self, fraction: f64) -> Self {
		self.0.set("max", &fraction);
		self
	}
}

/// A selectable list.
pub fn list<M>() -> List<M> {
	new("list")
}

common_kind!(
	/// `item`: one list row.
	Item
);

impl<M> Item<M> {
	props! {
		/// The row's text.
		label(Text) = "label";
		/// Dim text after the label.
		detail(Text) = "detail";
		/// Right-aligned text.
		value(Text) = "value";
		/// An icon.
		icon(Icon) = "icon";
		/// Key names drawn as keycaps.
		hint(Vec<String>) = "hint";
		/// Dims the row; it sends nothing.
		disabled(bool) = "disabled";
	}
}

/// A list row.
pub fn item<M>(label: impl Into<Text>) -> Item<M> {
	new::<M, Item<M>>("item").label(label)
}

common_kind!(
	/// `tabs`: a tab strip.
	Tabs
);

impl<M> Tabs<M> {
	props! {
		/// All tabs.
		items(Vec<Tab>) = "items";
		/// The active tab's id.
		active(String) = "active";
	}

	/// Adds a tab.
	pub fn tab(mut self, id: impl Into<String>, label: impl Into<Text>) -> Self {
		push(&mut self.0, "items", &Tab { id: id.into(), label: label.into() });
		self
	}
}

/// A tab strip.
pub fn tabs<M>() -> Tabs<M> {
	new("tabs")
}

kind_type!(
	/// `picker`: a searchable picker sheet (its child is the preview).
	Picker
);

impl<M> Picker<M> {
	base_methods!();

	max_method!();

	event_methods!();

	children_methods!();

	props! {
		/// The sheet's size.
		size(PickerSize) = "size";
		/// The row template.
		layout(PickerLayout) = "layout";
		/// Where the preview sits.
		preview(PickerPreview) = "preview";
		/// The heading.
		title(Text) = "title";
		/// The heading's second part.
		subtitle(Text) = "subtitle";
		/// The head icon.
		icon(Icon) = "icon";
		/// The search text (unset hides the search).
		query(String) = "query";
		/// The caret in `query`, UTF-16.
		cursor(usize) = "cursor";
		/// The search placeholder.
		placeholder(String) = "placeholder";
		/// What the rows are.
		noun(String) = "noun";
		/// Which part the keys drive.
		focus(PickerFocus) = "focus";
		/// Ready, loading or error.
		state(PickerState) = "state";
		/// The error card's text.
		message(Text) = "message";
		/// The text when there are no rows.
		empty(Text) = "empty";
		/// The catalog's size.
		total(u64) = "total";
		/// The catalog.
		items(Vec<PickerItem>) = "items";
		/// Items added to the catalog.
		items_add(Vec<PickerItem>) = "itemsAdd";
		/// Item ids removed from the catalog.
		items_del(Vec<String>) = "itemsDel";
		/// What to show, in order.
		order(Vec<OrderEntry>) = "order";
		/// The selected item.
		selected(String) = "selected";
		/// Items marked current.
		current(Vec<String>) = "current";
		/// Match ranges per item id.
		hits(std::collections::BTreeMap<String, Vec<[usize; 2]>>) = "hits";
		/// Fact columns.
		columns(Vec<PickerColumn>) = "columns";
		/// A confirm strip over the selected row.
		confirm(Confirm) = "confirm";
		/// A scope column.
		scopes(Vec<Scope>) = "scopes";
		/// The active scope.
		scope(String) = "scope";
		/// A tab row.
		tabs(Vec<PickerTab>) = "tabs";
		/// The active tab.
		tab(String) = "tab";
		/// A chip strip.
		strip(Strip) = "strip";
		/// The action bar.
		actions(Vec<PickerAction>) = "actions";
	}
}

/// A picker sheet.
pub fn picker<M>() -> Picker<M> {
	new("picker")
}

// ---------------------------------------------------------------------------
// Progress and motion
// ---------------------------------------------------------------------------

common_kind!(
	/// `spinner`: an activity indicator.
	Spinner
);

impl<M> Spinner<M> {
	props! {
		/// The indicator.
		style(SpinnerStyle) = "style";
		/// Text after it.
		label(Text) = "label";
	}
}

/// An activity indicator.
pub fn spinner<M>() -> Spinner<M> {
	new("spinner")
}

common_kind!(
	/// `shimmer`: a shimmering label.
	Shimmer
);

impl<M> Shimmer<M> {
	props! {
		/// How it sweeps.
		mode(ShimmerMode) = "mode";
		/// The sweep colors.
		palette(ShimmerPalette) = "palette";
	}

	/// Replaces the text (plain or spans).
	pub fn text(mut self, text: impl Into<Text>) -> Self {
		self.0.set_text(text.into());
		self
	}
}

/// A shimmering label.
pub fn shimmer<M>(text: impl Into<Text>) -> Shimmer<M> {
	new::<M, Shimmer<M>>("shimmer").text(text)
}

common_kind!(
	/// `elapsed`: a live timer Tern clocks.
	Elapsed
);

impl<M> Elapsed<M> {
	props! {
		/// Milliseconds already elapsed (negative: a countdown).
		age(i64) = "age";
		/// Freezes the display at this many ms.
		stopped(u64) = "stopped";
		/// How the time reads.
		format(ElapsedFormat) = "format";
	}

	/// Sets `age` from when the timed work started.
	pub fn since(self, start: Instant) -> Self {
		let age = i64::try_from(start.elapsed().as_millis()).unwrap_or(i64::MAX);
		self.age(age)
	}
}

/// A live timer.
pub fn elapsed<M>() -> Elapsed<M> {
	new("elapsed")
}

common_kind!(
	/// `rate`: a number easing between updates.
	Rate
);

impl<M> Rate<M> {
	props! {
		/// The target value.
		value(f64) = "value";
		/// Appended after a space.
		unit(String) = "unit";
	}
}

/// An easing number.
pub fn rate<M>(value: f64) -> Rate<M> {
	new::<M, Rate<M>>("rate").value(value)
}

common_kind!(
	/// `progress`: a progress bar (indeterminate without a value).
	Progress
);

impl<M> Progress<M> {
	props! {
		/// Fraction done, 0–1.
		value(f64) = "value";
		/// Text after the bar.
		label(Text) = "label";
	}
}

/// A progress bar.
pub fn progress<M>() -> Progress<M> {
	new("progress")
}

common_kind!(
	/// `meter`: a value as a bar, ring or grid.
	Meter
);

impl<M> Meter<M> {
	props! {
		/// The level, 0–1.
		value(f64) = "value";
		/// Stacked parts.
		parts(Vec<Part>) = "parts";
		/// Tint levels.
		thresholds(Thresholds) = "thresholds";
		/// The shape.
		style(MeterStyle) = "style";
		/// The size.
		size(Size) = "size";
		/// `blocks`: one row of this many cells.
		steps(u32) = "steps";
		/// Ticks.
		marks(Vec<MeterMark>) = "marks";
		/// Text after the shape.
		label(Text) = "label";
		/// Text after the label.
		total(Text) = "total";
	}
}

/// A meter.
pub fn meter<M>() -> Meter<M> {
	new("meter")
}

common_kind!(
	/// `chart`: bars, a sparkline or a heatmap.
	Chart
);

impl<M> Chart<M> {
	props! {
		/// Which chart.
		kind(ChartKind) = "kind";
		/// The size.
		size(Size) = "size";
		/// The fill color token.
		token(String) = "token";
		/// A line under the chart.
		summary(Text) = "summary";
		/// `bars`/`spark`: one bar per entry.
		series(Vec<Point>) = "series";
		/// `heatmap`: rows of intensities 0–1.
		cells(Vec<Vec<Option<f64>>>) = "cells";
		/// `heatmap`: a tooltip per cell.
		tips(Vec<Vec<Option<String>>>) = "tips";
		/// `heatmap`: a label per row.
		rows(Vec<String>) = "rows";
		/// `heatmap`: column labels.
		cols(Vec<ChartCol>) = "cols";
	}
}

/// A chart.
pub fn chart<M>(kind: impl Into<ChartKind>) -> Chart<M> {
	new::<M, Chart<M>>("chart").kind(kind)
}

common_kind!(
	/// `effort`: a thinking-effort glyph.
	Effort
);

impl<M> Effort<M> {
	props! {
		/// The rung.
		level(EffortLevel) = "level";
	}
}

/// A thinking-effort glyph.
pub fn effort<M>(level: impl Into<EffortLevel>) -> Effort<M> {
	new::<M, Effort<M>>("effort").level(level)
}

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

common_kind!(
	/// `editor`: a multi-line text field whose text the program owns.
	Editor
);

impl<M> Editor<M> {
	props! {
		/// The text.
		text(String) = "text";
		/// The caret, UTF-16.
		cursor(usize) = "cursor";
		/// The selection anchor, UTF-16.
		anchor(usize) = "anchor";
		/// Styled ranges.
		decor(Vec<Decor>) = "decor";
		/// An inline completion after the caret.
		ghost(String) = "ghost";
		/// Dim text while empty.
		placeholder(String) = "placeholder";
		/// Spans before the first line.
		prompt(Text) = "prompt";
		/// A vim mode label.
		mode(String) = "mode";
		/// The text is code in this language.
		lang(String) = "lang";
		/// Not editable.
		readonly(bool) = "readonly";
		/// Accepts an atomic `send`.
		sendable(bool) = "sendable";
		/// Cap the height at this many lines.
		max_lines(u32) = "maxLines";
	}
}

/// A multi-line text field.
pub fn editor<M>() -> Editor<M> {
	new("editor")
}

common_kind!(
	/// `input`: a single-line text field.
	Input
);

impl<M> Input<M> {
	props! {
		/// The text.
		text(String) = "text";
		/// The caret, UTF-16.
		cursor(usize) = "cursor";
		/// The selection anchor, UTF-16.
		anchor(usize) = "anchor";
		/// Styled ranges.
		decor(Vec<Decor>) = "decor";
		/// An inline completion after the caret.
		ghost(String) = "ghost";
		/// Dim text while empty.
		placeholder(String) = "placeholder";
		/// Spans before the text.
		prompt(Text) = "prompt";
		/// A vim mode label.
		mode(String) = "mode";
		/// The text is code in this language.
		lang(String) = "lang";
		/// Not editable.
		readonly(bool) = "readonly";
		/// Accepts an atomic `send`.
		sendable(bool) = "sendable";
	}
}

/// A single-line text field.
pub fn input<M>() -> Input<M> {
	new("input")
}

// ---------------------------------------------------------------------------
// Bars, toasts and overlays
// ---------------------------------------------------------------------------

container_kind!(
	/// `status`: a status bar strip of `seg`s.
	StatusBar
);

impl<M> StatusBar<M> {
	props! {
		/// No fill.
		transparent(bool) = "transparent";
	}
}

/// A status bar strip.
pub fn status<M>() -> StatusBar<M> {
	new("status")
}

container_kind!(
	/// `seg`: a status bar segment.
	Seg
);

impl<M> Seg<M> {
	props! {
		/// An icon.
		icon(Icon) = "icon";
		/// The right group.
		side(Side) = "side";
		/// Drop order: lower drops first.
		priority(f64) = "priority";
	}

	/// Replaces the text (plain or spans).
	pub fn text(mut self, text: impl Into<Text>) -> Self {
		self.0.set_text(text.into());
		self
	}
}

/// A status bar segment.
pub fn seg<M>(text: impl Into<Text>) -> Seg<M> {
	new::<M, Seg<M>>("seg").text(text)
}

common_kind!(
	/// `toast`: a transient notice.
	Toast
);

impl<M> Toast<M> {
	props! {
		/// The message.
		text(String) = "text";
		/// A mono detail.
		#[allow(clippy::should_implement_trait, reason = "the prop is named `sub`")]
		sub(String) = "sub";
		/// Take it down after this many ms.
		ttl(u64) = "ttl";
	}
}

/// A transient notice.
pub fn toast<M>(text: impl Into<String>) -> Toast<M> {
	new::<M, Toast<M>>("toast").text(text)
}

container_kind!(
	/// `overlay`: a floating panel or sheet in `layer`.
	Overlay
);

impl<M> Overlay<M> {
	props! {
		/// Placement.
		anchor(Anchor) = "anchor";
		/// Card width or full sheet.
		size(OverlaySize) = "size";
		/// A dim backdrop taking the pointer.
		modal(bool) = "modal";
		/// The title row.
		head(Text) = "head";
	}
}

/// A floating panel.
pub fn overlay<M>() -> Overlay<M> {
	new("overlay")
}

// ---------------------------------------------------------------------------
// Tool, agent and task kinds
// ---------------------------------------------------------------------------

kind_type!(
	/// `tool`: one coding-agent tool call.
	Tool
);

impl<M> Tool<M> {
	base_methods!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The tool's name (picks the icon).
		name(String) = "name";
		/// The verb.
		title(Text) = "title";
		/// The primary argument.
		target(Text) = "target";
		/// How a string target is drawn.
		target_kind(TargetKind) = "targetKind";
		/// Short facts after the target.
		meta(Vec<Text>) = "meta";
		/// Chips in the head.
		badges(Vec<BadgeSpec>) = "badges";
		/// A short state note.
		note(Text) = "note";
		/// A non-zero exit code.
		exit(i64) = "exit";
		/// The state.
		status(Status) = "status";
		/// Elapsed ms when sent.
		age(i64) = "age";
		/// The final duration in ms.
		took(u64) = "took";
		/// The head's tooltip.
		intent(String) = "intent";
		/// Ring or no ring.
		frame(ToolFrame) = "frame";
		/// The head toggles the body.
		collapsible(bool) = "collapsible";
		/// The initial fold.
		collapsed(bool) = "collapsed";
		/// The clamp of the collapsed body.
		preview(ToolPreview) = "preview";
		/// Hover buttons.
		tools(Vec<ToolButton>) = "tools";
	}
}

/// A tool call.
pub fn tool<M>(name: impl Into<String>) -> Tool<M> {
	new::<M, Tool<M>>("tool").name(name)
}

container_kind!(
	/// `agent`: one subagent row.
	Agent
);

impl<M> Agent<M> {
	props! {
		/// The display name.
		name(String) = "name";
		/// The agent type.
		agent(String) = "agent";
		/// More chips.
		badges(Vec<BadgeSpec>) = "badges";
		/// The one-line task.
		task(Text) = "task";
		/// The state.
		status(AgentStatus) = "status";
		/// The model.
		model(String) = "model";
		/// The model dot's palette token.
		thinking(String) = "thinking";
		/// Stats.
		stats(AgentStats) = "stats";
		/// The tool running now.
		tool(AgentTool) = "tool";
		/// A retry countdown.
		retry(Retry) = "retry";
		/// Nesting depth.
		depth(u32) = "depth";
		/// The row toggles the children.
		collapsible(bool) = "collapsible";
		/// The initial fold.
		collapsed(bool) = "collapsed";
	}
}

/// A subagent row.
pub fn agent<M>(name: impl Into<String>) -> Agent<M> {
	new::<M, Agent<M>>("agent").name(name)
}

common_kind!(
	/// `checklist`: a todo list.
	Checklist
);

impl<M> Checklist<M> {
	props! {
		/// The phases.
		phases(Vec<Phase>) = "phases";
		/// The presentation.
		mode(ChecklistMode) = "mode";
		/// `reminder`: text after the count.
		note(Text) = "note";
	}
}

/// A todo list.
pub fn checklist<M>() -> Checklist<M> {
	new("checklist")
}

kind_type!(
	/// `prefs`: a settings page or sheet.
	Prefs
);

impl<M> Prefs<M> {
	base_methods!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The brand title.
		title(String) = "title";
		/// Nav rows.
		pages(Vec<PrefsPage>) = "pages";
		/// The current page.
		page(String) = "page";
		/// A paragraph under the page title.
		lead(String) = "lead";
		/// The sections.
		sections(Vec<PrefsSection>) = "sections";
		/// Search text.
		query(String) = "query";
		/// The caret in the search field, UTF-16.
		cursor(usize) = "cursor";
		/// The row or section with the focus ring.
		focus(String) = "focus";
		/// The row being edited.
		editing(Editing) = "editing";
	}
}

/// A settings page.
pub fn prefs<M>() -> Prefs<M> {
	new("prefs")
}

/// Appends `value` to the list prop `key`.
fn push<M>(node: &mut Node<M>, key: &str, value: &impl serde::Serialize) {
	let Ok(value) = serde_json::to_value(value) else {
		return;
	};
	match node.props.get_mut(key) {
		Some(serde_json::Value::Array(list)) => list.push(value),
		_ => {
			node
				.props
				.insert(key.to_owned(), serde_json::Value::Array(vec![value]));
		},
	}
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;

	fn wire<M>(node: impl Into<Node<M>>) -> serde_json::Value {
		let node = node.into();
		json!({ "k": node.kind, "p": node.props })
	}

	#[test]
	fn text_takes_a_string_or_spans() {
		assert_eq!(wire::<()>(text("hi")), json!({"k": "text", "p": {"text": "hi"}}));
		let styled = text::<()>(spans!["a", span("b").style(Token::Muted).style("strong")]);
		assert_eq!(
			wire(styled.clone()),
			json!({"k": "text", "p": {"spans": [{"t": "a"}, {"t": "b", "s": "muted strong"}]}})
		);
		// Switching back to plain text drops the spans.
		assert_eq!(wire(styled.text("c")), json!({"k": "text", "p": {"text": "c"}}));
	}

	#[test]
	fn typed_values_and_escape_hatches_serialize() {
		let c = card::<()>()
			.head("Deploy")
			.status(Status::Running)
			.tone("brand-new")
			.min(Bound::w(Extent::Ch(10.0)))
			.basis(Basis::Content)
			.key(7)
			.prop("future", json!([1]))
			.prop("head", serde_json::Value::Null);
		assert_eq!(
			wire(c),
			json!({"k": "card", "p": {
				"status": "running", "tone": "brand-new", "min": {"w": "10ch"},
				"basis": "content", "key": 7, "future": [1]
			}})
		);
	}

	#[test]
	fn kind_specific_props_replace_common_meanings() {
		let title = spans![span("Heading").style("strong")];
		assert_eq!(
			wire(tool::<()>("read").title(title.clone()))["p"]["title"],
			json!([{"t": "Heading", "s": "strong"}])
		);
		assert_eq!(
			wire(picker::<()>().title(title).actions(vec![PickerAction {
				id: "open".into(),
				primary: Some(true),
				..PickerAction::default()
			}]))["p"],
			json!({
				"title": [{"t": "Heading", "s": "strong"}],
				"actions": [{"id": "open", "primary": true}]
			})
		);
		assert_eq!(wire(prefs::<()>().title("Settings"))["p"]["title"], "Settings");
		assert_eq!(
			wire(list::<()>().max_fraction(0.5).max_lines(4))["p"]["max"],
			json!({"lines": 4})
		);
		assert_eq!(wire(list::<()>().max_lines(4).max_fraction(0.5))["p"]["max"], 0.5);
	}

	#[test]
	fn agent_statuses_have_their_own_vocabulary() {
		for (status, expected) in [
			(AgentStatus::Pending, "pending"),
			(AgentStatus::Running, "running"),
			(AgentStatus::Done, "done"),
			(AgentStatus::Failed, "failed"),
			(AgentStatus::Aborted, "aborted"),
			(AgentStatus::Idle, "idle"),
			(AgentStatus::Parked, "parked"),
			(AgentStatus::Other("future".into()), "future"),
		] {
			assert_eq!(wire(agent::<()>("worker").status(status))["p"]["status"], expected);
		}
	}

	#[test]
	fn table_meter_cells_keep_the_wire_wrapper() {
		let node = wire(table::<()>().row(TableRow::new("a").cell("usage", MeterCell {
			value: Some(0.5),
			title: Some("Half".into()),
			tone: Some(Tone::Success),
			..MeterCell::default()
		})));
		assert_eq!(
			node["p"]["rows"][0]["cells"]["usage"],
			json!({
				"meter": {"value": 0.5, "title": "Half", "tone": "success"}
			})
		);
		assert_eq!(
			serde_json::to_value(Cell::Meter(MeterCell::default())).unwrap(),
			json!({"meter": {}})
		);
	}

	#[test]
	fn list_props_append() {
		let k = kv::<()>().item("a", "1").item("b", span("2").style("num"));
		assert_eq!(
			wire(k),
			json!({"k": "kv", "p": {"items": [
				{"k": "a", "v": "1"}, {"k": "b", "v": [{"t": "2", "s": "num"}]}
			]}})
		);
	}
}
