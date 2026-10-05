//! Layer 2b: the key decoder. It turns key bytes (what the input parser
//! passes through) into keys named as in Tern's plugin API: legacy xterm
//! input, kitty `CSI <code>[;<mods>] u` keys and bracketed paste.

use std::fmt;

use serde::Serialize;

/// One key press, or a paste (`name` `"paste"` with its text).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Key {
	/// The key: a lowercase character (`"a"`, `"é"`), `"space"`, a named key
	/// (`"enter"`, `"up"`, `"f5"`, …) or `"paste"`.
	pub name:  String,
	/// What the key types, when it types something.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub text:  Option<String>,
	/// Control held.
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	pub ctrl:  bool,
	/// Alt (Option) held.
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	pub alt:   bool,
	/// Shift held.
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	pub shift: bool,
	/// Cmd on macOS, Super elsewhere.
	#[serde(skip_serializing_if = "std::ops::Not::not")]
	pub meta:  bool,
}

impl Key {
	/// A key without modifiers or text.
	pub fn named(name: impl Into<String>) -> Self {
		Self { name: name.into(), ..Self::default() }
	}

	/// Whether this is the chord `chord`: modifiers and a name joined by `+`
	/// (`"ctrl+c"`, `"shift+tab"`, `"escape"`, `"ctrl++"`), every modifier
	/// matching exactly.
	///
	/// # Example
	/// ```
	/// use tern_sdk::keys::Key;
	///
	/// let key = Key { ctrl: true, ..Key::named("c") };
	/// assert!(key.is("ctrl+c"));
	/// assert!(!key.is("c"));
	/// ```
	pub fn is(&self, chord: &str) -> bool {
		let (mut ctrl, mut alt, mut shift, mut meta) = (false, false, false, false);
		let mut name = chord;
		while let Some((modifier, rest)) = name.split_once('+') {
			match modifier {
				"ctrl" => ctrl = true,
				"alt" => alt = true,
				"shift" => shift = true,
				"meta" | "cmd" | "super" => meta = true,
				_ => break,
			}
			name = rest;
		}
		self.name == name
			&& self.ctrl == ctrl
			&& self.alt == alt
			&& self.shift == shift
			&& self.meta == meta
	}

	/// Whether the key types text without Control, Alt or Meta held.
	pub fn typed(&self) -> Option<&str> {
		if self.ctrl || self.alt || self.meta || self.name == "paste" {
			return None;
		}
		self.text.as_deref()
	}

	/// Applies xterm modifier parameter `param` (1 + bits).
	const fn with_mods(mut self, param: u32) -> Self {
		let bits = param.saturating_sub(1);
		self.shift |= bits & 1 != 0;
		self.alt |= bits & 2 != 0;
		self.ctrl |= bits & 4 != 0;
		self.meta |= bits & (8 | 32) != 0;
		self
	}
}

impl fmt::Display for Key {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for (on, name) in
			[(self.ctrl, "ctrl+"), (self.alt, "alt+"), (self.shift, "shift+"), (self.meta, "meta+")]
		{
			if on {
				f.write_str(name)?;
			}
		}
		f.write_str(&self.name)
	}
}

/// What decoding the start of the buffer gave.
enum Step {
	/// Not decidable yet.
	Wait,
	/// This many bytes were consumed, giving maybe a key.
	Took(usize, Option<Key>),
}

/// The streaming key decoder.
///
/// # Example
/// ```
/// use tern_sdk::keys::Decoder;
///
/// let mut keys = Decoder::new();
/// let got = keys.feed(b"\x1b[1;5C");
/// assert!(got[0].is("ctrl+right"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Decoder {
	/// Bytes of an undecided sequence.
	buf:   Vec<u8>,
	/// The text of a paste in progress.
	paste: Option<Vec<u8>>,
}

impl Decoder {
	/// A decoder with nothing held.
	pub fn new() -> Self {
		Self::default()
	}

