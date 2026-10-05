//! Layer 1: the wire. Constants, the messages a program sends and receives,
//! the encoder (framing, chunking, blobs) and the decoder of replies and
//! events.
//!
//! Every message is one APC string, `ESC _ tsp;<verb>[;k=v]*;<body> ESC \`.
//! A body over the negotiated `apc` limit is cut by [`chunks`] exactly as
//! Tern's own `tsp_chunks` cuts it, and each piece is sent with `c=<id>`
//! (and `m=1` on all but the last).

use std::{collections::BTreeMap, fmt};

use base64::Engine as _;
use serde::{Deserialize, Serialize, Serializer, de::DeserializeOwned, ser::SerializeSeq};
use serde_json::{Map, Value};
use sha2::{Digest as _, Sha256};

use crate::Error;

/// The protocol version this SDK speaks.
pub const VERSION: u32 = 1;
/// The largest body sent in one message unless the `hello` reply says
/// otherwise.
pub const APC_LIMIT: usize = 65_536;
/// How many frames may be unacknowledged unless the `hello` reply says
/// otherwise.
pub const CREDITS: u32 = 2;
/// The largest blob Tern takes, in decoded bytes.
pub const MAX_BLOB: usize = 16 << 20;

/// Node kinds, as the `k` of a node and the `kinds` of the `hello` reply.
pub mod kind {
	/// A vertical stack.
	pub const COL: &str = "col";
	/// A horizontal flex row.
	pub const ROW: &str = "row";
	/// A card with a head and a body.
	pub const CARD: &str = "card";
	/// A disclosure group with no ring.
	pub const SECTION: &str = "section";
	/// A divider hairline.
	pub const RULE: &str = "rule";
	/// Vertical space.
	pub const SPACER: &str = "spacer";
	/// Styled spans.
	pub const TEXT: &str = "text";
	/// Markdown.
	pub const MD: &str = "md";
	/// Highlighted code.
	pub const CODE: &str = "code";
	/// A unified or split diff.
	pub const DIFF: &str = "diff";
	/// Output with escape sequences.
	pub const ANSI: &str = "ansi";
	/// Pre-rendered ANSI rows.
	pub const ROWS: &str = "rows";
	/// TeX math.
	pub const MATH: &str = "math";
	/// An aligned key/value list.
	pub const KV: &str = "kv";
	/// A table.
	pub const TABLE: &str = "table";
	/// A disclosure tree.
	pub const TREE: &str = "tree";
	/// A pill chip.
	pub const BADGE: &str = "badge";
	/// Keycaps.
	pub const KBD: &str = "kbd";
	/// A named icon.
	pub const ICON: &str = "icon";
	/// An image from a blob.
	pub const IMAGE: &str = "image";
	/// A selectable list.
	pub const LIST: &str = "list";
	/// One list row.
	pub const ITEM: &str = "item";
	/// A tab strip.
	pub const TABS: &str = "tabs";
	/// A searchable picker sheet.
	pub const PICKER: &str = "picker";
	/// An activity indicator.
	pub const SPINNER: &str = "spinner";
	/// A shimmering label.
	pub const SHIMMER: &str = "shimmer";
	/// A live timer.
	pub const ELAPSED: &str = "elapsed";
	/// A number easing between updates.
	pub const RATE: &str = "rate";
	/// A progress bar.
	pub const PROGRESS: &str = "progress";
	/// A value as a bar, ring or grid.
	pub const METER: &str = "meter";
	/// Bars, a sparkline or a heatmap.
	pub const CHART: &str = "chart";
	/// A thinking-effort glyph.
	pub const EFFORT: &str = "effort";
	/// A multi-line text field.
	pub const EDITOR: &str = "editor";
	/// A single-line text field.
	pub const INPUT: &str = "input";
	/// A status bar strip.
	pub const STATUS: &str = "status";
	/// A status bar segment.
	pub const SEG: &str = "seg";
	/// A transient notice.
	pub const TOAST: &str = "toast";
	/// A floating panel.
	pub const OVERLAY: &str = "overlay";
	/// A coding-agent tool call.
	pub const TOOL: &str = "tool";
	/// A subagent row.
	pub const AGENT: &str = "agent";
	/// A todo list.
	pub const CHECKLIST: &str = "checklist";
	/// A shell command's output (Tern's own).
	pub const BLOCK: &str = "block";
	/// A settings page.
	pub const PREFS: &str = "prefs";
	/// A plain HTML element.
	pub const EL: &str = "el";

