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
	actions_methods, base_methods, children_methods, event_methods, kind_type, max_method,
	min_method, props, title_method,
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
			min_method!();
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
	Col,
	Row,
	Card,
	Section,
	Rule,
	Spacer,
	TextNode,
	Md,
	Code,
	Diff,
	Ansi,
	Rows,
	Math,
	Kv,
	Table,
	Tree,
	Json,
	Badge,
	Kbd,
	IconNode,
	Image,
	Media,
	List,
	Item,
	Tabs,
	Picker,
	Spinner,
	Shimmer,
	Elapsed,
	Rate,
	Progress,
	Meter,
	Chart,
	Effort,
	Editor,
	Input,
	StatusBar,
	Seg,
	Toast,
	Overlay,
	Tool,
	Agent,
	Checklist,
	Prefs,
	Button,
	Stats,
	Select,
	Switch,
	Slider,
	Field,
	Otp,
	Buttons,
	Avatar,
	Avatars,
	Breadcrumb,
	Pager,
	Empty,
	Skeleton,
	Calendar,
	Date,
	Menu,
	Menubar,
	Nav,
	Split,
	Scroll,
	Carousel,
	Message,
	Attachment,
	Trace,
	Stacktrace,
	Suggestions,
	Questions
);

/// What a slot prop (`card` `head`/`end`, `section` `head`, `overlay`
/// `end`/`foot`) holds: text, or a node drawn there.
pub enum Slot<M = ()> {
	/// Text or spans (only `head` takes text).
	Text(Text),
	/// A node: added as the child keyed by the slot's name, the prop sent as
	/// its id (`<parent>.end`).
	Node(Node<M>),
}

impl<M> From<&str> for Slot<M> {
	fn from(t: &str) -> Self {
		Self::Text(t.into())
	}
}

impl<M> From<String> for Slot<M> {
	fn from(t: String) -> Self {
		Self::Text(t.into())
	}
}

impl<M> From<&String> for Slot<M> {
	fn from(t: &String) -> Self {
		Self::Text(t.into())
	}
}

impl<M, const N: usize> From<[Span; N]> for Slot<M> {
	fn from(s: [Span; N]) -> Self {
		Self::Text(s.into())
	}
}

impl<M> From<Text> for Slot<M> {
	fn from(t: Text) -> Self {
		Self::Text(t)
	}
}

impl<M> From<Span> for Slot<M> {
	fn from(s: Span) -> Self {
		Self::Text(s.into())
	}
}

impl<M> From<Vec<Span>> for Slot<M> {
	fn from(s: Vec<Span>) -> Self {
		Self::Text(s.into())
	}
}

impl<M> From<Node<M>> for Slot<M> {
	fn from(n: Node<M>) -> Self {
		Self::Node(n)
	}
}

/// Builders of nodes fill slots too.
macro_rules! slot_from {
	($($name:ident),*) => {
		$(impl<M> From<$name<M>> for Slot<M> {
			fn from(n: $name<M>) -> Self {
				Self::Node(n.0)
			}
		})*
	};
}

slot_from!(
	Col,
	Row,
	Card,
	Section,
	Rule,
	Spacer,
	TextNode,
	Md,
	Code,
	Diff,
	Ansi,
	Rows,
	Math,
	Kv,
	Table,
	Tree,
	Json,
	Badge,
	Kbd,
	IconNode,
	Image,
	Media,
	List,
	Item,
	Tabs,
	Picker,
	Spinner,
	Shimmer,
	Elapsed,
	Rate,
	Progress,
	Meter,
	Chart,
	Effort,
	Editor,
	Input,
	StatusBar,
	Seg,
	Toast,
	Overlay,
	Tool,
	Agent,
	Checklist,
	Prefs,
	Button,
	Stats,
	Select,
	Switch,
	Slider,
	Field,
	Otp,
	Buttons,
	Avatar,
	Avatars,
	Breadcrumb,
	Pager,
	Empty,
	Skeleton,
	Calendar,
	Date,
	Menu,
	Menubar,
	Nav,
	Split,
	Scroll,
	Carousel,
	Message,
	Attachment,
	Trace,
	Stacktrace,
	Suggestions,
	Questions
);

/// Sets slot prop `slot` of `node` from text or a node.
fn set_slot<M>(node: &mut Node<M>, slot: &'static str, value: Slot<M>) {
	match value {
		Slot::Text(text) => {
			node.clear_slot(slot);
			node.set(slot, &text);
		},
		Slot::Node(child) => node.set_slot(slot, child),
	}
}

