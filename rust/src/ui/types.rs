//! Typed prop values: styled text, enumerations (each with an escape hatch
//! for values newer than the SDK), sizes and actions.

use serde::{Serialize, Serializer};

use super::data::MenuEntry;

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
		/// A tool waiting for approval: Tern draws its `confirm` panel.
		Ask = "ask",
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
		/// The `pending` tone's color: waiting on someone.
		Pending = "pending",
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
		/// A page's title in its head row (Geist 15px, weight 550).
		Title = "title",
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
		/// A line over a value domain, one per series.
		Line = "line",
		/// A line with its area filled.
		Area = "area",
		/// A ring of segments with the total in its middle.
		Donut = "donut",
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
	/// A `button`'s chrome.
	ButtonVariant {
		/// A quiet chip with a hairline ring (the default).
		Default = "default",
		/// The one filled action of a view.
		Primary = "primary",
		/// No fill or ring until hovered.
		Quiet = "quiet",
		/// A destructive action, tinted `--bad`.
		Danger = "danger",
	}
);

string_enum!(
	/// A `button`'s height.
	ButtonSize {
		/// 22px: inline, in rows.
		Sm = "sm",
		/// 28px: bars, heads, forms (the default).
		Md = "md",
	}
);

string_enum!(
	/// A `tabs` strip's look.
	TabsVariant {
		/// An underline under the active tab (the default).
		Underline = "underline",
		/// A segmented well, sized to its tabs.
		Segmented = "segmented",
	}
);

/// Whether something is disabled: `true`, or a reason shown as its
/// tooltip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Disabled {
	/// Enabled or disabled, without a reason.
	Flag(bool),
	/// Disabled, with the reason as tooltip.
	Reason(String),
}

impl Serialize for Disabled {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Flag(b) => s.serialize_bool(*b),
			Self::Reason(r) => s.serialize_str(r),
		}
	}
}

impl From<bool> for Disabled {
	fn from(b: bool) -> Self {
		Self::Flag(b)
	}
}

impl From<&str> for Disabled {
	fn from(r: &str) -> Self {
		Self::Reason(r.to_owned())
	}
}

impl From<String> for Disabled {
	fn from(r: String) -> Self {
		Self::Reason(r)
	}
}

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

string_enum!(
	/// An `input`'s or `editor`'s look.
	InputVariant {
		/// The tinted one-line well (the default without addons).
		Well = "well",
		/// Kit's framed field (the default with an addon).
		Field = "field",
	}
);

string_enum!(
	/// The edge an overlay docks to as a sheet or drawer.
	OverlaySide {
		/// A full-height sheet at the right.
		Right = "right",
		/// A full-height sheet at the left.
		Left = "left",
		/// A drawer at the bottom.
		Bottom = "bottom",
	}
);

string_enum!(
	/// How a docked overlay sits at its edge.
	OverlayVariant {
		/// Flush with the edge (the default).
		Edge = "edge",
		/// Floating 8px off it with rounded corners.
		Inset = "inset",
	}
);

string_enum!(
	/// A `rule`'s look.
	RuleVariant {
		/// A hairline with the label in it (the default).
		Line = "line",
		/// A framed row: icon, label, time at the right.
		Note = "note",
		/// A pill on the hairline.
		Pin = "pin",
	}
);

string_enum!(
	/// How cited spans of a `text` look.
	CiteStyle {
		/// A superscript number (the default).
		Chip = "chip",
		/// A pill naming the source's site.
		Pill = "pill",
		/// A number, with the numbered sources under the text.
		Note = "note",
	}
);

string_enum!(
	/// A sort direction.
	SortDir {
		/// Ascending.
		Asc = "asc",
		/// Descending.
		Desc = "desc",
	}
);

string_enum!(
	/// How a `multi` table's picks and sort look.
	TableVariant {
		/// A checkbox column (the default).
		Check = "check",
		/// An accent bar on picked rows, the sorted header in a pill.
		Bar = "bar",
	}
);

string_enum!(
	/// The approval look of a tool whose status is `ask`.
	ToolConfirm {
		/// A warning-toned frame with Deny, Always and Allow (the default).
		Card = "card",
		/// A compact strip under the head.
		Strip = "strip",
		/// A radio list of scopes, then Deny and Allow.
		Scopes = "scopes",
	}
);