	/// Every kind of protocol version 1.
	pub const ALL: [&str; 44] = [
		COL, ROW, CARD, SECTION, RULE, SPACER, TEXT, MD, CODE, DIFF, ANSI, ROWS, MATH, KV, TABLE,
		TREE, BADGE, KBD, ICON, IMAGE, LIST, ITEM, TABS, PICKER, SPINNER, SHIMMER, ELAPSED, RATE,
		PROGRESS, METER, CHART, EFFORT, EDITOR, INPUT, STATUS, SEG, TOAST, OVERLAY, TOOL, AGENT,
		CHECKLIST, BLOCK, PREFS, EL,
	];

	/// The kinds whose `text` prop is primary text, updated with `text` and
	/// `splice` ops.
	pub const TEXT_KINDS: [&str; 9] = [TEXT, MD, CODE, ANSI, MATH, EDITOR, INPUT, SHIMMER, EL];

	/// Whether `kind` keeps primary text in its `text` prop.
	pub fn has_text(kind: &str) -> bool {
		TEXT_KINDS.contains(&kind)
	}
}

/// Feature names of the `hello` query (program) and reply (terminal).
pub mod feature {
	/// Program: applies `edit` events (native editing).
	pub const EDIT: &str = "edit";
	/// Program: applies `undo` events.
	pub const UNDO: &str = "undo";
	/// Program: submits text from `send` events.
	pub const SEND: &str = "send";

	/// Terminal: the `b` verb and the `blobs` query.
	pub const BLOBS: &str = "blobs";
	/// Terminal: the `settle` op is a useful hint.
	pub const SETTLE: &str = "settle";
	/// Terminal: `o` with `adopt:true` reopens a closed surface.
	pub const ADOPT: &str = "adopt";
	/// Terminal: the `dock` region sticks to the pane's bottom.
	pub const DOCK: &str = "dock";
	/// Terminal: the `t` verb colors the surface.
	pub const PROGRAM_PALETTE: &str = "program-palette";
	/// Terminal: `reduceMotion` and `motion` events are reported.
	pub const REDUCE_MOTION: &str = "reduce-motion";
	/// Terminal: a `prefs` node in an inline surface's `layer` docks beside
	/// the pane.
	pub const ASIDE: &str = "aside";
	/// Terminal: the `scroll` op.
	pub const SCROLL: &str = "scroll";
	/// Terminal: the `s` verb (stylesheets).
	pub const STYLES: &str = "styles";
	/// Terminal: `flow` surfaces and `listen:false`.
	pub const FLOW: &str = "flow";
}

// ---------------------------------------------------------------------------
// Program → terminal
// ---------------------------------------------------------------------------

/// A surface mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
	/// At the cursor row, owning the pane while live (a session program).
	#[default]
	Inline,
	/// Over the whole pane, like the alternate screen.
	Screen,
	/// At the cursor row among the output around it, kept as output.
	Flow,
}

/// A `q` query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
	/// `{"q":"hello","v":[1],"app"?,"ver"?,"features"?}`.
	Hello {
		/// The program's name.
		app:      Option<String>,
		/// The program's version.
		ver:      Option<String>,
		/// Program features (`edit`, `undo`, `send`).
		features: Vec<String>,
	},
	/// `{"q":"blobs","ids":[…]}`: which blobs Tern still holds.
	Blobs {
		/// Blob ids (lowercase SHA-256 hex).
		ids: Vec<String>,
	},
}

impl Serialize for Query {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		use serde::ser::SerializeMap as _;
		match self {
			Self::Hello { app, ver, features } => {
				let mut m = s.serialize_map(None)?;
				m.serialize_entry("q", "hello")?;
				m.serialize_entry("v", &[VERSION])?;
				if let Some(app) = app {
					m.serialize_entry("app", app)?;
				}
				if let Some(ver) = ver {
					m.serialize_entry("ver", ver)?;
				}
				if !features.is_empty() {
					m.serialize_entry("features", features)?;
				}
				m.end()
			},
			Self::Blobs { ids } => {
				let mut m = s.serialize_map(Some(2))?;
				m.serialize_entry("q", "blobs")?;
				m.serialize_entry("ids", ids)?;
				m.end()
			},
		}
	}
}

/// An `o` message: open a surface.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Open {
	/// The surface id; frames and events carry it as `sf`.
	pub id:     String,
	/// The mode (`inline` when absent).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub mode:   Option<Mode>,
	/// Names the pane until the program sets its own title.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub title:  Option<String>,
	/// `data-surface` on the region elements, for stylesheets.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub role:   Option<String>,
	/// `false`: the program never reads its input.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub listen: Option<bool>,
	/// `true` reopens a closed inline surface with this id.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub adopt:  Option<bool>,
}

