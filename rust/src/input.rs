//! Layer 2a: the input parser. It splits TSP replies and events, and DA1
//! answers, out of the pty's input and passes every other byte through as
//! key bytes, exactly as they came.
//!
//! A TSP message is `ESC _ tsp;` (or `ESC ] 877;tsp;` through a Windows
//! `ConPTY`), then `<verb>[;k=v]*;<body>`, ended by ST or BEL. Bytes inside a
//! bracketed paste pass through unexamined, so pasted text can never forge
//! an event.

use serde_json::Value;

use crate::wire::{Event, Reply};

/// A TSP message that grows past this many bytes unterminated is dropped.
pub const MAX_MESSAGE: usize = 32 << 20;

/// The start of a bracketed paste.
const PASTE_START: &[u8] = b"\x1b[200~";
/// The end of a bracketed paste.
const PASTE_END: &[u8] = b"\x1b[201~";
/// What follows `ESC _` in a TSP message.
const APC_PREFIX: &[u8] = b"tsp;";
/// What follows `ESC ]` in a TSP message through `ConPTY`.
const OSC_PREFIX: &[u8] = b"877;tsp;";

/// One thing the parser found in the input.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
	/// Bytes that aren't TSP: keys, pastes, other terminal reports.
	Keys(Vec<u8>),
	/// A reply to a query.
	Reply(Reply),
	/// An event.
	Event(Event),
	/// A DA1 answer (`ESC [ ? … c`).
	Da1,
}

/// What a prefix of the buffer is.
enum Scan {
	/// Not decidable yet: hold the bytes.
	Undecided,
	/// Pass this many bytes through as keys.
	Keys(usize),
	/// A DA1 answer of this many bytes.
	Da1(usize),
	/// A bracketed paste starts; pass this many bytes through.
	Paste(usize),
	/// A TSP message whose content starts at this offset.
	Tsp(usize),
}

/// The streaming input parser.
///
/// # Example
/// ```
/// use tern_sdk::input::{Item, Parser};
///
/// let mut parser = Parser::new();
/// let items = parser.feed(b"x\x1b_tsp;e;{\"ev\":\"ack\",\"sf\":\"s1\",\"s\":3}\x1b\\y");
/// assert_eq!(items.len(), 3);
/// assert!(matches!(&items[0], Item::Keys(k) if k == b"x"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Parser {
	/// Bytes not yet decided or a TSP message not yet terminated.
	buf:      Vec<u8>,
	/// Inside a bracketed paste.
	paste:    bool,
	/// Content offset of the TSP message at the start of `buf`.
	tsp:      Option<usize>,
	/// How far the TSP message has been searched for its terminator.
	scanned:  usize,
	/// Dropping an oversized TSP message until its terminator.
	skipping: bool,
}

impl Parser {
	/// A parser with nothing held.
	pub fn new() -> Self {
		Self::default()
	}

	/// Whether bytes are held that a [`flush`](Self::flush) would release
	/// (an undecided prefix, not a recognized TSP message or active paste).
	pub const fn pending(&self) -> bool {
		self.tsp.is_none() && !self.skipping && !self.paste && !self.buf.is_empty()
	}

	/// Feeds bytes as they arrive and returns what they complete, in order.
	pub fn feed(&mut self, bytes: &[u8]) -> Vec<Item> {
		let mut out = Vec::new();
		self.buf.extend_from_slice(bytes);
		self.run(&mut out);
		out
	}

	/// Releases an undecided prefix (a lone `ESC`, a partial CSI) as keys.
	/// A recognized TSP message or a paste's partial end marker stays held.
	pub fn flush(&mut self) -> Vec<Item> {
		if self.pending() {
			vec![Item::Keys(std::mem::take(&mut self.buf))]
		} else {
			Vec::new()
		}
	}

	/// Consumes `buf` as far as it can be decided.
	fn run(&mut self, out: &mut Vec<Item>) {
		let mut keys: Vec<u8> = Vec::new();
		let mut at = 0;
		loop {
			if self.skipping || self.tsp.is_some() {
				match self.message(at, out, &mut keys) {
					Some(next) => at = next,
					None => break,
				}
				continue;
			}
			if self.paste {
				let rest = &self.buf[at..];
				if let Some(end) = find(rest, PASTE_END) {
					keys.extend_from_slice(&rest[..end + PASTE_END.len()]);
					at += end + PASTE_END.len();
					self.paste = false;
					continue;
				}
				let hold = partial_suffix(rest, PASTE_END);
				keys.extend_from_slice(&rest[..rest.len() - hold]);
				at = self.buf.len() - hold;
				break;
			}
			let rest = &self.buf[at..];
			if rest.is_empty() {
				break;
			}
			let plain = rest.iter().position(|&b| b == 0x1b).unwrap_or(rest.len());
			if plain > 0 {
				keys.extend_from_slice(&rest[..plain]);
				at += plain;
				continue;
			}
			match scan(rest) {
				Scan::Undecided => break,
				Scan::Keys(n) => {
					keys.extend_from_slice(&rest[..n]);
					at += n;
				},
				Scan::Paste(n) => {
					keys.extend_from_slice(&rest[..n]);
					at += n;
					self.paste = true;
				},
				Scan::Da1(n) => {
					push_keys(out, &mut keys);
					out.push(Item::Da1);
					at += n;
				},
				Scan::Tsp(content) => {
					// Rebase the message at the buffer's start.
					self.buf.drain(..at);
					self.tsp = Some(content);
					self.scanned = content;
					at = 0;
				},
			}
		}
		push_keys(out, &mut keys);
		self.buf.drain(..at);
	}