	/// Whether bytes are held that [`flush`](Self::flush) would decide.
	pub const fn pending(&self) -> bool {
		!self.buf.is_empty() && self.paste.is_none()
	}

	/// Feeds key bytes and returns the keys they complete.
	pub fn feed(&mut self, bytes: &[u8]) -> Vec<Key> {
		self.buf.extend_from_slice(bytes);
		let mut out = Vec::new();
		self.run(&mut out, false);
		out
	}

	/// Decides what is held: a lone `ESC` is Escape, an incomplete sequence
	/// is dropped. A paste in progress keeps waiting for its end.
	pub fn flush(&mut self) -> Vec<Key> {
		let mut out = Vec::new();
		self.run(&mut out, true);
		out
	}

	/// Decodes as much of the buffer as it can.
	fn run(&mut self, out: &mut Vec<Key>, flush: bool) {
		let mut at = 0;
		while at < self.buf.len() {
			if let Some(paste) = &mut self.paste {
				let rest = &self.buf[at..];
				if let Some(end) = rest.windows(6).position(|w| w == b"\x1b[201~") {
					paste.extend_from_slice(&rest[..end]);
					at += end + 6;
					let text =
						String::from_utf8_lossy(&self.paste.take().unwrap_or_default()).into_owned();
					out.push(Key { text: Some(text), ..Key::named("paste") });
					continue;
				}
				let hold = (1..6)
					.rev()
					.find(|&n| rest.ends_with(&b"\x1b[201~"[..n]))
					.unwrap_or(0);
				paste.extend_from_slice(&rest[..rest.len() - hold]);
				at = self.buf.len() - hold;
				break;
			}
			match decode(&self.buf[at..], flush) {
				Step::Wait => break,
				Step::Took(n, key) => {
					if &self.buf[at..at + n] == b"\x1b[200~" {
						self.paste = Some(Vec::new());
					}
					at += n;
					out.extend(key);
				},
			}
		}
		self.buf.drain(..at);
	}
}

/// Decodes one key at the start of `b` (non-empty).
fn decode(b: &[u8], flush: bool) -> Step {
	match b[0] {
		0x1b => escape(b, flush),
		c if c < 0x20 || c == 0x7f => Step::Took(1, Some(control(c))),
		_ => match utf8(b, flush) {
			Some((n, Some(ch))) => Step::Took(n, Some(printable(ch))),
			Some((n, None)) => Step::Took(n, None),
			None => Step::Wait,
		},
	}
}

/// A control character's key.
fn control(c: u8) -> Key {
	match c {
		b'\r' | b'\n' => Key::named("enter"),
		b'\t' => Key::named("tab"),
		0x7f => Key::named("backspace"),
		0x08 => Key { ctrl: true, ..Key::named("backspace") },
		0x1b => Key::named("escape"),
		0x00 => Key { ctrl: true, ..Key::named("space") },
		0x01..=0x1a => Key { ctrl: true, ..Key::named(((b'a' + c - 1) as char).to_string()) },
		0x1c => Key { ctrl: true, ..Key::named("\\") },
		0x1d => Key { ctrl: true, ..Key::named("]") },
		0x1e => Key { ctrl: true, ..Key::named("^") },
		_ => Key { ctrl: true, ..Key::named("_") },
	}
}

/// A typed character's key.
fn printable(ch: char) -> Key {
	if ch == ' ' {
		return Key { text: Some(" ".into()), ..Key::named("space") };
	}
	let lower: String = ch.to_lowercase().collect();
	let shift = lower.chars().ne(std::iter::once(ch));
	Key { text: Some(ch.to_string()), shift, ..Key::named(lower) }
}

/// One UTF-8 character at the start of `b`: its length and the character
/// (`None` for an invalid byte), or `None` when it is incomplete.
fn utf8(b: &[u8], flush: bool) -> Option<(usize, Option<char>)> {
	let len = match b[0] {
		0x00..=0x7f => 1,
		0xc0..=0xdf => 2,
		0xe0..=0xef => 3,
		0xf0..=0xf7 => 4,
		_ => return Some((1, None)),
	};
	if b.len() < len {
		return if flush { Some((b.len(), None)) } else { None };
	}
	match std::str::from_utf8(&b[..len]) {
		Ok(s) => Some((len, s.chars().next())),
		Err(_) => Some((1, None)),
	}
}