/// An `f` message: an atomic batch of ops on one surface.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Frame {
	/// The surface id.
	pub sf:  String,
	/// The frame's sequence number, from 1 per surface.
	pub s:   u64,
	/// The ops, applied in order.
	pub ops: Vec<Op>,
}

/// How a `text` op changes primary text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextMode {
	/// Add to the end (the streaming path).
	Append,
	/// Set the whole text.
	Replace,
}

/// Where a `reveal` op scrolls a node.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RevealAt {
	/// The node's start at the scroller's start.
	Start,
	/// The node's end at the scroller's end.
	End,
	/// The least scrolling that shows the node.
	#[default]
	Nearest,
}

/// How far a `scroll` op moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ScrollBy {
	/// One line up.
	LineUp,
	/// One line down.
	LineDown,
	/// A viewport less a line up.
	PageUp,
	/// A viewport less a line down.
	PageDown,
	/// To the start.
	Start,
	/// To the end (resumes following).
	End,
}

/// A node with its id, as an `add` op carries it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WireNode {
	/// The node id.
	pub id: String,
	/// The kind.
	pub k:  String,
	/// Props (omitted when empty).
	#[serde(default, skip_serializing_if = "Map::is_empty")]
	pub p:  Map<String, Value>,
	/// Children (omitted when empty).
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub c:  Vec<Self>,
}

/// One frame op.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
	/// `["add", id, parent, before|null, node]`.
	Add {
		/// Parent id.
		parent: String,
		/// The sibling to insert before, or last.
		before: Option<String>,
		/// The subtree; its id is the op's.
		node:   WireNode,
	},
	/// `["set", id, props]`: merge props shallowly; `null` deletes.
	Set {
		/// Node id.
		id:    String,
		/// Props to replace.
		props: Map<String, Value>,
	},
	/// `["text", id, mode, string]`.
	Text {
		/// Node id.
		id:   String,
		/// Append or replace.
		mode: TextMode,
		/// The text.
		text: String,
	},
	/// `["splice", id, at, del, string]` in UTF-16 units.
	Splice {
		/// Node id.
		id:   String,
		/// Offset in UTF-16 units.
		at:   usize,
		/// UTF-16 units removed.
		del:  usize,
		/// Text inserted.
		text: String,
	},
	/// `["move", id, parent, before|null]`.
	Move {
		/// Node id.
		id:     String,
		/// New parent id.
		parent: String,
		/// The sibling to move before, or last.
		before: Option<String>,
	},
	/// `["del", id]`.
	Del {
		/// Node id.
		id: String,
	},
	/// `["settle", id]`: a hint that the subtree won't change soon.
	Settle {
		/// Node id.
		id: String,
	},
	/// `["focus", id|null]`: the field owning the caret.
	Focus {
		/// Field id, or none.
		id: Option<String>,
	},
	/// `["reveal", id, at]`.
	Reveal {
		/// Node id.
		id: String,
		/// Where to scroll it.
		at: RevealAt,
	},
	/// `["scroll", id, by]`.
	Scroll {
		/// A node in the scroll container.
		id: String,
		/// How far.
		by: ScrollBy,
	},
	/// `["suspend"]`: hand the pane back to the grid.
	Suspend,
	/// `["resume"]`: take the pane again.
	Resume,
}

impl Serialize for Op {
	fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
		let mut seq = s.serialize_seq(None)?;
		match self {
			Self::Add { parent, before, node } => {
				seq.serialize_element("add")?;
				seq.serialize_element(&node.id)?;
				seq.serialize_element(parent)?;
				seq.serialize_element(before)?;
				seq.serialize_element(node)?;
			},
			Self::Set { id, props } => {
				seq.serialize_element("set")?;
				seq.serialize_element(id)?;
				seq.serialize_element(props)?;
			},
			Self::Text { id, mode, text } => {
				seq.serialize_element("text")?;
				seq.serialize_element(id)?;
				seq.serialize_element(mode)?;
				seq.serialize_element(text)?;
			},
			Self::Splice { id, at, del, text } => {
				seq.serialize_element("splice")?;
				seq.serialize_element(id)?;
				seq.serialize_element(at)?;
				seq.serialize_element(del)?;
				seq.serialize_element(text)?;
			},
			Self::Move { id, parent, before } => {
				seq.serialize_element("move")?;
				seq.serialize_element(id)?;
				seq.serialize_element(parent)?;
				seq.serialize_element(before)?;
			},
			Self::Del { id } => {
				seq.serialize_element("del")?;
				seq.serialize_element(id)?;
			},
			Self::Settle { id } => {
				seq.serialize_element("settle")?;
				seq.serialize_element(id)?;
			},
			Self::Focus { id } => {
				seq.serialize_element("focus")?;
				seq.serialize_element(id)?;
			},
			Self::Reveal { id, at } => {
				seq.serialize_element("reveal")?;
				seq.serialize_element(id)?;
				seq.serialize_element(at)?;
			},
			Self::Scroll { id, by } => {
				seq.serialize_element("scroll")?;
				seq.serialize_element(id)?;
				seq.serialize_element(by)?;
			},
			Self::Suspend => seq.serialize_element("suspend")?,
			Self::Resume => seq.serialize_element("resume")?,
		}
		seq.end()
	}
}