string_enum!(
	/// A tool's resolved approval, shown as a head chip.
	Approved {
		/// Allowed once.
		Once = "once",
		/// Allowed this session.
		Session = "session",
		/// Always allowed.
		Always = "always",
		/// Denied.
		Denied = "denied",
	}
);

string_enum!(
	/// A `select`'s look.
	SelectVariant {
		/// A framed trigger (the default).
		Field = "field",
		/// No fill or ring, the value after a muted label.
		Quiet = "quiet",
	}
);

string_enum!(
	/// A `switch`'s look.
	SwitchVariant {
		/// A 36×22 switch, a labeled one filling its row (the default).
		Default = "default",
		/// A 26×15 switch with its label to the right.
		Compact = "compact",
		/// An Off | On segmented control.
		Segment = "segment",
	}
);

string_enum!(
	/// A `slider`'s look.
	SliderVariant {
		/// One row: label, track, value (the default).
		Default = "default",
		/// The label and value over an accent track.
		Accent = "accent",
	}
);

string_enum!(
	/// How a `slider`'s value reads.
	SliderFormat {
		/// The value with the step's decimals, then `unit` (the default).
		Number = "number",
		/// A fraction as a percentage.
		Percent = "percent",
		/// Milliseconds as `200 ms`, `1.2 s`, `1 min 30 s`.
		Duration = "duration",
	}
);

string_enum!(
	/// A `field`'s layout.
	FieldVariant {
		/// The label over the control (the default).
		Stacked = "stacked",
		/// The label and note left, the control right.
		Row = "row",
		/// A right-aligned label column `labelWidth` wide.
		Grid = "grid",
	}
);

string_enum!(
	/// An `otp`'s look.
	OtpVariant {
		/// One cell per character (the default).
		Cells = "cells",
		/// Each group one field split into cells.
		Joined = "joined",
		/// Digits over rules, no boxes.
		Underline = "underline",
	}
);

string_enum!(
	/// An `otp`'s state.
	OtpState {
		/// Ringed in `--bad`, shaken once.
		Error = "error",
		/// Ringed in `--ok`.
		Success = "success",
	}
);

string_enum!(
	/// A `buttons` group's look.
	ButtonsVariant {
		/// Segments sharing one ring (the default).
		Joined = "joined",
		/// `joined` for an action and its menu part.
		Split = "split",
		/// A soft well holding bare buttons.
		Toolbar = "toolbar",
	}
);

string_enum!(
	/// An avatar's shape.
	AvatarVariant {
		/// A circle (the default): people.
		Circle = "circle",
		/// A tinted squircle: bots, orgs, repositories.
		Square = "square",
	}
);

string_enum!(
	/// An avatar's size.
	AvatarSize {
		/// 16px.
		Xs = "xs",
		/// 20px.
		Sm = "sm",
		/// 28px (the default).
		Md = "md",
		/// 36px.
		Lg = "lg",
	}
);

string_enum!(
	/// A `breadcrumb`'s look.
	BreadcrumbVariant {
		/// Crumbs between chevrons (the default).
		Chevron = "chevron",
		/// A path joined by slashes.
		Path = "path",
	}
);

string_enum!(
	/// A `pager`'s look.
	PagerVariant {
		/// "12–24 of 312" and arrows in one capsule (the default).
		Range = "range",
		/// Page numbers in a seven-slot window.
		Pages = "pages",
		/// A centered "Show 50 more" row.
		More = "more",
	}
);

string_enum!(
	/// An `empty` state's look.
	EmptyVariant {
		/// Icon, title, line and actions, centered (the default).
		Center = "center",
		/// One row: icon, title, muted text, actions at the end.
		Inline = "inline",
		/// A dashed zone that takes a dropped file.
		Drop = "drop",
	}
);

string_enum!(
	/// A `skeleton`'s look.
	SkeletonVariant {
		/// One band sweeping across the blocks (the default).
		Shimmer = "shimmer",
		/// Blocks fading in and out slowly.
		Pulse = "pulse",
		/// Dashed outlines, no fill, still.
		Wire = "wire",
	}
);

string_enum!(
	/// What a `skeleton` stands for.
	SkeletonShape {
		/// Lines of text (the default).
		Text = "text",
		/// An avatar beside a name.
		Avatar = "avatar",
		/// List rows.
		Rows = "rows",
		/// A card of stat tiles.
		Card = "card",
		/// A block.
		Block = "block",
	}
);