/// A sequence starting with `ESC`.
fn escape(b: &[u8], flush: bool) -> Step {
	let Some(&next) = b.get(1) else {
		return if flush {
			Step::Took(1, Some(Key::named("escape")))
		} else {
			Step::Wait
		};
	};
	match next {
		b'[' => csi(b, flush),
		b'O' => match b.get(2) {
			Some(&c) => Step::Took(3, ss3(c)),
			None if flush => Step::Took(2, None),
			None => Step::Wait,
		},
		0x1b => match escape(&b[1..], flush) {
			Step::Wait => Step::Wait,
			Step::Took(n, key) => Step::Took(n + 1, key.map(|k| Key { alt: true, text: None, ..k })),
		},
		_ => match decode(&b[1..], flush) {
			Step::Wait => Step::Wait,
			Step::Took(n, key) => Step::Took(n + 1, key.map(|k| Key { alt: true, text: None, ..k })),
		},
	}
}

/// An SS3 key (`ESC O <c>`).
fn ss3(c: u8) -> Option<Key> {
	let name = match c {
		b'A' => "up",
		b'B' => "down",
		b'C' => "right",
		b'D' => "left",
		b'H' => "home",
		b'F' => "end",
		b'E' => "begin",
		b'P' => "f1",
		b'Q' => "f2",
		b'R' => "f3",
		b'S' => "f4",
		b'M' => "enter",
		_ => return None,
	};
	Some(Key::named(name))
}

/// A CSI sequence (`ESC [ … final`).
fn csi(b: &[u8], flush: bool) -> Step {
	let mut i = 2;
	while let Some(&c) = b.get(i) {
		match c {
			0x20..=0x3f => i += 1,
			0x40..=0x7e => return Step::Took(i + 1, csi_key(&b[2..i], c)),
			_ => return Step::Took(i, None),
		}
	}
	if flush {
		Step::Took(b.len(), None)
	} else {
		Step::Wait
	}
}

/// Parses `a;b:c;d` into numeric fields (each a list of `:` parts).
fn fields(params: &[u8]) -> Option<Vec<Vec<u32>>> {
	let text = std::str::from_utf8(params).ok()?;
	text
		.split(';')
		.map(|f| {
			f.split(':')
				.map(|n| {
					if n.is_empty() {
						Some(0)
					} else {
						n.parse().ok()
					}
				})
				.collect::<Option<Vec<u32>>>()
		})
		.collect()
}

/// The key of a CSI sequence with parameters `params` and final byte `fin`.
fn csi_key(params: &[u8], fin: u8) -> Option<Key> {
	if params
		.first()
		.is_some_and(|&p| matches!(p, b'?' | b'<' | b'>' | b'='))
	{
		return None;
	}
	let f = fields(params)?;
	let num = |i: usize| f.get(i).and_then(|v| v.first()).copied().unwrap_or(0);
	let mods = num(1).max(1);
	match fin {
		b'A' | b'B' | b'C' | b'D' | b'H' | b'F' | b'E' | b'P' | b'Q' | b'R' | b'S' => {
			let name = match fin {
				b'A' => "up",
				b'B' => "down",
				b'C' => "right",
				b'D' => "left",
				b'H' => "home",
				b'F' => "end",
				b'E' => "begin",
				b'P' => "f1",
				b'Q' => "f2",
				b'R' => "f3",
				_ => "f4",
			};
			Some(Key::named(name).with_mods(mods))
		},
		b'Z' => Some(Key { shift: true, ..Key::named("tab") }.with_mods(mods)),
		b'~' => tilde(num(0)).map(|name| Key::named(name).with_mods(mods)),
		b'u' => kitty(&f),
		_ => None,
	}
}