/// The names of a palette's two variants.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PaletteNames {
	/// The dark variant's name.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dark:  Option<String>,
	/// The light variant's name.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub light: Option<String>,
}

/// A `t` message: the program palette, token → `#rrggbb` per variant.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Palette {
	/// The surface (the live one when absent).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sf:    Option<String>,
	/// The dark variant.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub dark:  Option<BTreeMap<String, String>>,
	/// The light variant.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub light: Option<BTreeMap<String, String>>,
	/// The variants' names.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name:  Option<PaletteNames>,
}

/// An `s` message: install, replace or (without `css`) remove a stylesheet.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Sheet {
	/// The surface (the live one when absent).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub sf:   Option<String>,
	/// The sheet's name (1–64 of `A-Z a-z 0-9 _ -`).
	pub name: String,
	/// The CSS; absent removes the sheet.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub css:  Option<String>,
}

/// An `x` message: close a surface.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Close {
	/// The surface id.
	pub id:   String,
	/// Keep it on screen (`main` stays in the scrollback).
	pub keep: bool,
}

/// A `b` message: binary data named by its SHA-256.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Blob {
	/// The lowercase hex SHA-256 of `data`.
	pub id:   String,
	/// The MIME type.
	pub mime: Option<String>,
	/// The bytes.
	pub data: Vec<u8>,
}

impl Blob {
	/// A blob of `data`, its id computed.
	///
	/// # Errors
	/// [`Error::BlobTooLarge`] over [`MAX_BLOB`] bytes.
	pub fn new(data: Vec<u8>, mime: Option<String>) -> Result<Self, Error> {
		if data.len() > MAX_BLOB {
			return Err(Error::BlobTooLarge(data.len()));
		}
		Ok(Self { id: blob_id(&data), mime, data })
	}
}

/// The id of a blob: the lowercase hex SHA-256 of its bytes.
pub fn blob_id(data: &[u8]) -> String {
	use fmt::Write as _;
	let digest = Sha256::digest(data);
	let mut out = String::with_capacity(64);
	for b in digest {
		let _ = write!(out, "{b:02x}");
	}
	out
}

/// Any program → terminal message.
#[derive(Clone, Debug, PartialEq)]
pub enum Message {
	/// `q`.
	Query(Query),
	/// `o`.
	Open(Open),
	/// `f`.
	Frame(Frame),
	/// `b`.
	Blob(Blob),
	/// `t`.
	Palette(Palette),
	/// `s`.
	Sheet(Sheet),
	/// `x`.
	Close(Close),
}

impl Message {
	/// The one-letter verb.
	pub const fn verb(&self) -> &'static str {
		match self {
			Self::Query(_) => "q",
			Self::Open(_) => "o",
			Self::Frame(_) => "f",
			Self::Blob(_) => "b",
			Self::Palette(_) => "t",
			Self::Sheet(_) => "s",
			Self::Close(_) => "x",
		}
	}

	/// The parameters, in order (only a blob has any).
	pub fn params(&self) -> Vec<(&'static str, &str)> {
		match self {
			Self::Blob(b) => {
				let mut out = vec![("id", b.id.as_str())];
				if let Some(mime) = &b.mime {
					out.push(("mime", mime.as_str()));
				}
				out
			},
			_ => Vec::new(),
		}
	}

	/// The body: compact JSON, or base64 for a blob.
	///
	/// # Errors
	/// When serialization fails (it doesn't for these types).
	pub fn body(&self) -> Result<String, Error> {
		Ok(match self {
			Self::Query(m) => serde_json::to_string(m)?,
			Self::Open(m) => serde_json::to_string(m)?,
			Self::Frame(m) => serde_json::to_string(m)?,
			Self::Blob(b) => base64::engine::general_purpose::STANDARD.encode(&b.data),
			Self::Palette(m) => serde_json::to_string(m)?,
			Self::Sheet(m) => serde_json::to_string(m)?,
			Self::Close(m) => serde_json::to_string(m)?,
		})
	}
}

