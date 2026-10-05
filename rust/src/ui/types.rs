//! Typed prop values: styled text, enumerations (each with an escape hatch
//! for values newer than the SDK), sizes and actions.

use serde::{Serialize, Serializer};

/// Defines a string enumeration with an `Other` escape hatch.
macro_rules! string_enum {
	($(#[$m:meta])* $name:ident { $($(#[$vm:meta])* $var:ident = $s:literal),* $(,)? }) => {
		$(#[$m])*
		#[derive(Clone, Debug, PartialEq, Eq, Hash)]
		pub enum $name {
			$($(#[$vm])* $var,)*
			/// A value this SDK doesn't name.
			Other(String),
		}

		impl $name {
			/// The value on the wire.
			pub fn as_str(&self) -> &str {
				match self {
					$(Self::$var => $s,)*
					Self::Other(s) => s,
				}
			}
		}

		impl From<&str> for $name {
			fn from(s: &str) -> Self {
				match s {
					$($s => Self::$var,)*
					_ => Self::Other(s.to_owned()),
				}
			}
		}

		impl From<String> for $name {
			fn from(s: String) -> Self {
				Self::from(s.as_str())
			}
		}

		impl ::std::fmt::Display for $name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				f.write_str(self.as_str())
			}
		}

		impl ::serde::Serialize for $name {
			fn serialize<S: ::serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
				s.serialize_str(self.as_str())
			}
		}
	};
}

pub(crate) use string_enum;

string_enum!(
	/// The semantic color of a node's chrome.
	Tone {
		/// Gray.
		Neutral = "neutral",
		/// The accent.
		Accent = "accent",
		/// Informational.
		Info = "info",
		/// Success.
		Success = "success",
		/// Warning.
		Warning = "warning",
		/// Error.
		Error = "error",
		/// Pending.
		Pending = "pending",
		/// Quiet.
		Muted = "muted",
		/// The user's own.
		User = "user",
	}
);

string_enum!(
	/// The state of a card or tool.
	Status {
		/// Not started.
		Pending = "pending",
		/// In progress.
		Running = "running",
		/// Finished.
		Done = "done",
		/// Failed.
		Error = "error",
		/// Stopped.
		Cancelled = "cancelled",
	}
);

string_enum!(
	/// The state of an agent.
	AgentStatus {
		/// Not started.
		Pending = "pending",
		/// In progress.
		Running = "running",
		/// Finished.
		Done = "done",
		/// Failed.
		Failed = "failed",
		/// Stopped.
		Aborted = "aborted",
		/// Waiting for work.
		Idle = "idle",
		/// Parked.
		Parked = "parked",
	}
);

string_enum!(
	/// A transient selection mark drawn over a node.
	Mark {
		/// An accent bar at the node's left.
		Pick = "pick",
		/// Dims the node.
		Drop = "drop",
	}
);

string_enum!(
	/// Space between children, or a spacer's height.
	Gap {
		/// None.
		None = "none",
		/// 2px.
		Xs = "xs",
		/// 6px.
		Sm = "sm",
		/// 10px (12px for a spacer).
		Md = "md",
		/// 16px (20px for a spacer).
		Lg = "lg",
	}
);

string_enum!(
	/// Cross-axis alignment of a row's or column's children.
	Align {
		/// Start.
		Start = "start",
		/// Center.
		Center = "center",
		/// End.
		End = "end",
		/// Text baselines.
		Baseline = "baseline",
		/// Stretch.
		Stretch = "stretch",
	}
);

string_enum!(
	/// Main-axis distribution of a row's or column's children.
	Justify {
		/// Space between.
		Between = "between",
		/// Packed at the end.
		End = "end",
	}
);

string_enum!(
	/// How text wraps.
	Wrap {
		/// Between words.
		Word = "word",
		/// Anywhere.
		Char = "char",
		/// Never.
		None = "none",
	}
);

string_enum!(
	/// Where overflowing text is cut.
	Truncate {
		/// Keep the start.
		End = "end",
		/// Keep the end.
		Start = "start",
		/// Keep both ends.
		Middle = "middle",
	}
);

string_enum!(
	/// Text alignment in a table column.
	TextAlign {
		/// Start.
		Start = "start",
		/// Center.
		Center = "center",
		/// End (numbers).
		End = "end",
	}
);

string_enum!(
	/// A span effect.
	Fx {
		/// A highlight sweeping across.
		Shimmer = "shimmer",
		/// Fades in and out.
		Pulse = "pulse",
		/// None.
		None = "none",
	}
);

string_enum!(
	/// A span style token (several join with spaces; palette tokens through
	/// `Other`).
	Token {
		/// Quiet text.
		Muted = "muted",
		/// Quieter text.
		Dim = "dim",
		/// Weight 600.
		Strong = "strong",
		/// Italic.
		Em = "em",
		/// The accent color.
		Accent = "accent",
		/// Success color.
		Success = "success",
		/// Warning color.
		Warning = "warning",
		/// Error color.
		Error = "error",
		/// Info color.
		Info = "info",
		/// An inline code chip.
		Code = "code",
		/// Tabular figures.
		Mono = "mono",
		/// A file path.
		Path = "path",
		/// A keycap.
		Key = "key",
		/// Link color.
		Link = "link",
		/// A number.
		Num = "num",
		/// Inserted text.
		Ins = "ins",
		/// Deleted text.
		Del = "del",
		/// A highlight.
		Mark = "mark",
		/// A misspelled word.
		Typo = "typo",
		/// A Nerd Font glyph.
		Icon = "icon",
		/// Not drawn.
		Hide = "hide",
	}
);

string_enum!(
	/// A spinner's indicator.
	SpinnerStyle {
		/// Braille dots.
		Braille = "braille",
		/// Three pulsing dots.
		Dots = "dots",
		/// omp's thinking glyph.
		Starburst = "starburst",
		/// A dot circling a ring.
		Orbit = "orbit",
	}
);

string_enum!(
	/// How a shimmer sweeps.
	ShimmerMode {
		/// Left to right, restarting.
		Classic = "classic",
		/// Back and forth.
		Kitt = "kitt",
	}
);

string_enum!(
	/// How an elapsed timer reads.
	ElapsedFormat {
		/// `1m 5s`.
		Short = "short",
		/// `1:05`.
		Clock = "clock",
	}
);

string_enum!(
	/// A meter's shape.
	MeterStyle {
		/// A rounded track.
		Bar = "bar",
		/// A ring.
		Ring = "ring",
		/// A grid of squares.
		Blocks = "blocks",
	}
);

string_enum!(
	/// Small, medium or large.
	Size {
		/// Small.
		Sm = "sm",
		/// Medium.
		Md = "md",
		/// Large.
		Lg = "lg",
	}
);

string_enum!(
	/// Which chart.
	ChartKind {
		/// A heatmap of cells.
		Heatmap = "heatmap",
		/// Bars.
		Bars = "bars",
		/// A sparkline.
		Spark = "spark",
	}
);

string_enum!(
	/// A thinking-effort rung.
	EffortLevel {
		/// 0/6.
		Off = "off",
		/// 1/6.
		Minimal = "minimal",
		/// 2/6.
		Low = "low",
		/// 3/6.
		Medium = "medium",
		/// 4/6.
		High = "high",
		/// 5/6.
		Xhigh = "xhigh",
		/// Full, on fire.
		Max = "max",
	}
);

string_enum!(
	/// How a diff lays out.
	DiffMode {
		/// One column.
		Unified = "unified",
		/// Side by side.
		Split = "split",
		/// The user's setting.
		Auto = "auto",
	}
);

string_enum!(
	/// How a `kv` lays out.
	KvLayout {
		/// Aligned rows.
		Grid = "grid",
		/// One wrapping line.
		Inline = "inline",
	}
);

string_enum!(
	/// A text's line-length cap.
	Measure {
		/// 80 cells.
		Prose = "prose",
	}
);

string_enum!(
	/// A card variant.
	CardVariant {
		/// No ring, fill or rounding.
		Bare = "bare",
	}
);

string_enum!(
	/// A picker sheet's size.
	PickerSize {
		/// Medium.
		Md = "md",
		/// Large.
		Lg = "lg",
		/// The whole pane.
		Screen = "screen",
	}
);

string_enum!(
	/// A picker's row template.
	PickerLayout {
		/// Rows.
		Rows = "rows",
		/// Cards (detail on its own line).
		Cards = "cards",
		/// A node per row.
		Timeline = "timeline",
		/// Depth guides and chevrons.
		Tree = "tree",
	}
);

string_enum!(
	/// Where a picker's preview sits.
	PickerPreview {
		/// Beside the list.
		Side = "side",
		/// Under the list.
		Below = "below",
		/// No preview.
		None = "none",
	}
);

string_enum!(
	/// Which part of a picker the keys drive.
	PickerFocus {
		/// The list.
		List = "list",
		/// The scope column.
		Scopes = "scopes",
		/// The tab row.
		Tabs = "tabs",
		/// The chip strip.
		Strip = "strip",
		/// The preview.
		Preview = "preview",
	}
);

string_enum!(
	/// A picker's state.
	PickerState {
		/// Rows shown.
		Ready = "ready",
		/// Skeleton rows.
		Loading = "loading",
		/// An error card.
		Error = "error",
	}
);

string_enum!(
	/// How a tool's string target is drawn.
	TargetKind {
		/// A shell line.
		Command = "command",
		/// A file path.
		Path = "path",
		/// A pattern in quotes.
		Pattern = "pattern",
		/// A query in quotes.
		Query = "query",
		/// Text.
		Text = "text",
	}
);

string_enum!(
	/// A tool's frame.
	ToolFrame {
		/// A ring.
		Card = "card",
		/// No ring.
		Inline = "inline",
	}
);

string_enum!(
	/// A checklist's presentation.
	ChecklistMode {
		/// The whole list.
		Full = "full",
		/// A pill.
		Hud = "hud",
		/// A reminder line.
		Reminder = "reminder",
	}
);

string_enum!(
	/// A checklist item's state.
	ItemStatus {
		/// Not started.
		Pending = "pending",
		/// In progress.
		Active = "active",
		/// Finished.
		Done = "done",
		/// Dropped.
		Dropped = "dropped",
		/// Blocked.
		Blocked = "blocked",
	}
);

string_enum!(
	/// An overlay's size.
	OverlaySize {
		/// Small.
		Sm = "sm",
		/// Medium.
		Md = "md",
		/// Large.
		Lg = "lg",
		/// A full sheet.
		Full = "full",
	}
);

string_enum!(
	/// Which group of a status strip a segment sits in.
	Side {
		/// The right group.
		Right = "right",
	}
);

string_enum!(
	/// A picker column's format.
	ColumnFormat {
		/// Text.
		Text = "text",
		/// A number.
		Num = "num",
		/// A price.
		Price = "price",
		/// A meter.
		Bar = "bar",
		/// A time.
		Time = "time",
		/// A duration.
		Elapsed = "elapsed",
		/// Dim text.
		Dim = "dim",
	}
);

string_enum!(
	/// How a `prefs` choice draws.
	ChoiceStyle {
		/// Segments when few and short, else a menu.
		Auto = "auto",
		/// Segments.
		Segmented = "segmented",
		/// A popup menu.
		Menu = "menu",
	}
);

string_enum!(
	/// The type of an `el` `input`.
	InputType {
		/// A checkbox.
		Checkbox = "checkbox",
		/// A radio.
		Radio = "radio",
	}
);

/// One styled run of text: `{t, s?, fx?, href?}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Span {
	/// The text.
	pub t:    String,
	/// Space-separated style tokens.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub s:    Option<String>,
	/// An effect.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub fx:   Option<Fx>,
	/// A link.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub href: Option<String>,
}