/// The name of a `CSI <n> ~` key.
const fn tilde(n: u32) -> Option<&'static str> {
	Some(match n {
		1 | 7 => "home",
		2 => "insert",
		3 => "delete",
		4 | 8 => "end",
		5 => "page_up",
		6 => "page_down",
		11 => "f1",
		12 => "f2",
		13 => "f3",
		14 => "f4",
		15 => "f5",
		17 => "f6",
		18 => "f7",
		19 => "f8",
		20 => "f9",
		21 => "f10",
		23 => "f11",
		24 => "f12",
		25 => "f13",
		26 => "f14",
		28 => "f15",
		29 => "menu",
		31 => "f17",
		32 => "f18",
		33 => "f19",
		34 => "f20",
		_ => return None,
	})
}

/// A kitty `CSI code[:alt…][;mods[:event]][;text] u` key.
fn kitty(f: &[Vec<u32>]) -> Option<Key> {
	let code = f.first().and_then(|v| v.first()).copied()?;
	let mods_field = f.get(1);
	let mods = mods_field
		.and_then(|v| v.first())
		.copied()
		.unwrap_or(1)
		.max(1);
	// Event type 3 is a release, which is never delivered.
	if mods_field.and_then(|v| v.get(1)).copied() == Some(3) {
		return None;
	}
	let named = |name: &str| Some(Key::named(name).with_mods(mods));
	let key = match code {
		27 => named("escape"),
		13 => named("enter"),
		9 => named("tab"),
		127 | 8 => named("backspace"),
		32 => named("space"),
		57358 => named("caps_lock"),
		57359 => named("scroll_lock"),
		57360 => named("num_lock"),
		57361 => named("print_screen"),
		57362 => named("pause"),
		57363 => named("menu"),
		57364..=57398 => {
			let name = format!("f{}", code - 57364 + 1);
			Some(Key::named(name).with_mods(mods))
		},
		57399..=57427 => keypad(code).map(|name| Key::named(name).with_mods(mods)),
		// Unmapped functional keys, media keys and lone modifiers are not delivered.
		57344..=63743 => None,
		_ => {
			let ch = char::from_u32(code)?;
			let lower: String = ch.to_lowercase().collect();
			Some(Key::named(lower).with_mods(mods))
		},
	}?;
	Some(with_text(key, f.get(2), code))
}

/// Adds the text a kitty key types: its text field, or for a printable key
/// without Control, Alt or Meta, the (shifted) character.
fn with_text(mut key: Key, text: Option<&Vec<u32>>, code: u32) -> Key {
	if key.ctrl || key.alt || key.meta {
		return key;
	}
	if let Some(cps) = text {
		let s: String = cps.iter().filter_map(|&c| char::from_u32(c)).collect();
		if !s.is_empty() {
			key.text = Some(s);
		}
		return key;
	}
	if key.name == "space" {
		key.text = Some(" ".into());
	} else if key.name.chars().count() == 1 && code >= 0x20 {
		key.text = Some(if key.shift {
			key.name.to_uppercase()
		} else {
			key.name.clone()
		});
	}
	key
}

/// The character or name a keypad key stands for.
const fn keypad(code: u32) -> Option<&'static str> {
	Some(match code {
		57399 => "0",
		57400 => "1",
		57401 => "2",
		57402 => "3",
		57403 => "4",
		57404 => "5",
		57405 => "6",
		57406 => "7",
		57407 => "8",
		57408 => "9",
		57409 => ".",
		57410 => "/",
		57411 => "*",
		57412 => "-",
		57413 => "+",
		57414 => "enter",
		57415 => "=",
		57416 => ",",
		57417 => "left",
		57418 => "right",
		57419 => "up",
		57420 => "down",
		57421 => "page_up",
		57422 => "page_down",
		57423 => "home",
		57424 => "end",
		57425 => "insert",
		57426 => "delete",
		57427 => "begin",
		_ => return None,
	})
}