string_enum!(
	/// A `calendar`'s look.
	CalendarVariant {
		/// A month grid (the default).
		Month = "month",
		/// Presets beside a week strip.
		Week = "week",
	}
);

string_enum!(
	/// The first column of a month grid.
	WeekStart {
		/// Monday (the default).
		Mon = "mon",
		/// Sunday.
		Sun = "sun",
	}
);

string_enum!(
	/// A `menu`'s look.
	MenuVariant {
		/// Kit's menu (the default).
		Default = "default",
		/// 24px rows for long sort and filter menus.
		Dense = "dense",
	}
);

string_enum!(
	/// Which edge of its trigger a menu lines up with.
	MenuAlign {
		/// The start (the default).
		Start = "start",
		/// The end.
		End = "end",
	}
);

string_enum!(
	/// A `menubar`'s look.
	MenubarVariant {
		/// A full-width hairline strip (the default).
		Strip = "strip",
		/// A frosted capsule as wide as its titles.
		Pill = "pill",
	}
);

string_enum!(
	/// A `nav`'s look.
	NavVariant {
		/// Pills in a well, sub-items in a menu (the default).
		Pills = "pills",
		/// Text links over a hairline, sub-items in a panel.
		Links = "links",
	}
);

string_enum!(
	/// A `split`'s look.
	SplitVariant {
		/// One framed box, a 1px divider (the default).
		Hairline = "hairline",
		/// Every pane its own card, a dotted grip in the gap.
		Gutter = "gutter",
		/// Every pane its own card; the gap is the handle.
		Tiles = "tiles",
	}
);

string_enum!(
	/// How a `split` lays out its panes.
	SplitDir {
		/// Side by side (the default).
		Row = "row",
		/// Stacked.
		Col = "col",
	}
);

string_enum!(
	/// A `scroll`'s look.
	ScrollVariant {
		/// An overlay thumb while scrolling (the default).
		Bar = "bar",
		/// The bar, plus faded edges and a "N more" chip.
		Fade = "fade",
	}
);

string_enum!(
	/// Where a `scroll` keeps to while its content grows.
	Stick {
		/// The end, until the user scrolls up from it.
		End = "end",
	}
);

string_enum!(
	/// When a `scroll`'s thumb shows.
	ScrollBar {
		/// While scrolling (the default).
		Auto = "auto",
		/// Always, while there is something to scroll.
		Always = "always",
	}
);

string_enum!(
	/// A `carousel`'s look.
	CarouselVariant {
		/// The next slide peeking in (the default).
		Peek = "peek",
		/// Round arrow buttons over the slide's edges.
		Arrows = "arrows",
		/// A row of thumbnails under the slide.
		Strip = "strip",
	}
);

string_enum!(
	/// A `message`'s look.
	MessageVariant {
		/// An accent bubble on the end side, plain text on the start side
		/// (the default).
		Auto = "auto",
		/// Bubbles on both sides.
		Bubble = "bubble",
		/// Rows from the left under avatar, name and time.
		Flat = "flat",
	}
);

string_enum!(
	/// Which side a `message` sits on.
	MessageSide {
		/// Others (the default).
		Start = "start",
		/// The local user.
		End = "end",
	}
);

string_enum!(
	/// When a `message`'s action buttons show.
	Reveal {
		/// On hover (the default).
		Hover = "hover",
		/// Always.
		Always = "always",
	}
);

string_enum!(
	/// An `attachment`'s look.
	AttachmentVariant {
		/// Type tile, name, meta line, action icons (the default).
		Chip = "chip",
		/// A thumbnail, the name under it.
		Tile = "tile",
		/// A capsule: icon, mono name, ×.
		Pill = "pill",
	}
);

string_enum!(
	/// An `attachment`'s upload state.
	AttachmentState {
		/// At rest (the default).
		Idle = "idle",
		/// Uploading.
		Uploading = "uploading",
		/// Processing.
		Processing = "processing",
		/// Failed.
		Error = "error",
		/// Done.
		Done = "done",
	}
);

string_enum!(
	/// A `trace`'s look.
	TraceVariant {
		/// Lines on a hairline rail (the default).
		Rail = "rail",
		/// One summary line that unfolds into the lines.
		Summary = "summary",
		/// A bare card: a head, a prose body, source chips.
		Card = "card",
	}
);