/// A byte of a parameter key.
const fn key_byte(b: u8) -> bool {
	b.is_ascii_alphanumeric() || b == b'_' || b == b'-'
}

/// A byte of a parameter value: printable ASCII other than `;`.
const fn value_byte(b: u8) -> bool {
	b >= 0x21 && b <= 0x7e && b != b';'
}

/// Whether `rest`, placed after a message's parameters, would be read as one
/// more parameter: a `key=value` segment with a `;` after it.
fn leads_with_parameter(rest: &[u8]) -> bool {
	let key = rest.iter().take_while(|&&b| key_byte(b)).count();
	if key == 0 || rest.get(key) != Some(&b'=') {
		return false;
	}
	rest[key + 1..].iter().find(|&&b| !value_byte(b)) == Some(&b';')
}

/// Where the chunk of `body` starting at `start` ends.
fn chunk_end(body: &[u8], start: usize, limit: usize) -> usize {
	let target = start.saturating_add(limit.max(1));
	if target >= body.len() {
		return body.len();
	}
	let safe = |at: usize| body[at] & 0xc0 != 0x80 && !leads_with_parameter(&body[at..]);
	(start + 1..=target)
		.rev()
		.find(|&at| safe(at))
		.or_else(|| (target + 1..body.len()).find(|&at| safe(at)))
		.unwrap_or(body.len())
}

/// Splits a body into chunk bodies exactly as Tern's `tsp_chunks` does.
///
/// Each piece is at most `limit` bytes, cut where a cut is safe: on a UTF-8
/// boundary, and never where the next piece would lead with a
/// parameter-shaped segment. A body within `limit` is one chunk; an empty
/// body has none.
pub fn chunks(body: &[u8], limit: usize) -> impl Iterator<Item = &[u8]> {
	let mut start = 0;
	std::iter::from_fn(move || {
		(start < body.len()).then(|| {
			let end = chunk_end(body, start, limit);
			let piece = &body[start..end];
			start = end;
			piece
		})
	})
}

/// Appends one APC string `ESC _ tsp;<verb>;<params…>;<body> ESC \`.
fn push_apc(out: &mut Vec<u8>, verb: &str, params: &[(&str, &str)], body: &[u8]) {
	out.extend_from_slice(b"\x1b_tsp;");
	out.extend_from_slice(verb.as_bytes());
	out.push(b';');
	for (k, v) in params {
		out.extend_from_slice(k.as_bytes());
		out.push(b'=');
		out.extend_from_slice(v.as_bytes());
		out.push(b';');
	}
	out.extend_from_slice(body);
	out.extend_from_slice(b"\x1b\\");
}

/// Encodes one message of `verb` with `params` and an already serialized
/// `body`, chunked under `limit` with chunk id `chunk` when it is longer.
///
/// # Example
/// ```
/// let out = tern_sdk::wire::encode("o", &[], br#"{"id":"s1"}"#, 65536, "1");
/// assert_eq!(out, b"\x1b_tsp;o;{\"id\":\"s1\"}\x1b\\");
/// ```
pub fn encode(
	verb: &str,
	params: &[(&str, &str)],
	body: &[u8],
	limit: usize,
	chunk: &str,
) -> Vec<u8> {
	let mut out = Vec::with_capacity(body.len() + 16);
	if body.len() <= limit {
		push_apc(&mut out, verb, params, body);
		return out;
	}
	let pieces: Vec<&[u8]> = chunks(body, limit).collect();
	let last = pieces.len() - 1;
	for (i, piece) in pieces.into_iter().enumerate() {
		let mut ps: Vec<(&str, &str)> = if i == 0 { params.to_vec() } else { Vec::new() };
		ps.push(("c", chunk));
		if i < last {
			ps.push(("m", "1"));
		}
		push_apc(&mut out, verb, &ps, piece);
	}
	out
}

/// Encodes messages under the session's `apc` limit, numbering chunk
/// sequences with a base-36 counter.
#[derive(Clone, Debug)]
pub struct Encoder {
	/// The largest body per APC string.
	limit:      usize,
	/// The next chunk id.
	next_chunk: u64,
}

impl Default for Encoder {
	fn default() -> Self {
		Self::new(APC_LIMIT)
	}
}

impl Encoder {
	/// An encoder chunking bodies over `limit` bytes.
	pub const fn new(limit: usize) -> Self {
		Self { limit, next_chunk: 1 }
	}

	/// The limit in use.
	pub const fn limit(&self) -> usize {
		self.limit
	}

	/// Changes the limit (the `hello` reply's `apc`).
	pub const fn set_limit(&mut self, limit: usize) {
		self.limit = limit;
	}