impl Span {
	/// A plain span.
	pub fn new(t: impl Into<String>) -> Self {
		Self { t: t.into(), ..Self::default() }
	}

	/// Adds a style token (or several, space-separated).
	pub fn style(mut self, token: impl Into<Token>) -> Self {
		let token = token.into();
		self.s = Some(match self.s.take() {
			Some(s) if !s.is_empty() => format!("{s} {token}"),
			_ => token.to_string(),
		});
		self
	}

	/// Sets the effect.
	pub fn fx(mut self, fx: impl Into<Fx>) -> Self {
		self.fx = Some(fx.into());
		self
	}

	/// Makes the span a link.
	pub fn href(mut self, href: impl Into<String>) -> Self {
		self.href = Some(href.into());
		self
	}
}

impl From<&str> for Span {
	fn from(t: &str) -> Self {
		Self::new(t)
	}
}

impl From<String> for Span {
	fn from(t: String) -> Self {
		Self::new(t)
	}
}

/// Text given as a plain string or as styled spans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Text {
	/// A plain string.
	Plain(String),
	/// Styled spans.
	Spans(Vec<Span>),
}

impl Text {
	/// The text without styles.
	pub fn plain(&self) -> String {
		match self {
			Self::Plain(s) => s.clone(),
			Self::Spans(spans) => spans.iter().map(|s| s.t.as_str()).collect(),
		}
	}
}