string_enum!(
	/// A `stacktrace`'s look.
	StacktraceVariant {
		/// An error card over the app frames (the default).
		Frames = "frames",
		/// A frame list beside the selected frame's code.
		Split = "split",
		/// A mono well of `at fn (file:line:col)` lines.
		Mono = "mono",
	}
);

string_enum!(
	/// A `suggestions`'s look.
	SuggestionsVariant {
		/// Wrapping pills (the default).
		Chips = "chips",
		/// Full-width rows with keycaps.
		Rows = "rows",
		/// Two columns of cards.
		Cards = "cards",
	}
);

string_enum!(
	/// A `questions`'s look.
	QuestionsVariant {
		/// One card per question, in steps (the default).
		Steps = "steps",
		/// Prompt, keyed chips and a hint, in the flow.
		Inline = "inline",
		/// Every question; answered ones fold.
		List = "list",
	}
);

/// One styled run of text: `{t, s?, fx?, href?, cite?}`.
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
	/// The sources this run cites; the marker follows the run (an empty `t`
	/// draws the marker alone).
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub cite: Vec<Cite>,
}

/// A source a span cites.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Cite {
	/// Where the source is; opened on click. A cite without one is ignored.
	pub href:    String,
	/// The source's title.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:   Option<String>,
	/// The number shown (default: the next free one, by first appearance).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub n:       Option<u32>,
	/// The name the pill shows (default: the `href`'s host, `www.` dropped).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub site:    Option<String>,
	/// A quote shown in the hover card.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub snippet: Option<String>,
}

impl Cite {
	/// A source at `href`.
	pub fn new(href: impl Into<String>) -> Self {
		Self { href: href.into(), ..Self::default() }
	}

	/// Sets the title.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Sets the number shown.
	pub const fn n(mut self, n: u32) -> Self {
		self.n = Some(n);
		self
	}

	/// Sets the site name.
	pub fn site(mut self, site: impl Into<String>) -> Self {
		self.site = Some(site.into());
		self
	}

	/// Sets the quote.
	pub fn snippet(mut self, snippet: impl Into<String>) -> Self {
		self.snippet = Some(snippet.into());
		self
	}
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

	/// Cites a source after the run.
	pub fn cite(mut self, cite: Cite) -> Self {
		self.cite.push(cite);
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

/// A height in pixels or in the surface's cells and lines.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Height {
	/// Pixels.
	Px(f64),
	/// Character cells.
	Ch(f64),
	/// Text lines.
	Lines(f64),
}

impl Serialize for Height {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match *self {
			Self::Px(px) => s.serialize_f64(px),
			Self::Ch(n) => s.collect_str(&format_args!("{n}ch")),
			Self::Lines(n) => s.collect_str(&format_args!("{n}lines")),
		}
	}
}

/// The minimum size of a `split`'s panes along the split.
#[derive(Clone, Debug, PartialEq)]
pub enum PaneMin {
	/// The same minimum for every pane.
	All(Extent),
	/// A minimum per pane, in child order (`None` for none).
	Each(Vec<Option<Extent>>),
	/// The common size bound of the split itself.
	Bound(Bound),
}

impl Serialize for PaneMin {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::All(extent) => extent.serialize(s),
			Self::Each(extents) => extents.serialize(s),
			Self::Bound(bound) => bound.serialize(s),
		}
	}
}

impl From<Extent> for PaneMin {
	fn from(extent: Extent) -> Self {
		Self::All(extent)
	}
}

impl From<Vec<Option<Extent>>> for PaneMin {
	fn from(extents: Vec<Option<Extent>>) -> Self {
		Self::Each(extents)
	}
}

impl From<Bound> for PaneMin {
	fn from(bound: Bound) -> Self {
		Self::Bound(bound)
	}
}

/// Text, or a bare flag: a `field`'s `error`/`warning` (`true` sets the
/// state without a line), a chart's `last` (`true` prints the value,
/// `false` hides it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TextFlag {
	/// On or off, without text.
	Flag(bool),
	/// The text.
	Text(Text),
}

impl Serialize for TextFlag {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Flag(b) => s.serialize_bool(*b),
			Self::Text(t) => t.serialize(s),
		}
	}
}

impl From<bool> for TextFlag {
	fn from(b: bool) -> Self {
		Self::Flag(b)
	}
}

impl<T: Into<Text>> From<T> for TextFlag {
	fn from(t: T) -> Self {
		Self::Text(t.into())
	}
}