/// Card/section/overlay head props shared by the panel kinds.
macro_rules! panel_head_methods {
	() => {
		/// The head icon (wins over the role's).
		pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
			self.0.set("icon", &icon.into());
			self
		}

		/// Muted facts after the head text.
		pub fn meta(mut self, meta: impl Into<Text>) -> Self {
			self.0.set("meta", &meta.into());
			self
		}

		/// A node drawn at the head's end (buttons, segmented tabs, badges);
		/// clicks on it never fold the panel.
		pub fn end(mut self, end: impl Into<Slot<M>>) -> Self {
			set_slot(&mut self.0, "end", end.into());
			self
		}
	};
}

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
	panel_head_methods!();

	props! {
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

	/// Head content: text, spans, or a node (sent as the child `head`).
	pub fn head(mut self, head: impl Into<Slot<M>>) -> Self {
		set_slot(&mut self.0, "head", head.into());
		self
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
		/// The head flips the fold.
		collapsible(bool) = "collapsible";
		/// The starting fold state.
		collapsed(bool) = "collapsed";
	}

	/// Head content: text, spans, or a node (sent as the child `head`).
	pub fn head(mut self, head: impl Into<Slot<M>>) -> Self {
		set_slot(&mut self.0, "head", head.into());
		self
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
		/// The label: centered (`line`), in a framed row (`note`) or a pill
		/// (`pin`).
		label(Text) = "label";
		/// The look.
		variant(RuleVariant) = "variant";
		/// An icon before the label.
		icon(Icon) = "icon";
		/// A time or short stamp.
		time(String) = "time";
	}

	/// A node drawn at the end: after the label (`line`), at the row's
	/// right (`note`) or beside the pill (`pin`).
	pub fn end(mut self, end: impl Into<Slot<M>>) -> Self {
		set_slot(&mut self.0, "end", end.into());
		self
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
		/// How cited spans look.
		cite(CiteStyle) = "cite";
		/// Draws the numbered sources list under the text (default only
		/// for `note` cites).
		sources(bool) = "sources";
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
		/// Each line's number, strictly ascending, in place of counting from
		/// `start`: a `…` row goes where the numbers jump (search hits,
		/// excerpts).
		nums(Vec<u64>) = "nums";
		/// Marked lines.
		marks(Vec<CodeMark>) = "marks";
		/// Soft-wrap long lines.
		wrap(bool) = "wrap";
		/// Runs of lines drawn folded: sorted, not overlapping.
		folds(Vec<CodeFold>) = "folds";
		/// A strip above the lines of their heat and tones, with a bracket
		/// over the lines in view; a click or drag on it scrolls there.
		overview(bool) = "overview";
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
		/// Rows take clicks: `select` on click, `activate` on double click,
		/// with the row id as `item`.
		selectable(bool) = "selectable";
		/// The row drawn selected, kept in view.
		selected(String) = "selected";
		/// Shown muted when there are no rows.
		empty(Text) = "empty";
		/// The column the rows are sorted by (its header draws the arrow).
		/// A header click sends `action` `sort` with `{col, dir}`.
		sort(Sort) = "sort";
		/// Rows are picked several at a time; each change sends `change`
		/// with the picked ids.
		multi(bool) = "multi";
		/// The ids of the picked rows.
		picked(Vec<String>) = "picked";
		/// How picks and the sort look.
		variant(TableVariant) = "variant";
		/// Buttons in the selection bar, sent with the picked ids.
		bar(Vec<ActButton>) = "bar";
		/// Muted text after the count in the selection bar.
		hint(Text) = "hint";
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

	/// Adds a selection bar button.
	pub fn bar_button(mut self, button: ActButton) -> Self {
		push(&mut self.0, "bar", &button);
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
	/// `json`: a JSON value Tern draws as a foldable tree.
	Json
);

impl<M> Json<M> {
	/// How many levels of containers start expanded (default 0).
	pub fn open(mut self, levels: u32) -> Self {
		self.0.set("open", &levels);
		self
	}
}

/// A JSON value Tern draws natively as an interactive tree. Check the hello
/// reply's `kinds` for `json` and fall back to [`tree`] otherwise.
pub fn json<M>(value: impl Into<serde_json::Value>) -> Json<M> {
	let mut node: Json<M> = new("json");
	node.0.set("value", &value.into());
	node
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

common_kind!(
	/// `media`: a file Tern loads by path and shows in its media viewer.
	Media
);

impl<M> Media<M> {
	props! {
		/// A git revision to read the file from (`HEAD`, a sha, `:0` for the index).
		rev(String) = "rev";
		/// `media` nodes sharing this link pan and zoom together.
		link(String) = "link";
	}
}

/// A file at `path` (absolute, or relative to the pane's working directory) in
/// Tern's media viewer.
pub fn media<M>(path: impl Into<String>) -> Media<M> {
	let mut node = new::<M, Media<M>>("media");
	node.0.set("path", &path.into());
	node
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

	min_method!();

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
		/// Underline (default) or a segmented well.
		variant(TabsVariant) = "variant";
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

	min_method!();

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

kind_type!(
	/// `chart`: bars, a sparkline, a heatmap, a line or area chart or a
	/// donut.
	///
	/// Its `min`/`max` are a number value domain, not the common size
	/// bounds.
	Chart
);

impl<M> Chart<M> {
	base_methods!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// Which chart.
		kind(ChartKind) = "kind";
		/// The size.
		size(Size) = "size";
		/// The fill color token (the first series' color).
		token(String) = "token";
		/// A line under the chart.
		summary(Text) = "summary";
		/// `bars`/`spark`: one bar per entry; `line`/`area`: one point per
		/// entry; `donut`: one segment per entry.
		series(Vec<Point>) = "series";
		/// `line`/`area`: several series, one line each.
		multi_series(Vec<Series>) = "series";
		/// `line`/`area`: the x labels by point index.
		labels(Vec<String>) = "labels";
		/// `line`/`area`/`donut`: written with every value printed: a currency
		/// symbol (`$`, `€`, `£`, `¥`) leads it, `%`, `×` and `°` attach after it,
		/// words follow a space.
		unit(String) = "unit";
		/// `line`/`area` with one series: the label beside the last point
		/// (`true` its value, text replaces it, `false` hides it).
		last(TextFlag) = "last";
		/// `donut`: what the ring's middle says (default the total).
		total(Text) = "total";
		/// `donut`: a quiet line under the ring.
		caption(Text) = "caption";
		/// `bars`/`spark`/`line`/`area`: the value domain's low end (default
		/// 0); values are clamped into it.
		min(f64) = "min";
		/// `bars`/`spark`/`line`/`area`: the value domain's high end
		/// (default the largest series or mark value).
		max(f64) = "max";
		/// `bars`/`spark`/`line`/`area`: dashed reference lines in value
		/// units.
		marks(Vec<ChartMark>) = "marks";
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
		placeholder(Text) = "placeholder";
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
		/// Input cannot be read yet: Tern keeps what is typed until this clears.
		typeahead(bool) = "typeahead";
		/// Cap the height at this many lines.
		max_lines(u32) = "maxLines";
		/// The look (`field` frames it as a prompt input).
		variant(InputVariant) = "variant";
		/// Buttons attached inside the ring, in order.
		buttons(Vec<ActButton>) = "buttons";
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
		placeholder(Text) = "placeholder";
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
		/// Input cannot be read yet: Tern keeps what is typed until this clears.
		typeahead(bool) = "typeahead";
		/// The look (default `field` with any addon, else `well`).
		variant(InputVariant) = "variant";
		/// A leading icon.
		icon(Icon) = "icon";
		/// In a sans region, keep the mono face for the text.
		mono(bool) = "mono";
		/// A clear × while there is text; sends `action` `clear`.
		clear(bool) = "clear";
		/// A mono well before the text.
		prefix(String) = "prefix";
		/// A mono well after the text.
		suffix(String) = "suffix";
		/// A mono unit after the text, inside the field.
		unit(String) = "unit";
		/// An attached ⌃⌄ segment; sends `action` `inc` / `dec`.
		stepper(bool) = "stepper";
		/// Draws a `•` per character, with an eye that reveals it.
		secret(bool) = "secret";
		/// A secret's shown state.
		reveal(bool) = "reveal";
		/// Buttons attached to the ring, in order.
		buttons(Vec<ActButton>) = "buttons";
	}

	/// The keycap of the chord that reaches the field, shown while it is
	/// idle and empty.
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self
			.0
			.set("keys", &keys.into_iter().map(Into::into).collect::<Vec<String>>());
		self
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
		/// Card width or full sheet; with `side`, the sheet's width or the
		/// drawer's height.
		size(OverlaySize) = "size";
		/// A dim backdrop taking the pointer.
		modal(bool) = "modal";
		/// The title row.
		head(Text) = "head";
		/// Docks it to an edge: a full-height sheet or a bottom drawer (wins
		/// over `anchor`).
		side(OverlaySide) = "side";
		/// With `side`: flush with the edge or inset from it.
		variant(OverlayVariant) = "variant";
	}

	panel_head_methods!();

	/// With `side`: the sheet's width or the drawer's height as a fraction
	/// of the layer, in (0, 1].
	pub fn size_fraction(mut self, fraction: f64) -> Self {
		self.0.set("size", &fraction);
		self
	}

	/// A node drawn in a sticky footer bar under the body (the sheet's
	/// actions).
	pub fn foot(mut self, foot: impl Into<Slot<M>>) -> Self {
		set_slot(&mut self.0, "foot", foot.into());
		self
	}
}

/// A floating panel.
pub fn overlay<M>() -> Overlay<M> {
	new("overlay")
}

common_kind!(
	/// `button`: a push button. A click sends `action` with
	/// `actions.click` (`"click"` by default); it never takes focus.
	Button
);

impl<M> Button<M> {
	props! {
		/// The label.
		text(String) = "text";
		/// A leading icon.
		icon(Icon) = "icon";
		/// The chrome.
		variant(ButtonVariant) = "variant";
		/// The height.
		size(ButtonSize) = "size";
		/// Drawn pressed (a toggle that is on).
		on(bool) = "on";
		/// Dimmed and silent; a reason string is the tooltip.
		disabled(Disabled) = "disabled";
		/// A spinner leads and it sends nothing.
		busy(bool) = "busy";
	}

	/// Keycaps drawn after the label (display only).
	pub fn keys<K: Into<String>>(mut self, keys: impl IntoIterator<Item = K>) -> Self {
		self
			.0
			.set("keys", &keys.into_iter().map(Into::into).collect::<Vec<String>>());
		self
	}
}

/// A push button labeled `text`.
pub fn button<M>(text: impl Into<String>) -> Button<M> {
	new::<M, Button<M>>("button").text(text)
}

common_kind!(
	/// `stats`: a well of stat tiles.
	Stats
);

impl<M> Stats<M> {
	props! {
		/// All tiles.
		items(Vec<StatItem>) = "items";
	}

	/// Adds a tile.
	pub fn item(mut self, item: StatItem) -> Self {
		push(&mut self.0, "items", &item);
		self
	}
}

/// A well of stat tiles.
pub fn stats<M>(items: impl IntoIterator<Item = StatItem>) -> Stats<M> {
	new::<M, Stats<M>>("stats").items(items.into_iter().collect::<Vec<_>>())
}

// ---------------------------------------------------------------------------
// Form controls
// ---------------------------------------------------------------------------

common_kind!(
	/// `select`: a value picked from a menu, as a field or a quiet trigger,
	/// optionally searchable.
	///
	/// A pick sends `change` with the option's `value`; the pointer opening
	/// or closing it sends `toggle` with `open`.
	Select
);

impl<M> Select<M> {
	props! {
		/// The current option's value.
		value(String) = "value";
		/// The options, in order.
		options(Vec<SelectOption>) = "options";
		/// The look.
		variant(SelectVariant) = "variant";
		/// A muted label before the value on the trigger.
		label(String) = "label";
		/// The trigger's text without a value (and the search line's
		/// without a query).
		placeholder(String) = "placeholder";
		/// The height.
		size(Size) = "size";
		/// Dimmed and inert; a reason string is the tooltip.
		disabled(Disabled) = "disabled";
		/// The menu shows.
		open(bool) = "open";
		/// The highlighted option's value (your keyboard cursor).
		active(String) = "active";
		/// A combobox: a search line over the options.
		search(bool) = "search";
		/// The search text.
		query(String) = "query";
		/// The caret in `query`, UTF-16.
		cursor(usize) = "cursor";
		/// The size of the whole set, counted beside the options shown.
		total(u64) = "total";
		/// The menu's text when there are no options.
		empty(String) = "empty";
	}

	/// Adds an option.
	pub fn option(mut self, option: impl Into<SelectOption>) -> Self {
		push(&mut self.0, "options", &option.into());
		self
	}
}

/// A value picked from a menu.
pub fn select<M>() -> Select<M> {
	new("select")
}

common_kind!(
	/// `switch`: an on/off setting. A click flips it at once and sends
	/// `change` with the new state as `value`.
	Switch
);

impl<M> Switch<M> {
	props! {
		/// Whether it is on.
		on(bool) = "on";
		/// The look.
		variant(SwitchVariant) = "variant";
		/// The setting's name (a labeled switch fills its row).
		label(String) = "label";
		/// A muted line under the label.
		hint(String) = "hint";
		/// Dimmed and inert; a reason string is the tooltip.
		disabled(Disabled) = "disabled";
	}

	/// The words of a `segment`'s two sides, off first.
	pub fn labels(mut self, off: impl Into<String>, on: impl Into<String>) -> Self {
		let labels: [String; 2] = [off.into(), on.into()];
		self.0.set("labels", &labels);
		self
	}
}

/// An on/off switch labeled `label`.
pub fn switch<M>(label: impl Into<String>) -> Switch<M> {
	new::<M, Switch<M>>("switch").label(label)
}

kind_type!(
	/// `slider`: a value, or a range, dragged along a track.
	///
	/// Releasing a drag that moved it sends `change` with the value (a pair
	/// for a range). Its `min`/`max` are the track's ends, not the common
	/// size bounds.
	Slider
);

impl<M> Slider<M> {
	base_methods!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// The value, or a range of two.
		value(SliderValue) = "value";
		/// The low end (default 0).
		min(f64) = "min";
		/// The high end (default 100).
		max(f64) = "max";
		/// The distance between values, counted from `min`.
		step(f64) = "step";
		/// The name at the left.
		label(String) = "label";
		/// Written after the number.
		unit(String) = "unit";
		/// How the value reads.
		format(SliderFormat) = "format";
		/// Tick marks under the track.
		ticks(Ticks) = "ticks";
		/// The look.
		variant(SliderVariant) = "variant";
		/// Dimmed and inert; a reason string is the tooltip.
		disabled(Disabled) = "disabled";
	}
}

/// A slider.
pub fn slider<M>() -> Slider<M> {
	new("slider")
}

container_kind!(
	/// `field`: a labelled form row around its control(s): label, hint,
	/// error or warning.
	Field
);

impl<M> Field<M> {
	props! {
		/// The field's name.
		label(Text) = "label";
		/// The help line.
		hint(Text) = "hint";
		/// Something is wrong: the note in `--bad` (`true` rings the
		/// control without a line).
		error(TextFlag) = "error";
		/// Worth a look: the note in `--warn`.
		warning(TextFlag) = "warning";
		/// A `*` after the label (display only).
		required(bool) = "required";
		/// `grid`: the label column's width.
		label_width(Extent) = "labelWidth";
		/// The layout.
		variant(FieldVariant) = "variant";
	}
}

/// A form row labeled `label`.
pub fn field<M>(label: impl Into<Text>) -> Field<M> {
	new::<M, Field<M>>("field").label(label)
}

common_kind!(
	/// `otp`: a one-time code, one cell per character.
	///
	/// Your program owns the keys; a click on a cell sends `action` `focus`
	/// with the cell as `value`.
	Otp
);

impl<M> Otp<M> {
	props! {
		/// The characters typed so far.
		text(String) = "text";
		/// The number of cells (1–12).
		length(u32) = "length";
		/// Cells per group (`0`: one group).
		group(u32) = "group";
		/// The look.
		variant(OtpVariant) = "variant";
		/// Hides what was typed.
		mask(OtpMask) = "mask";
		/// `error` or `success`.
		state(OtpState) = "state";
		/// A quiet line under the cells.
		hint(String) = "hint";
		/// Dimmed and inert; a reason string is the tooltip.
		disabled(Disabled) = "disabled";
	}

	/// The active cell (0-based; default after the text); `None` shows
	/// none.
	pub fn cursor(mut self, cursor: Option<u32>) -> Self {
		match cursor {
			Some(cell) => self.0.set("cursor", &cell),
			None => self.0.set("cursor", &false),
		}
		self
	}
}

/// A one-time code of `length` cells.
pub fn otp<M>(length: u32) -> Otp<M> {
	new::<M, Otp<M>>("otp").length(length)
}

container_kind!(
	/// `buttons`: `button` children drawn as one group: joined, split or a
	/// toolbar.
	Buttons
);

impl<M> Buttons<M> {
	props! {
		/// The look.
		variant(ButtonsVariant) = "variant";
		/// The children's size.
		size(ButtonSize) = "size";
		/// The group's accessible name.
		label(String) = "label";
	}
}

/// A group of buttons.
pub fn buttons<M>() -> Buttons<M> {
	new("buttons")
}

kind_type!(
	/// `calendar`: a month grid or a week strip of days.
	///
	/// A pick sends `change` with the day (a `[from, to]` range with
	/// `range`), a preset `select`, a month step `action` `month`. Its
	/// `min`/`max` are days, not the common size bounds.
	Calendar
);

impl<M> Calendar<M> {
	base_methods!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// The look.
		variant(CalendarVariant) = "variant";
		/// The selection: a day, or a range.
		value(DateValue) = "value";
		/// Pick a range instead of a day.
		range(bool) = "range";
		/// The month shown, `YYYY-MM`.
		month(String) = "month";
		/// The day ringed as today, `YYYY-MM-DD`.
		today(String) = "today";
		/// The first day that can be picked.
		min(String) = "min";
		/// The last day that can be picked.
		max(String) = "max";
		/// The first column of the month grid.
		week_start(WeekStart) = "weekStart";
		/// Days carrying a dot.
		marks(Vec<DayMark>) = "marks";
		/// The day your keys are on.
		cursor(String) = "cursor";
		/// `week`: the presets column.
		presets(Vec<Preset>) = "presets";
		/// The active preset's id.
		preset(String) = "preset";
		/// A muted line under the days.
		note(String) = "note";
	}
}