impl Default for Text {
	fn default() -> Self {
		Self::Plain(String::new())
	}
}

impl Serialize for Text {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Plain(t) => s.serialize_str(t),
			Self::Spans(spans) => spans.serialize(s),
		}
	}
}

impl From<&str> for Text {
	fn from(t: &str) -> Self {
		Self::Plain(t.to_owned())
	}
}

impl From<String> for Text {
	fn from(t: String) -> Self {
		Self::Plain(t)
	}
}

impl From<&String> for Text {
	fn from(t: &String) -> Self {
		Self::Plain(t.clone())
	}
}

impl From<Span> for Text {
	fn from(span: Span) -> Self {
		Self::Spans(vec![span])
	}
}

impl From<Vec<Span>> for Text {
	fn from(spans: Vec<Span>) -> Self {
		Self::Spans(spans)
	}
}

impl<const N: usize> From<[Span; N]> for Text {
	fn from(spans: [Span; N]) -> Self {
		Self::Spans(spans.into())
	}
}

/// A size bound: `"<n>ch"`, `"<n>lines"` or a fraction of the parent.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Extent {
	/// Character cells.
	Ch(f64),
	/// Text lines.
	Lines(f64),
	/// A fraction of the parent (`0.5` is half).
	Fraction(f64),
}