/// A `slider`'s value: one, or a range drawn with two thumbs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SliderValue {
	/// One value.
	One(f64),
	/// A range, low to high.
	Range(f64, f64),
}

impl Serialize for SliderValue {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match *self {
			Self::One(v) => s.serialize_f64(v),
			Self::Range(lo, hi) => [lo, hi].serialize(s),
		}
	}
}

impl From<f64> for SliderValue {
	fn from(v: f64) -> Self {
		Self::One(v)
	}
}

impl From<(f64, f64)> for SliderValue {
	fn from((lo, hi): (f64, f64)) -> Self {
		Self::Range(lo, hi)
	}
}

/// A `slider`'s tick marks.
#[derive(Clone, Debug, PartialEq)]
pub enum Ticks {
	/// One at every step (`true`), or none (`false`).
	Steps(bool),
	/// The track divided into this many equal parts.
	Parts(u32),
	/// One at each of these values.
	At(Vec<f64>),
}

impl Serialize for Ticks {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Steps(b) => s.serialize_bool(*b),
			Self::Parts(n) => s.serialize_u32(*n),
			Self::At(values) => values.serialize(s),
		}
	}
}

impl From<bool> for Ticks {
	fn from(b: bool) -> Self {
		Self::Steps(b)
	}
}

impl From<u32> for Ticks {
	fn from(n: u32) -> Self {
		Self::Parts(n)
	}
}

impl From<Vec<f64>> for Ticks {
	fn from(values: Vec<f64>) -> Self {
		Self::At(values)
	}
}

/// How an `otp` hides what was typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OtpMask {
	/// A dot in each filled cell (`true`), or shown (`false`).
	Dots(bool),
	/// This character in each filled cell.
	Char(String),
}

impl Serialize for OtpMask {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Dots(b) => s.serialize_bool(*b),
			Self::Char(c) => s.serialize_str(c),
		}
	}
}

impl From<bool> for OtpMask {
	fn from(b: bool) -> Self {
		Self::Dots(b)
	}
}

impl From<&str> for OtpMask {
	fn from(c: &str) -> Self {
		Self::Char(c.to_owned())
	}
}

impl From<char> for OtpMask {
	fn from(c: char) -> Self {
		Self::Char(c.to_string())
	}
}

/// What a `scroll`'s counts name: one word made plural with an `s`, or
/// both forms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Noun {
	/// `"line"`: "4 more lines".
	Word(String),
	/// `["entry", "entries"]`.
	Forms(String, String),
}

impl Serialize for Noun {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Word(w) => s.serialize_str(w),
			Self::Forms(one, many) => [one, many].serialize(s),
		}
	}
}

impl From<&str> for Noun {
	fn from(w: &str) -> Self {
		Self::Word(w.to_owned())
	}
}

impl From<String> for Noun {
	fn from(w: String) -> Self {
		Self::Word(w)
	}
}

impl From<(&str, &str)> for Noun {
	fn from((one, many): (&str, &str)) -> Self {
		Self::Forms(one.to_owned(), many.to_owned())
	}
}

/// A `calendar`'s or `date`'s selection: a `YYYY-MM-DD` day, or a range
/// whose end may still be open.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateValue {
	/// One day.
	Day(String),
	/// From a day to another (`None`: waiting for its end).
	Range(String, Option<String>),
}

impl Serialize for DateValue {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		match self {
			Self::Day(day) => s.serialize_str(day),
			Self::Range(from, to) => (from, to).serialize(s),
		}
	}
}

impl From<&str> for DateValue {
	fn from(day: &str) -> Self {
		Self::Day(day.to_owned())
	}
}

impl From<String> for DateValue {
	fn from(day: String) -> Self {
		Self::Day(day)
	}
}

impl From<(&str, &str)> for DateValue {
	fn from((from, to): (&str, &str)) -> Self {
		Self::Range(from.to_owned(), Some(to.to_owned()))
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
	/// Context-menu entries: actions, items with icons, keys and checks,
	/// separators, headers and submenus.
	#[serde(skip_serializing_if = "Vec::is_empty")]
	pub menu:     Vec<MenuEntry>,
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

	/// Adds a context-menu entry: an action's name, or a [`MenuEntry`].
	pub fn menu(mut self, entry: impl Into<MenuEntry>) -> Self {
		self.menu.push(entry.into());
		self
	}
}