	/// The bytes of `message`, chunked when its body is over the limit.
	///
	/// # Errors
	/// When the body can't be serialized.
	pub fn encode(&mut self, message: &Message) -> Result<Vec<u8>, Error> {
		let body = message.body()?;
		let chunk = if body.len() > self.limit {
			let id = base36(self.next_chunk);
			self.next_chunk += 1;
			id
		} else {
			String::new()
		};
		Ok(encode(message.verb(), &message.params(), body.as_bytes(), self.limit, &chunk))
	}
}

/// `n` in lowercase base 36.
fn base36(mut n: u64) -> String {
	const DIGITS: &[u8; 36] = b"0123456789abcdefghijklmnopqrstuvwxyz";
	let mut out = Vec::new();
	loop {
		out.push(DIGITS[(n % 36) as usize]);
		n /= 36;
		if n == 0 {
			break;
		}
	}
	out.reverse();
	String::from_utf8(out).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Terminal → program
// ---------------------------------------------------------------------------

/// A cell size in pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Cell {
	/// Width.
	#[serde(default)]
	pub w: f64,
	/// Height.
	#[serde(default)]
	pub h: f64,
}

/// The `hello` reply: what the terminal draws and supports.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hello {
	/// The version picked (`1`).
	#[serde(default)]
	pub v:             u32,
	/// The terminal (`tern`).
	#[serde(default)]
	pub term:          String,
	/// The terminal's version.
	#[serde(default)]
	pub ver:           String,
	/// Every kind it draws.
	#[serde(default)]
	pub kinds:         Vec<String>,
	/// Terminal features.
	#[serde(default)]
	pub features:      Vec<String>,
	/// The largest body per message.
	#[serde(default = "default_apc")]
	pub apc:           usize,
	/// Frames that may be unacknowledged.
	#[serde(default = "default_credits")]
	pub credits:       u32,
	/// The pane's width in columns.
	#[serde(default)]
	pub cols:          u32,
	/// The cell size.
	#[serde(default)]
	pub cell:          Option<Cell>,
	/// Whether the terminal shows its dark appearance.
	#[serde(default)]
	pub dark:          bool,
	/// Whether Reduce Motion is on.
	#[serde(default)]
	pub reduce_motion: bool,
	/// Whether the user's system reads a 12-hour clock (`3:05 PM`); false
	/// (24-hour) on terminals that don't say.
	#[serde(default)]
	pub hour12:        bool,
	/// The reply as received.
	#[serde(skip)]
	pub raw:           Map<String, Value>,
}

/// The default of `apc`.
const fn default_apc() -> usize {
	APC_LIMIT
}

/// The default of `credits`.
const fn default_credits() -> u32 {
	CREDITS
}

/// The `blobs` reply: which of the asked blobs Tern holds.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
pub struct BlobsReply {
	/// The ids held, in the order asked.
	#[serde(default)]
	pub have: Vec<String>,
	/// The reply as received.
	#[serde(skip)]
	pub raw:  Map<String, Value>,
}

/// An `r` message: the reply to a query.
#[derive(Clone, Debug, PartialEq)]
pub enum Reply {
	/// `hello`.
	Hello(Hello),
	/// `blobs`.
	Blobs(BlobsReply),
	/// Any other reply, kept raw.
	Unknown(Map<String, Value>),
}

/// Types decoded from a raw object that keep it.
trait Raw: DeserializeOwned {
	/// Stores the raw object.
	fn set_raw(&mut self, raw: Map<String, Value>);
}

/// Decodes `map` as `T`, or `None` when its fields don't fit.
fn typed<T: Raw>(map: Map<String, Value>) -> Result<T, Map<String, Value>> {
	let value = Value::Object(map);
	let decoded = T::deserialize(&value);
	let Value::Object(map) = value else {
		unreachable!("built as an object")
	};
	match decoded {
		Ok(mut t) => {
			t.set_raw(map);
			Ok(t)
		},
		Err(_) => Err(map),
	}
}

impl Reply {
	/// Decodes a reply body; fields that don't fit keep it raw.
	pub fn decode(map: Map<String, Value>) -> Self {
		match map.get("r").and_then(Value::as_str) {
			Some("hello") => typed(map).map_or_else(Self::Unknown, Self::Hello),
			Some("blobs") => typed(map).map_or_else(Self::Unknown, Self::Blobs),
			_ => Self::Unknown(map),
		}
	}

	/// The reply as received.
	pub const fn raw(&self) -> &Map<String, Value> {
		match self {
			Self::Hello(r) => &r.raw,
			Self::Blobs(r) => &r.raw,
			Self::Unknown(r) => r,
		}
	}
}