/// A calendar.
pub fn calendar<M>() -> Calendar<M> {
	new("calendar")
}

kind_type!(
	/// `date`: a date field that opens a calendar popover.
	///
	/// It sends the calendar's events, and `toggle` when a click opens or
	/// closes it. Its `min`/`max` are days, not the common size bounds.
	Date
);

impl<M> Date<M> {
	base_methods!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// The selection: a day, or a range.
		value(DateValue) = "value";
		/// Pick a range; the popover stays open until its second end.
		range(bool) = "range";
		/// The popover shows.
		open(bool) = "open";
		/// Shown in the field without a value.
		placeholder(String) = "placeholder";
		/// The popover's month, `YYYY-MM`.
		month(String) = "month";
		/// The day ringed as today, `YYYY-MM-DD`.
		today(String) = "today";
		/// The first day that can be picked.
		min(String) = "min";
		/// The last day that can be picked.
		max(String) = "max";
		/// The first column of the month grid.
		week_start(WeekStart) = "weekStart";
		/// Days carrying a dot.
		marks(Vec<DayMark>) = "marks";
		/// The day your keys are on.
		cursor(String) = "cursor";
		/// A muted line under the days.
		note(String) = "note";
	}
}

/// A date field.
pub fn date<M>() -> Date<M> {
	new("date")
}