	/// Handles the TSP message at the start of `buf[at..]` (`at` is 0 when a
	/// message is held). Returns where to continue, or `None` when it waits
	/// for more bytes.
	fn message(&mut self, at: usize, out: &mut Vec<Item>, keys: &mut Vec<u8>) -> Option<usize> {
		debug_assert_eq!(at, 0, "a TSP message is rebased to the buffer start");
		let from = self.scanned;
		let Some((end, term_len)) = terminator(&self.buf[from..]).map(|(i, n)| (from + i, n)) else {
			// A trailing ESC may start the ST: search it again with what follows.
			self.scanned = self.buf.len().saturating_sub(1).max(from);
			if !self.skipping && self.buf.len() > MAX_MESSAGE {
				tracing::warn!(bytes = self.buf.len(), "dropping an oversized TSP message");
				self.skipping = true;
				self.tsp = None;
			}
			if self.skipping {
				let keep = usize::from(self.buf.last() == Some(&0x1b));
				let len = self.buf.len();
				self.buf.drain(..len - keep);
				self.scanned = 0;
			}
			return None;
		};
		if !self.skipping
			&& let Some(content) = self.tsp
		{
			push_keys(out, keys);
			if let Some(item) = decode(&self.buf[content..end]) {
				out.push(item);
			}
		}
		self.skipping = false;
		self.tsp = None;
		self.scanned = 0;
		self.buf.drain(..end + term_len);
		Some(0)
	}
}

/// Moves accumulated key bytes into an item.
fn push_keys(out: &mut Vec<Item>, keys: &mut Vec<u8>) {
	if !keys.is_empty() {
		out.push(Item::Keys(std::mem::take(keys)));
	}
}

/// The first index of `needle` in `hay`.
fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
	hay.windows(needle.len()).position(|w| w == needle)
}

/// The length of the longest proper prefix of `needle` that `hay` ends with.
fn partial_suffix(hay: &[u8], needle: &[u8]) -> usize {
	(1..needle.len())
		.rev()
		.find(|&n| hay.ends_with(&needle[..n]))
		.unwrap_or(0)
}

/// The terminator (ST or BEL) in a message's bytes: its index and length.
fn terminator(bytes: &[u8]) -> Option<(usize, usize)> {
	let mut i = 0;
	while i < bytes.len() {
		match bytes[i] {
			0x07 => return Some((i, 1)),
			0x1b if bytes.get(i + 1) == Some(&b'\\') => return Some((i, 2)),
			_ => {},
		}
		i += 1;
	}
	None
}

/// Classifies the escape sequence at the start of `rest` (`rest[0]` is
/// `ESC`).
fn scan(rest: &[u8]) -> Scan {
	let Some(&second) = rest.get(1) else {
		return Scan::Undecided;
	};
	match second {
		b'_' => prefixed(rest, APC_PREFIX),
		b']' => prefixed(rest, OSC_PREFIX),
		b'[' => csi(rest),
		// Any other ESC passes through alone; what follows is scanned anew.
		_ => Scan::Keys(1),
	}
}

/// `ESC <c> <prefix>` starts a TSP message; anything else passes through.
fn prefixed(rest: &[u8], prefix: &[u8]) -> Scan {
	let after = &rest[2..];
	let n = after.len().min(prefix.len());
	if after[..n] != prefix[..n] {
		return Scan::Keys(2);
	}
	if n < prefix.len() {
		return Scan::Undecided;
	}
	Scan::Tsp(2 + prefix.len())
}

/// A CSI sequence: a DA1 answer, a paste start, or keys.
fn csi(rest: &[u8]) -> Scan {
	let mut i = 2;
	while let Some(&b) = rest.get(i) {
		match b {
			0x20..=0x3f => i += 1,
			0x40..=0x7e => {
				let seq = &rest[..=i];
				let params = &seq[2..i];
				if b == b'c'
					&& params.first() == Some(&b'?')
					&& params[1..].iter().all(|&p| p.is_ascii_digit() || p == b';')
				{
					return Scan::Da1(i + 1);
				}
				if seq == PASTE_START {
					return Scan::Paste(i + 1);
				}
				return Scan::Keys(i + 1);
			},
			// Not a CSI byte: the sequence ends here as keys.
			_ => return Scan::Keys(i),
		}
	}
	Scan::Undecided
}

/// Decodes a message's content (`<verb>[;k=v]*;<body>`): `r` and `e`
/// bodies that are JSON objects; everything else is dropped.
fn decode(content: &[u8]) -> Option<Item> {
	let semi = content.iter().position(|&b| b == b';')?;
	let verb = &content[..semi];
	let mut pos = semi + 1;
	while let Some(len) = content[pos..].iter().position(|&b| b == b';') {
		if !parameter(&content[pos..pos + len]) {
			break;
		}
		pos += len + 1;
	}
	let body = &content[pos..];
	let Ok(Value::Object(map)) = serde_json::from_slice::<Value>(body) else {
		tracing::debug!("dropping a TSP message whose body is not a JSON object");
		return None;
	};
	match verb {
		b"r" => Some(Item::Reply(Reply::decode(map))),
		b"e" => Some(Item::Event(Event::decode(map))),
		_ => None,
	}
}

/// Whether a segment is `key=value` (`[A-Za-z0-9_-]+=[\x21-\x7e but ;]*`).
fn parameter(segment: &[u8]) -> bool {
	let Some(eq) = segment.iter().position(|&b| b == b'=') else {
		return false;
	};
	let (key, value) = (&segment[..eq], &segment[eq + 1..]);
	!key.is_empty()
		&& key
			.iter()
			.all(|&b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
		&& value
			.iter()
			.all(|&b| (0x21..=0x7e).contains(&b) && b != b';')
}