/// Defines an event payload struct that keeps its raw object.
macro_rules! event_struct {
	($(#[$m:meta])* $name:ident { $($(#[$fm:meta])* $field:ident : $ty:ty),* $(,)? }) => {
		$(#[$m])*
		#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
		#[allow(clippy::derive_partial_eq_without_eq, reason = "some payloads hold floats")]
		pub struct $name {
			$($(#[$fm])* #[serde(default)] pub $field: $ty,)*
			/// The event as received.
			#[serde(skip)]
			pub raw: Map<String, Value>,
		}

		impl Raw for $name {
			fn set_raw(&mut self, raw: Map<String, Value>) {
				self.raw = raw;
			}
		}
	};
}

impl Raw for Hello {
	fn set_raw(&mut self, raw: Map<String, Value>) {
		self.raw = raw;
	}
}

impl Raw for BlobsReply {
	fn set_raw(&mut self, raw: Map<String, Value>) {
		self.raw = raw;
	}
}

event_struct!(
	/// `ack`: the highest frame applied and drawn.
	Ack {
		/// The surface.
		sf: String,
		/// The frame's sequence number.
		s:  u64,
	}
);

event_struct!(
	/// `resize`: the pane's width or cell size changed.
	Resize {
		/// The surface.
		sf: Option<String>,
		/// The width in columns.
		cols: u32,
		/// The cell size.
		cell: Option<Cell>,
		/// Whether the pane is visible.
		visible: Option<bool>,
	}
);

event_struct!(
	/// `theme`: the appearance switched.
	Theme {
		/// Whether it is dark now.
		dark: bool,
	}
);

event_struct!(
	/// `motion`: Reduce Motion was toggled.
	Motion {
		/// Whether motion is reduced now.
		reduce: bool,
	}
);

event_struct!(
	/// `visible`: the pane was hidden or shown.
	Visible {
		/// The surface.
		sf: Option<String>,
		/// Whether it shows.
		visible: bool,
	}
);

event_struct!(
	/// `toggle`: a collapsible node was folded or unfolded.
	Toggle {
		/// The surface.
		sf: Option<String>,
		/// The node.
		id: String,
		/// Whether it is folded now.
		collapsed: bool,
		/// The node's `key` prop (or a `tree` item).
		key: Option<Value>,
	}
);

event_struct!(
	/// `select` or `activate`: an item was picked or opened.
	Select {
		/// The surface.
		sf: Option<String>,
		/// The node (the list, for a list's item).
		id: String,
		/// The item.
		item: String,
		/// Form values, inside an `el` form.
		values: Option<Map<String, Value>>,
	}
);

event_struct!(
	/// `action`: a pointer action with a custom name.
	Action {
		/// The surface.
		sf: Option<String>,
		/// The node.
		id: String,
		/// The action's name (before any `=`).
		act: String,
		/// The part after `=` in the action's name.
		value: Option<String>,
		/// Modifiers held, in the order `shift`, `ctrl`, `alt`, `meta`.
		mods: Vec<String>,
		/// Form values, inside an `el` form.
		values: Option<Map<String, Value>>,
	}
);

event_struct!(
	/// `change`: an `el` control flipped, or a `prefs` row changed.
	Change {
		/// The surface.
		sf: Option<String>,
		/// The node.
		id: String,
		/// The control's value, or the `prefs` row's new value.
		value: Value,
		/// The control's new state.
		checked: Option<bool>,
		/// The control's name.
		name: Option<String>,
		/// The `prefs` row.
		item: Option<String>,
		/// Form values, inside an `el` form.
		values: Option<Map<String, Value>>,
	}
);

event_struct!(
	/// `focus`: a click asked for the keys in a field.
	Focus {
		/// The surface.
		sf: Option<String>,
		/// The field.
		id: String,
	}
);

event_struct!(
	/// `edit`: native editing replaced UTF-16 `[from, to)` with `text`.
	Edit {
		/// The surface.
		sf: Option<String>,
		/// The field.
		id: String,
		/// Start, UTF-16 units.
		from: usize,
		/// End, UTF-16 units.
		to: usize,
		/// The replacement.
		text: String,
		/// The caret afterwards.
		cursor: usize,
		/// The length of the text Tern saw.
		len: usize,
	}
);

event_struct!(
	/// `undo`: undo the last change to a field.
	Undo {
		/// The surface.
		sf: Option<String>,
		/// The field.
		id: String,
	}
);

event_struct!(
	/// `send`: submit text through a composer.
	SendText {
		/// The surface.
		sf: Option<String>,
		/// The field.
		id: String,
		/// The text.
		text: String,
	}
);

event_struct!(
	/// `error`: a rejected op, message, stylesheet or element.
	ErrorEvent {
		/// The surface, when the message named one.
		sf: Option<String>,
		/// The frame.
		s: Option<u64>,
		/// The op's index in the frame.
		op: Option<u64>,
		/// What went wrong.
		msg: String,
		/// The stylesheet.
		sheet: Option<String>,
		/// The element.
		id: Option<String>,
	}
);

event_struct!(
	/// `gone`: nodes or a surface Tern dropped.
	Gone {
		/// The surface.
		sf: Option<String>,
		/// The ids dropped.
		ids: Vec<String>,
	}
);

/// An `e` message: an event.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
	/// `ack`.
	Ack(Ack),
	/// `resize`.
	Resize(Resize),
	/// `theme`.
	Theme(Theme),
	/// `motion`.
	Motion(Motion),
	/// `visible`.
	Visible(Visible),
	/// `toggle`.
	Toggle(Toggle),
	/// `select`.
	Select(Select),
	/// `activate`.
	Activate(Select),
	/// `action`.
	Action(Action),
	/// `change`.
	Change(Change),
	/// `focus`.
	Focus(Focus),
	/// `edit`.
	Edit(Edit),
	/// `undo`.
	Undo(Undo),
	/// `send`.
	Send(SendText),
	/// `error`.
	Error(ErrorEvent),
	/// `gone`.
	Gone(Gone),
	/// Any other event, or a known one whose fields don't fit, kept raw.
	Unknown(Map<String, Value>),
}

impl Event {
	/// Decodes an event body; fields that don't fit keep it raw.
	pub fn decode(map: Map<String, Value>) -> Self {
		let ev = map
			.get("ev")
			.and_then(Value::as_str)
			.unwrap_or_default()
			.to_owned();
		match ev.as_str() {
			"ack" => typed(map).map_or_else(Self::Unknown, Self::Ack),
			"resize" => typed(map).map_or_else(Self::Unknown, Self::Resize),
			"theme" => typed(map).map_or_else(Self::Unknown, Self::Theme),
			"motion" => typed(map).map_or_else(Self::Unknown, Self::Motion),
			"visible" => typed(map).map_or_else(Self::Unknown, Self::Visible),
			"toggle" => typed(map).map_or_else(Self::Unknown, Self::Toggle),
			"select" => typed(map).map_or_else(Self::Unknown, Self::Select),
			"activate" => typed(map).map_or_else(Self::Unknown, Self::Activate),
			"action" => typed(map).map_or_else(Self::Unknown, Self::Action),
			"change" => typed(map).map_or_else(Self::Unknown, Self::Change),
			"focus" => typed(map).map_or_else(Self::Unknown, Self::Focus),
			"edit" => typed(map).map_or_else(Self::Unknown, Self::Edit),
			"undo" => typed(map).map_or_else(Self::Unknown, Self::Undo),
			"send" => typed(map).map_or_else(Self::Unknown, Self::Send),
			"error" => typed(map).map_or_else(Self::Unknown, Self::Error),
			"gone" => typed(map).map_or_else(Self::Unknown, Self::Gone),
			_ => Self::Unknown(map),
		}
	}

	/// The event as received.
	pub const fn raw(&self) -> &Map<String, Value> {
		match self {
			Self::Ack(e) => &e.raw,
			Self::Resize(e) => &e.raw,
			Self::Theme(e) => &e.raw,
			Self::Motion(e) => &e.raw,
			Self::Visible(e) => &e.raw,
			Self::Toggle(e) => &e.raw,
			Self::Select(e) | Self::Activate(e) => &e.raw,
			Self::Action(e) => &e.raw,
			Self::Change(e) => &e.raw,
			Self::Focus(e) => &e.raw,
			Self::Edit(e) => &e.raw,
			Self::Undo(e) => &e.raw,
			Self::Send(e) => &e.raw,
			Self::Error(e) => &e.raw,
			Self::Gone(e) => &e.raw,
			Self::Unknown(e) => e,
		}
	}

	/// The event's name (`ev`).
	pub fn name(&self) -> &str {
		self
			.raw()
			.get("ev")
			.and_then(Value::as_str)
			.unwrap_or_default()
	}

	/// The surface the event is about, when it names one.
	pub fn sf(&self) -> Option<&str> {
		self.raw().get("sf").and_then(Value::as_str)
	}

	/// The node the event is about, when it names one.
	pub fn id(&self) -> Option<&str> {
		self.raw().get("id").and_then(Value::as_str)
	}
}