impl Serialize for Extent {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match *self {
			Self::Ch(n) => s.collect_str(&format_args!("{n}ch")),
			Self::Lines(n) => s.collect_str(&format_args!("{n}lines")),
			Self::Fraction(f) => s.serialize_f64(f),
		}
	}
}

/// Size bounds `{w?, h?}` for `min` and `max`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Bound {
	/// The width bound.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub w: Option<Extent>,
	/// The height bound.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub h: Option<Extent>,
}

impl Bound {
	/// A width bound.
	pub const fn w(w: Extent) -> Self {
		Self { w: Some(w), h: None }
	}

	/// A height bound.
	pub const fn h(h: Extent) -> Self {
		Self { w: None, h: Some(h) }
	}

	/// Both bounds.
	pub const fn wh(w: Extent, h: Extent) -> Self {
		Self { w: Some(w), h: Some(h) }
	}
}

/// A flex basis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Basis {
	/// A fraction of the parent.
	Fraction(f64),
	/// The node's own size, without shrinking.
	Content,
}

impl Serialize for Basis {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match *self {
			Self::Fraction(f) => s.serialize_f64(f),
			Self::Content => s.serialize_str("content"),
		}
	}
}

impl From<f64> for Basis {
	fn from(f: f64) -> Self {
		Self::Fraction(f)
	}
}

/// A node key: a string, or an integer named in decimal.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeKey {
	/// A string key.
	Str(String),
	/// An integer key.
	Int(i64),
}

impl Serialize for NodeKey {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Str(k) => s.serialize_str(k),
			Self::Int(n) => s.serialize_i64(*n),
		}
	}
}

impl From<&str> for NodeKey {
	fn from(k: &str) -> Self {
		Self::Str(k.to_owned())
	}
}

impl From<String> for NodeKey {
	fn from(k: String) -> Self {
		Self::Str(k)
	}
}

impl From<&String> for NodeKey {
	fn from(k: &String) -> Self {
		Self::Str(k.clone())
	}
}

/// Implements `From<integer>` for [`NodeKey`].
macro_rules! int_key {
	($($t:ty),*) => {
		$(impl From<$t> for NodeKey {
			fn from(n: $t) -> Self {
				Self::Int(n as i64)
			}
		})*
	};
}

int_key!(i32, i64, u32, u64, usize);

/// What a pointer does on a node: `{click?, dblclick?, menu?}`.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Actions {
	/// The click action.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub click:    Option<String>,
	/// The double-click action.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dblclick: Option<String>,
	/// Context-menu actions.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub menu:     Vec<String>,
}

impl Actions {
	/// No actions.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the click action.
	pub fn click(mut self, act: impl Into<String>) -> Self {
		self.click = Some(act.into());
		self
	}

	/// Sets the double-click action.
	pub fn dblclick(mut self, act: impl Into<String>) -> Self {
		self.dblclick = Some(act.into());
		self
	}

	/// Adds a context-menu action.
	pub fn menu(mut self, act: impl Into<String>) -> Self {
		self.menu.push(act.into());
		self
	}
}