// ---------------------------------------------------------------------------
// Menus and navigation
// ---------------------------------------------------------------------------

common_kind!(
	/// `menu`: a button that opens a menu of commands.
	///
	/// Choosing a row sends `action` with its id as `act` (and `checked` for
	/// check and radio rows); the pointer opening or closing it sends
	/// `toggle` with `open`.
	Menu
);

impl<M> Menu<M> {
	props! {
		/// The rows.
		items(Vec<MenuEntry>) = "items";
		/// The trigger's label (without it and `icon`, a … button).
		text(String) = "text";
		/// The trigger's icon.
		icon(Icon) = "icon";
		/// The trigger's chrome.
		trigger(ButtonVariant) = "trigger";
		/// A chevron after the label.
		caret(bool) = "caret";
		/// The trigger's height.
		size(ButtonSize) = "size";
		/// The menu's look.
		variant(MenuVariant) = "variant";
		/// The menu shows.
		open(bool) = "open";
		/// The highlighted row's id (your keyboard cursor).
		active(String) = "active";
		/// The trigger edge the menu lines up with.
		align(MenuAlign) = "align";
		/// The trigger is dimmed and does not open; a reason string is the
		/// tooltip.
		disabled(Disabled) = "disabled";
	}

	/// Adds a row.
	pub fn item(mut self, entry: impl Into<MenuEntry>) -> Self {
		push(&mut self.0, "items", &entry.into());
		self
	}
}

/// A menu button.
pub fn menu<M>() -> Menu<M> {
	new("menu")
}

common_kind!(
	/// `menubar`: a strip of menus.
	///
	/// Choosing a row sends `action` with the menu as `item`; the pointer
	/// opening or closing a menu sends `toggle` with `item` and `open`.
	Menubar
);

impl<M> Menubar<M> {
	props! {
		/// The menus, in order.
		menus(Vec<MenubarMenu>) = "menus";
		/// The look.
		variant(MenubarVariant) = "variant";
		/// The app's mark before the titles.
		icon(Icon) = "icon";
		/// The id of the menu shown open.
		open(String) = "open";
		/// The highlighted row's id in the open menu.
		active(String) = "active";
	}

	/// Adds a menu.
	pub fn menu(mut self, menu: MenubarMenu) -> Self {
		push(&mut self.0, "menus", &menu);
		self
	}
}

/// A strip of menus.
pub fn menubar<M>() -> Menubar<M> {
	new("menubar")
}

common_kind!(
	/// `nav`: navigation links, some opening a menu or panel of sub-items.
	/// A pick sends `select` with the item.
	Nav
);

impl<M> Nav<M> {
	props! {
		/// The items, in order.
		items(Vec<NavItem>) = "items";
		/// The active item's or sub-item's id.
		active(String) = "active";
		/// The id of the item whose menu or panel shows.
		open(String) = "open";
		/// The look.
		variant(NavVariant) = "variant";
	}

	/// Adds an item.
	pub fn item(mut self, item: NavItem) -> Self {
		push(&mut self.0, "items", &item);
		self
	}
}

/// Navigation links.
pub fn nav<M>() -> Nav<M> {
	new("nav")
}

kind_type!(
	/// `breadcrumb`: a trail of places, the last one current.
	///
	/// A click on a crumb sends `select` with its id. Its `max` is a crumb
	/// count, not the common size bounds.
	Breadcrumb
);

impl<M> Breadcrumb<M> {
	base_methods!();

	min_method!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// The trail, root first.
		items(Vec<Crumb>) = "items";
		/// The look.
		variant(BreadcrumbVariant) = "variant";
		/// Most crumbs shown; the rest fold into a "…" menu.
		max(u32) = "max";
		/// The fold's menu shows.
		open(bool) = "open";
		/// The id of the crumb under your keyboard cursor.
		active(String) = "active";
	}

	/// Adds a crumb.
	pub fn crumb(mut self, crumb: Crumb) -> Self {
		push(&mut self.0, "items", &crumb);
		self
	}
}

/// A trail of places.
pub fn breadcrumb<M>() -> Breadcrumb<M> {
	new("breadcrumb")
}

common_kind!(
	/// `pager`: pagination. A page click sends `change` with the page; the
	/// `more` row sends `action` `more`.
	Pager
);

impl<M> Pager<M> {
	props! {
		/// The look.
		variant(PagerVariant) = "variant";
		/// The current page, 1-based (`more`: the pages loaded).
		page(u64) = "page";
		/// The page count (default from `total` / `size`).
		pages(u64) = "pages";
		/// The item count.
		total(u64) = "total";
		/// Items per page.
		size(u64) = "size";
		/// A turning ring; the more row sends nothing.
		loading(bool) = "loading";
		/// What the items are, plural.
		label(String) = "label";
	}
}

/// Pagination.
pub fn pager<M>() -> Pager<M> {
	new("pager")
}

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

kind_type!(
	/// `split`: panes (its children) the user resizes by dragging the
	/// handles between them.
	///
	/// A release sends `change` with the shares. Its `min` is the panes'
	/// minimums (or the common bound).
	Split
);

impl<M> Split<M> {
	base_methods!();

	title_method!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The look.
		variant(SplitVariant) = "variant";
		/// Side by side or stacked.
		dir(SplitDir) = "dir";
		/// Each pane's share, in child order.
		sizes(Vec<f64>) = "sizes";
		/// The panes' minimums along the split.
		min(PaneMin) = "min";
		/// The split's height.
		h(Extent) = "h";
		/// The handle your keys move (0 is after the first pane).
		active(u32) = "active";
	}
}

/// Resizable panes.
pub fn split<M>() -> Split<M> {
	new("split")
}

container_kind!(
	/// `scroll`: a scrolling container, optionally pinned to its end.
	/// Scrolling is the viewer's; it sends nothing.
	Scroll
);

impl<M> Scroll<M> {
	props! {
		/// The look.
		variant(ScrollVariant) = "variant";
		/// Keep to the end while content grows.
		stick(Stick) = "stick";
		/// A stuck view follows its end.
		pinned(bool) = "pinned";
		/// When the thumb shows.
		bar(ScrollBar) = "bar";
		/// Space between the children.
		gap(Gap) = "gap";
		/// What the counts name.
		noun(Noun) = "noun";
	}
}

/// A scrolling container.
pub fn scroll<M>() -> Scroll<M> {
	new("scroll")
}

container_kind!(
	/// `carousel`: slides (its children) shown one at a time. A move by
	/// the user sends `change` with the index shown.
	Carousel
);

impl<M> Carousel<M> {
	props! {
		/// The slide shown, from 0.
		index(u32) = "index";
		/// Moving past either end wraps around.
		looping(bool) = "loop";
		/// The look.
		variant(CarouselVariant) = "variant";
		/// The slides' height (default 200px).
		h(Height) = "h";
	}
}

/// Slides shown one at a time.
pub fn carousel<M>() -> Carousel<M> {
	new("carousel")
}

// ---------------------------------------------------------------------------
// People and placeholders
// ---------------------------------------------------------------------------

common_kind!(
	/// `avatar`: a person, bot or org as initials, an icon or a picture.
	Avatar
);

impl<M> Avatar<M> {
	props! {
		/// The shape.
		variant(AvatarVariant) = "variant";
		/// The name: its first two characters are the initials.
		text(String) = "text";
		/// Picks the colors (default `text`).
		seed(String) = "seed";
		/// An icon drawn instead of the initials.
		icon(Icon) = "icon";
		/// A picture's blob id.
		blob(String) = "blob";
		/// The size.
		size(AvatarSize) = "size";
		/// A status dot in the tone's color.
		status(Tone) = "status";
	}
}

/// An avatar of `text`.
pub fn avatar<M>(text: impl Into<String>) -> Avatar<M> {
	new::<M, Avatar<M>>("avatar").text(text)
}

kind_type!(
	/// `avatars`: avatars stacked, with an overflow count. Its `max` is an
	/// avatar count, not the common size bounds.
	Avatars
);

impl<M> Avatars<M> {
	base_methods!();

	min_method!();

	title_method!();

	actions_methods!();

	event_methods!();

	props! {
		/// The avatars, front first.
		items(Vec<AvatarItem>) = "items";
		/// How many show before the `+N` chip.
		max(u32) = "max";
		/// Every avatar's size.
		size(AvatarSize) = "size";
		/// Muted text after the stack, and its accessible name.
		label(String) = "label";
	}
}

/// A stack of avatars.
pub fn avatars<M>(items: impl IntoIterator<Item = AvatarItem>) -> Avatars<M> {
	new::<M, Avatars<M>>("avatars").items(items.into_iter().collect::<Vec<_>>())
}

kind_type!(
	/// `empty`: an empty state: icon, title, a line and its children (the
	/// actions). Its `title` is text, not a tooltip.
	Empty
);

impl<M> Empty<M> {
	base_methods!();

	min_method!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The look.
		variant(EmptyVariant) = "variant";
		/// The icon (`""` for none).
		icon(Icon) = "icon";
		/// What is missing, in a few words.
		title(Text) = "title";
		/// One muted line: why, or what to do.
		text(Text) = "text";
		/// `inline`: a hairline card around the row.
		frame(bool) = "frame";
		/// `drop`: a drag is over the zone.
		over(bool) = "over";
	}
}

/// An empty state titled `title`.
pub fn empty<M>(title: impl Into<Text>) -> Empty<M> {
	new::<M, Empty<M>>("empty").title(title)
}

common_kind!(
	/// `skeleton`: placeholder shapes while content loads.
	Skeleton
);

impl<M> Skeleton<M> {
	props! {
		/// The look.
		variant(SkeletonVariant) = "variant";
		/// What it stands for.
		shape(SkeletonShape) = "shape";
		/// `text`: how many lines.
		lines(u32) = "lines";
		/// `rows`: how many rows.
		rows(u32) = "rows";
		/// `card`: how many stat tiles.
		tiles(u32) = "tiles";
	}
}

/// Placeholder shapes.
pub fn skeleton<M>() -> Skeleton<M> {
	new("skeleton")
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

	min_method!();

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
		/// The state (`ask` waits for approval).
		status(Status) = "status";
		/// The approval look while `status` is `ask`.
		confirm(ToolConfirm) = "confirm";
		/// The approval panel's contents. Its buttons send `action` `allow`,
		/// `deny` or `always` with the selected scope as `value`.
		ask(Ask) = "ask";
		/// The resolved answer, shown as a head chip.
		approved(Approved) = "approved";
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

	min_method!();

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

// ---------------------------------------------------------------------------
// Chat and agent work
// ---------------------------------------------------------------------------

kind_type!(
	/// `message`: one chat message: author, body (its children), reactions
	/// and actions.
	///
	/// A reaction flip sends `change` with the chip's text and `on`; an
	/// action button sends `action`. Its `actions` are icon buttons, not the
	/// common pointer actions.
	Message
);

impl<M> Message<M> {
	base_methods!();

	min_method!();

	title_method!();

	max_method!();

	event_methods!();

	children_methods!();

	props! {
		/// The look.
		variant(MessageVariant) = "variant";
		/// `end` is the local user.
		side(MessageSide) = "side";
		/// The name in the header.
		author(String) = "author";
		/// The time in the header.
		time(String) = "time";
		/// Same author as the message before: no header, no avatar.
		continued(bool) = "continued";
		/// Reaction chips.
		reactions(Vec<Reaction>) = "reactions";
		/// Icon buttons shown on hover.
		actions(Vec<IconAction>) = "actions";
		/// When the actions show.
		reveal(Reveal) = "reveal";
		/// A quiet line under the message.
		status(String) = "status";
		/// Still on its way: the status shimmers.
		pending(bool) = "pending";
	}

	/// The avatar: an `avatar`, `icon` or `image` node drawn in the avatar
	/// slot (or the id of a child).
	pub fn avatar(mut self, avatar: impl Into<Slot<M>>) -> Self {
		set_slot(&mut self.0, "avatar", avatar.into());
		self
	}
}

/// A chat message.
pub fn message<M>() -> Message<M> {
	new("message")
}

kind_type!(
	/// `attachment`: a file attached to a message, with its upload state.
	///
	/// Its buttons send `action` (`remove`, `retry` or one of `actions`).
	/// Its `actions` are icon buttons, not the common pointer actions.
	Attachment
);

impl<M> Attachment<M> {
	base_methods!();

	min_method!();

	title_method!();

	max_method!();

	event_methods!();

	props! {
		/// The file name.
		name(String) = "name";
		/// The look.
		variant(AttachmentVariant) = "variant";
		/// The extension shown (default from `name`).
		ext(String) = "ext";
		/// The MIME type.
		mime(String) = "mime";
		/// Size in bytes.
		size(u64) = "size";
		/// The meta line at rest.
		meta(String) = "meta";
		/// An image blob id: the thumbnail.
		blob(String) = "blob";
		/// A file opened by a click when there is no `href`.
		path(String) = "path";
		/// The upload state.
		state(AttachmentState) = "state";
		/// 0–1 while busy.
		progress(f64) = "progress";
		/// The meta line in the `error` state.
		error(String) = "error";
		/// A × that sends `remove`.
		removable(bool) = "removable";
		/// Action icons before the ×.
		actions(Vec<IconAction>) = "actions";
	}
}

/// An attached file named `name`.
pub fn attachment<M>(name: impl Into<String>) -> Attachment<M> {
	new::<M, Attachment<M>>("attachment").name(name)
}

kind_type!(
	/// `trace`: an agent's steps, each opening in place.
	///
	/// A child keyed by a line's id is that line's content; the rest is the
	/// body. Flipping a line or the head sends `toggle` with `open` (and the
	/// line as `item`). Its `title` is text, not a tooltip.
	Trace
);

impl<M> Trace<M> {
	base_methods!();

	min_method!();

	max_method!();

	actions_methods!();

	event_methods!();

	children_methods!();

	props! {
		/// The look.
		variant(TraceVariant) = "variant";
		/// The steps, in order.
		lines(Vec<TraceLine>) = "lines";
		/// The whole trace's status.
		status(Status) = "status";
		/// The head text.
		title(Text) = "title";
		/// The `summary` head's text (default derived).
		summary(Text) = "summary";
		/// The head glyph.
		icon(Icon) = "icon";
		/// Elapsed ms when sent.
		age(u64) = "age";
		/// The total time in ms once settled.
		took(u64) = "took";
		/// Whether the head's fold is open.
		open(bool) = "open";
		/// The `Sources` row at the foot.
		sources(Vec<Source>) = "sources";
	}

	/// Adds a step.
	pub fn line(mut self, line: TraceLine) -> Self {
		push(&mut self.0, "lines", &line);
		self
	}
}

/// An agent's steps.
pub fn trace<M>() -> Trace<M> {
	new("trace")
}

common_kind!(
	/// `stacktrace`: an error and its stack frames.
	///
	/// A click on a frame sends `select`, a double click `activate`, with
	/// the frame's index as `item`.
	Stacktrace
);

impl<M> Stacktrace<M> {
	props! {
		/// The error.
		error(StackError) = "error";
		/// The stack, innermost first.
		frames(Vec<Frame>) = "frames";
		/// The selected frame's index.
		selected(u32) = "selected";
		/// The excerpts' language.
		lang(String) = "lang";
		/// The look.
		variant(StacktraceVariant) = "variant";
	}

	/// Adds a frame.
	pub fn frame(mut self, frame: Frame) -> Self {
		push(&mut self.0, "frames", &frame);
		self
	}
}

/// An error and its stack.
pub fn stacktrace<M>(error: StackError) -> Stacktrace<M> {
	new::<M, Stacktrace<M>>("stacktrace").error(error)
}

common_kind!(
	/// `suggestions`: suggested prompts. A click sends `activate` with the
	/// item's id (else its text).
	Suggestions
);

impl<M> Suggestions<M> {
	props! {
		/// The look.
		variant(SuggestionsVariant) = "variant";
		/// The prompts, in order.
		items(Vec<Suggestion>) = "items";
	}

	/// Adds a prompt.
	pub fn item(mut self, item: Suggestion) -> Self {
		push(&mut self.0, "items", &item);
		self
	}
}

/// Suggested prompts.
pub fn suggestions<M>() -> Suggestions<M> {
	new("suggestions")
}

container_kind!(
	/// `questions`: a questionnaire.
	///
	/// A choice sends `change` with the question as `item`; Next, Back and
	/// Skip send `action` with the question left as `item`; Submit sends
	/// `action` `submit` with every answer as `value`. An `input` child keyed
	/// `<id>.other` takes a question's free-text answer.
	Questions
);

impl<M> Questions<M> {
	props! {
		/// The questions.
		items(Vec<Question>) = "items";
		/// The current question's index.
		step(u32) = "step";
		/// The answers by question id.
		answers(std::collections::BTreeMap<String, QuestionAnswer>) = "answers";
		/// The look.
		variant(QuestionsVariant) = "variant";
		/// `inline`'s hint line (`""` hides it).
		hint(String) = "hint";
		/// The submit label.
		submit(String) = "submit";
	}

	/// Adds a question.
	pub fn question(mut self, question: Question) -> Self {
		push(&mut self.0, "items", &question);
		self
	}
}

/// A questionnaire.
pub fn questions<M>() -> Questions<M> {
	new("questions")
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
	fn json_carries_its_value_and_open_levels() {
		assert_eq!(
			wire::<()>(super::json(json!({"a": 1})).open(1)),
			json!({"k": "json", "p": {"value": {"a": 1}, "open": 1}})
		);
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
	fn code_bars_folds_and_overview_serialize() {
		let c = code::<()>("a\nb\nc")
			.marks(vec![CodeMark { line: 1, wash: vec![0.9, 0.3], ..CodeMark::default() }])
			.folds(vec![
				CodeFold {
					from: 2,
					to: 3,
					note: Some("cut".into()),
					tone: Some(Tone::Error),
					hovercard: Some(Node::<()>::from(text("why")).to_json()),
					..CodeFold::default()
				},
				CodeFold { from: 5, to: 6, ..CodeFold::default() },
			])
			.overview(true)
			.nums(vec![1, 2, 7]);
		assert_eq!(
			wire(c),
			json!({"k": "code", "p": {
				"text": "a\nb\nc", "nums": [1, 2, 7], "marks": [{"line": 1, "wash": [0.9, 0.3]}],
				"folds": [
					{"from": 2, "to": 3, "note": "cut", "tone": "error", "hovercard": {"k": "text", "p": {"text": "why"}}},
					{"from": 5, "to": 6}
				],
				"overview": true
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
	fn form_controls_serialize_their_props() {
		assert_eq!(
			wire(
				select::<()>()
					.label("Model")
					.value("haiku")
					.option("opus")
					.option(
						SelectOption::new("haiku", "Haiku")
							.detail("200k")
							.group("Fast")
					)
					.search(true)
			)["p"],
			json!({
				"label": "Model", "value": "haiku", "search": true, "options": [
					{"value": "opus"},
					{"value": "haiku", "label": "Haiku", "detail": "200k", "group": "Fast"}
				]
			})
		);
		assert_eq!(
			wire(
				switch::<()>("Wrap")
					.on(true)
					.variant(SwitchVariant::Segment)
					.labels("Off", "On")
			),
			json!({"k": "switch", "p": {
				"label": "Wrap", "on": true, "variant": "segment", "labels": ["Off", "On"]
			}})
		);
		assert_eq!(
			wire(
				slider::<()>()
					.value((200.0, 1200.0))
					.min(0.0)
					.max(2000.0)
					.ticks(4u32)
					.unit("ms")
			)["p"],
			json!({"value": [200.0, 1200.0], "min": 0.0, "max": 2000.0, "ticks": 4, "unit": "ms"})
		);
		assert_eq!(
			wire(
				field::<()>("Name")
					.required(true)
					.error(true)
					.label_width(Extent::Ch(14.0))
					.child(input())
			)["p"],
			json!({"label": "Name", "required": true, "error": true, "labelWidth": "14ch"})
		);
		assert_eq!(
			wire(otp::<()>(6).text("12").mask("*").cursor(None))["p"],
			json!({"length": 6, "text": "12", "mask": "*", "cursor": false})
		);
		assert_eq!(
			wire(
				calendar::<()>()
					.range(true)
					.value(DateValue::Range("2026-10-19".into(), None))
					.week_start(WeekStart::Sun)
					.marks(vec![DayMark::from("2026-10-20"), DayMark::new("2026-10-21", Tone::Error)])
			)["p"],
			json!({
				"range": true, "value": ["2026-10-19", null], "weekStart": "sun",
				"marks": [{"date": "2026-10-20"}, {"date": "2026-10-21", "tone": "error"}]
			})
		);
	}

	#[test]
	fn menu_entries_serialize_as_strings_headers_and_items() {
		let entries = vec![
			MenuEntry::from("open"),
			MenuEntry::header("Danger"),
			MenuEntry::Separator,
			MenuItem::new("trash", "Move to Trash")
				.icon("trash")
				.keys(["cmd", "backspace"])
				.danger(true)
				.into(),
			MenuItem::new("export", "Export")
				.item(MenuItem::new("csv", "CSV").checked(true))
				.into(),
		];
		let expected = json!([
			"open", {"header": "Danger"}, "-",
			{"id": "trash", "label": "Move to Trash", "icon": "trash",
				"keys": ["cmd", "backspace"], "danger": true},
			{"id": "export", "label": "Export",
				"items": [{"id": "csv", "label": "CSV", "checked": true}]}
		]);
		assert_eq!(wire(menu::<()>().text("More").items(entries.clone()))["p"]["items"], expected);
		let actions = entries
			.into_iter()
			.fold(Actions::new(), |actions, entry| actions.menu(entry));
		assert_eq!(wire(text::<()>("log").actions(actions))["p"]["actions"]["menu"], expected);
	}

	#[test]
	fn kinds_with_their_own_min_max_title_and_actions_keep_them() {
		assert_eq!(wire(avatars::<()>([AvatarItem::new("Cara")]).max(4u32))["p"]["max"], 4);
		assert_eq!(wire(breadcrumb::<()>().crumb(Crumb::new("src", "src")).max(3u32))["p"]["max"], 3);
		assert_eq!(
			wire(
				split::<()>()
					.min(vec![Some(Extent::Ch(20.0)), None])
					.sizes(vec![2.0, 1.0])
			)["p"],
			json!({"min": ["20ch", null], "sizes": [2.0, 1.0]})
		);
		assert_eq!(
			wire(empty::<()>(spans![span("No runs").style("strong")]))["p"]["title"],
			json!([{"t": "No runs", "s": "strong"}])
		);
		assert_eq!(
			wire(attachment::<()>("a.png").actions(vec![IconAction::new("open").icon("eye")]))["p"],
			json!({"name": "a.png", "actions": [{"act": "open", "icon": "eye"}]})
		);
		assert_eq!(
			wire(carousel::<()>().looping(true).h(Height::Lines(10.0)))["p"],
			json!({"loop": true, "h": "10lines"})
		);
	}

	#[test]
	fn work_kinds_serialize_lines_frames_and_questions() {
		let t = trace::<()>().status(Status::Running).line(
			TraceLine::new("s1", "Searched")
				.target("tern tsp")
				.status(Status::Done)
				.took(1200)
				.source(Source::new("https://x.dev").site("x")),
		);
		assert_eq!(
			wire(t)["p"],
			json!({"status": "running", "lines": [{
				"id": "s1", "title": "Searched", "target": "tern tsp", "status": "done",
				"took": 1200, "sources": [{"href": "https://x.dev", "site": "x"}]
			}]})
		);
		let s = stacktrace::<()>(StackError::new("TypeError", "x is undefined"))
			.frame(Frame::new("main", "src/a.ts").at(3, 7).app(false));
		assert_eq!(
			wire(s)["p"],
			json!({
				"error": {"type": "TypeError", "message": "x is undefined"},
				"frames": [{"fn": "main", "file": "src/a.ts", "line": 3, "col": 7, "app": false}]
			})
		);
		let q = questions::<()>()
			.question(
				Question::new("db", "Which database?")
					.choice("postgres")
					.choice(QuestionChoice::new("sqlite", "SQLite"))
					.other("Something else"),
			)
			.answers(std::collections::BTreeMap::from([(
				"db".to_owned(),
				QuestionAnswer::from("postgres"),
			)]));
		assert_eq!(
			wire(q)["p"],
			json!({
				"items": [{"id": "db", "prompt": "Which database?",
					"choices": [{"value": "postgres"}, {"value": "sqlite", "label": "SQLite"}],
					"other": {"placeholder": "Something else"}}],
				"answers": {"db": "postgres"}
			})
		);
		let m = message::<()>()
			.author("Cara")
			.side(MessageSide::End)
			.reactions(vec![Reaction::new("👍", 2).on(true)])
			.avatar(avatar("Cara"))
			.child(md("hi"));
		let node: Node<()> = m.into();
		assert_eq!(node.props()["reactions"], json!([{"text": "👍", "count": 2, "on": true}]));
		assert_eq!(node.child_nodes()[0].props()["key"], "avatar");
		assert_eq!(node.child_nodes()[1].kind(), "md");
	}

	#[test]
	fn extended_kinds_take_their_new_props() {
		let t = table::<()>()
			.col(Column::new("name", "Name").sortable(true))
			.sort(Sort::desc("name"))
			.multi(true)
			.picked(vec!["a".to_owned()])
			.bar_button(ActButton::new("restart", "Restart").icon("refresh"));
		assert_eq!(
			wire(t)["p"],
			json!({
				"cols": [{"id": "name", "head": "Name", "sortable": true}],
				"sort": {"col": "name", "dir": "desc"}, "multi": true, "picked": ["a"],
				"bar": [{"act": "restart", "text": "Restart", "icon": "refresh"}]
			})
		);
		assert_eq!(
			wire(
				chart::<()>(ChartKind::Line)
					.multi_series(vec![Series::new(vec![Some(1.0), None]).label("p50")])
					.unit("ms")
					.last(false)
			)["p"],
			json!({
				"kind": "line", "series": [{"label": "p50", "values": [1.0, null]}],
				"unit": "ms", "last": false
			})
		);
		assert_eq!(
			wire(
				input::<()>()
					.icon("search")
					.keys(["cmd", "f"])
					.clear(true)
					.secret(true)
			)["p"],
			json!({"icon": "search", "keys": ["cmd", "f"], "clear": true, "secret": true})
		);
		assert_eq!(
			wire(overlay::<()>().side(OverlaySide::Right).size_fraction(0.4))["p"],
			json!({"side": "right", "size": 0.4})
		);
		assert_eq!(
			wire(
				tool::<()>("bash")
					.status(Status::Ask)
					.confirm(ToolConfirm::Scopes)
					.ask(
						Ask::new("Run it?")
							.scope_option(AskScope::new("once", "Once"))
							.always(AskAlways::Hidden)
					)
			)["p"],
			json!({
				"name": "bash", "status": "ask", "confirm": "scopes",
				"ask": {"text": "Run it?", "always": false, "scopes": [{"id": "once", "label": "Once"}]}
			})
		);
		assert_eq!(
			wire(
				rule::<()>()
					.variant(RuleVariant::Note)
					.label("Resumed")
					.time("14:02")
			)["p"],
			json!({"variant": "note", "label": "Resumed", "time": "14:02"})
		);
		let cited = text::<()>(spans![
			span("Rust 2024"),
			span("").cite(Cite::new("https://blog.rust-lang.org").n(2))
		])
		.cite(CiteStyle::Note);
		assert_eq!(
			wire(cited)["p"],
			json!({"cite": "note", "spans": [
				{"t": "Rust 2024"}, {"t": "", "cite": [{"href": "https://blog.rust-lang.org", "n": 2}]}
			]})
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
