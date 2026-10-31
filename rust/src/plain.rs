//! The plain-text fallback: a view rendered as readable text without
//! escapes, for output outside Tern. Text kinds are their text, rows join
//! on one line, cards and sections are a head line over an indented body,
//! tables and `kv` are aligned columns, lists are bullets, progress and
//! meters are `[####------] 40%`, motion kinds are their label, and `el` is
//! its text with block tags on lines of their own. The controls, menus,
//! navigation, chat and work kinds read as a line or a few (`[x] Wrap`,
//! `Page 2 of 7`, `- Searched tsp (done)`). It is not a copy of Tern's
//! layout.

use serde_json::{Map, Value};

use crate::ui::{Node, View, node::spans_text};

/// The width of a plain progress bar, in cells.
const BAR: usize = 10;

/// Units a slider writes against its number (`72%`); others follow a space.
const SLIDER_UNITS: [&str; 9] = ["%", "‰", "°", "×", "x", "′", "″", "'", "\""];

/// Currency symbols a donut writes before its numbers (`$51.4`).
const CURRENCY: [&str; 4] = ["$", "€", "£", "¥"];

/// `el` tags that start a line of their own.
const BLOCK_TAGS: [&str; 30] = [
	"div",
	"p",
	"section",
	"header",
	"footer",
	"nav",
	"aside",
	"main",
	"article",
	"figure",
	"blockquote",
	"ul",
	"ol",
	"li",
	"dl",
	"dt",
	"dd",
	"h1",
	"h2",
	"h3",
	"h4",
	"pre",
	"hr",
	"table",
	"thead",
	"tbody",
	"tr",
	"form",
	"label",
	"button",
];

/// Renders a message-free view or builder as plain text `cols` columns
/// wide (rules span it; text isn't wrapped). Lines end with `\n`.
///
/// # Example
/// ```
/// use tern_sdk::ui::{self, View};
///
/// let view: View = View::new().main([ui::progress().value(0.5).label("half")]);
/// assert_eq!(tern_sdk::plain(view, 80), "[#####-----] 50% half\n");
/// assert_eq!(tern_sdk::plain(ui::text("hello"), 80), "hello\n");
/// ```
pub fn plain(view: impl Into<View>, cols: usize) -> String {
	view.into().plain(cols)
}

impl<M> View<M> {
	/// The view as plain text `cols` wide: what a program prints outside
	/// Tern. Works for views of any message type (see [`plain`]).
	pub fn plain(&self, cols: usize) -> String {
		let mut lines = Vec::new();
		for node in self.regions().into_iter().flatten() {
			block(node, cols, &mut lines);
		}
		let mut out = String::new();
		for line in lines {
			out.push_str(line.trim_end());
			out.push('\n');
		}
		out
	}
}

/// The eighths blocks a sparkline is drawn with, low to high.
const SPARKS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

/// Values as a block sparkline, scaled to `min`..`max` (default: their own
/// range).
fn sparkline(values: &[f64], min: Option<f64>, max: Option<f64>) -> String {
	let lo = min.unwrap_or_else(|| values.iter().copied().fold(f64::INFINITY, f64::min));
	let hi = max.unwrap_or_else(|| values.iter().copied().fold(f64::NEG_INFINITY, f64::max));
	let span = hi - lo;
	let top = (SPARKS.len() - 1) as f64;
	values
		.iter()
		.map(|v| {
			let at = if span > 0.0 {
				((v - lo) / span * top).round()
			} else {
				0.0
			};
			SPARKS[at.clamp(0.0, top) as usize]
		})
		.collect()
}

/// A prop as text (a string, or spans joined).
fn text<M>(node: &Node<M>, key: &str) -> Option<String> {
	node.text_prop(key).filter(|t| !t.is_empty())
}

/// A prop as a string.
fn string<'a, M>(node: &'a Node<M>, key: &str) -> Option<&'a str> {
	node.props.get(key).and_then(Value::as_str)
}

/// A prop as a number.
fn number<M>(node: &Node<M>, key: &str) -> Option<f64> {
	node.props.get(key).and_then(Value::as_f64)
}

/// A JSON value (string or spans) as text.
fn value_text(value: &Value) -> String {
	match value {
		Value::String(s) => s.clone(),
		Value::Array(spans) => spans_text(spans),
		Value::Number(n) => n.to_string(),
		Value::Bool(b) => b.to_string(),
		_ => String::new(),
	}
}

/// A list prop.
fn list<'a, M>(node: &'a Node<M>, key: &str) -> &'a [Value] {
	node
		.props
		.get(key)
		.and_then(Value::as_array)
		.map_or_default(Vec::as_slice)
}

/// A node's primary text: `spans` (with their cite marks) when they are a
/// list, else `text`.
fn primary<M>(node: &Node<M>) -> Option<String> {
	match node.props.get("spans") {
		Some(Value::Array(spans)) => Some(cited_text(spans)),
		_ => text(node, "text"),
	}
}

/// The cites of a span that have an `href` (one cite, or a list).
fn cites(span: &Value) -> Vec<&Map<String, Value>> {
	let all: Vec<&Value> = match span.get("cite") {
		Some(Value::Array(list)) => list.iter().collect(),
		Some(one) => vec![one],
		None => Vec::new(),
	};
	all.into_iter()
		.filter_map(Value::as_object)
		.filter(|c| c.get("href").is_some_and(Value::is_string))
		.collect()
}

/// Spans joined, each cited run followed by its sources' numbers (`[1][2]`):
/// a cite's own `n`, else the next free number by first appearance.
fn cited_text(spans: &[Value]) -> String {
	use std::fmt::Write as _;
	fn href(c: &Map<String, Value>) -> &str {
		c.get("href").and_then(Value::as_str).unwrap_or_default()
	}
	let mut numbers: Vec<(String, u64)> = Vec::new();
	for c in spans.iter().flat_map(cites) {
		if let Some(n) = c.get("n").and_then(Value::as_f64).filter(|n| *n >= 1.0)
			&& !numbers.iter().any(|(h, _)| h == href(c))
		{
			numbers.push((href(c).to_owned(), n.floor() as u64));
		}
	}
	let mut taken: Vec<u64> = numbers.iter().map(|(_, n)| *n).collect();
	let mut next = 1;
	let mut out = String::new();
	for span in spans {
		out.push_str(&spans_text(std::slice::from_ref(span)));
		let mut marks: Vec<u64> = Vec::new();
		for c in cites(span) {
			let n = if let Some((_, n)) = numbers.iter().find(|(h, _)| h == href(c)) {
				*n
			} else {
				while taken.contains(&next) {
					next += 1;
				}
				taken.push(next);
				numbers.push((href(c).to_owned(), next));
				next
			};
			if !marks.contains(&n) {
				marks.push(n);
			}
		}
		for n in marks {
			let _ = write!(out, "[{n}]");
		}
	}
	out
}

/// A number as JavaScript prints it (`72`, `0.6`), or the value as text.
fn num_text(value: &Value) -> String {
	value
		.as_f64()
		.map_or_else(|| value_text(value), |n| n.to_string())
}

/// `value` then `unit`: against it when `attached`, else after a space.
fn with_unit(value: &str, unit: &str, attached: bool) -> String {
	if unit.is_empty() {
		value.to_owned()
	} else if attached {
		format!("{value}{unit}")
	} else {
		format!("{value} {unit}")
	}
}

/// `label: shown`, or `shown` without a label.
fn labeled(label: Option<String>, shown: String) -> String {
	match label {
		Some(label) => format!("{label}: {shown}"),
		None => shown,
	}
}

/// A `calendar`'s or `date`'s value: a day, or a range as `from – to` (an
/// open range as its start).
fn date_text(value: &Value) -> String {
	match value {
		Value::String(day) => day.clone(),
		Value::Array(ends) => {
			let from = ends.first().and_then(Value::as_str).unwrap_or_default();
			match ends.get(1).and_then(Value::as_str) {
				Some(to) => format!("{from} – {to}"),
				None => from.to_owned(),
			}
		},
		_ => String::new(),
	}
}

/// The numbers of chart series entries (bare or `{value}`), gaps dropped.
fn chart_values(entries: &[Value]) -> Vec<f64> {
	entries
		.iter()
		.filter_map(|e| e.get("value").unwrap_or(e).as_f64())
		.collect()
}

/// A chart's lines: its `summary`, else a `line`/`area` chart as a
/// sparkline per series and a `donut` as its segments.
fn chart<M>(node: &Node<M>) -> Vec<String> {
	if let Some(summary) = text(node, "summary") {
		return vec![summary];
	}
	let series = list(node, "series");
	match string(node, "kind") {
		Some("line" | "area") => {
			let several: Vec<&Value> = series
				.iter()
				.filter(|s| s.get("values").is_some_and(Value::is_array))
				.collect();
			if several.is_empty() {
				return vec![sparkline(&chart_values(series), None, None)];
			}
			several
				.into_iter()
				.map(|s| {
					let values = s
						.get("values")
						.and_then(Value::as_array)
						.map_or_default(Vec::as_slice);
					let spark = sparkline(&chart_values(values), None, None);
					match s.get("label").map(value_text) {
						Some(label) if !label.is_empty() => format!("{label} {spark}"),
						_ => spark,
					}
				})
				.collect()
		},
		Some("donut") => {
			let unit = string(node, "unit").unwrap_or_default();
			let attached = unit.starts_with(['%', '×', '°']);
			vec![
				series
					.iter()
					.filter_map(|entry| {
						let value = entry.get("value").unwrap_or(entry).as_f64()?;
						let shown = if CURRENCY.contains(&unit) {
							// A currency symbol leads the number, after any minus
							// sign.
							let sign = if value < 0.0 { "−" } else { "" };
							format!("{sign}{unit}{}", value.abs())
						} else {
							with_unit(&value.to_string(), unit, attached)
						};
						Some(match entry.get("label").map(value_text) {
							Some(label) if !label.is_empty() => format!("{label} {shown}"),
							_ => shown,
						})
					})
					.collect::<Vec<_>>()
					.join("  "),
			]
		},
		_ => Vec::new(),
	}
}

/// `[####------] 40%` for `value` in 0–1.
fn bar(value: f64) -> String {
	let v = value.clamp(0.0, 1.0);
	let filled = (v * BAR as f64).round() as usize;
	format!("[{}{}] {:.0}%", "#".repeat(filled), "-".repeat(BAR - filled), v * 100.0)
}

/// A duration in ms as `1h 2m`, `3m 4s` or `5s`.
fn duration(ms: f64) -> String {
	let s = (ms.abs() / 1000.0).floor() as u64;
	let (h, m, s) = (s / 3600, s / 60 % 60, s % 60);
	if h > 0 {
		format!("{h}h {m}m")
	} else if m > 0 {
		format!("{m}m {s}s")
	} else {
		format!("{s}s")
	}
}

/// Removes escape sequences and other control characters but `\n` and
/// `\t`.
fn strip(text: &str) -> String {
	let mut out = String::with_capacity(text.len());
	let mut chars = text.chars().peekable();
	while let Some(c) = chars.next() {
		match c {
			'\x1b' => match chars.next() {
				Some('[') => {
					for c in chars.by_ref() {
						if ('\x40'..='\x7e').contains(&c) {
							break;
						}
					}
				},
				Some(']' | '_' | 'P' | '^') => {
					while let Some(c) = chars.next() {
						if c == '\x07' || (c == '\x1b' && chars.next_if_eq(&'\\').is_some()) {
							break;
						}
					}
				},
				_ => {},
			},
			'\n' | '\t' => out.push(c),
			'\r' => {},
			c if c.is_control() => {},
			c => out.push(c),
		}
	}
	out
}

/// Pushes `text`'s lines (a final newline ends the last line).
fn push_text(text: &str, out: &mut Vec<String>) {
	out.extend(
		text
			.strip_suffix('\n')
			.unwrap_or(text)
			.split('\n')
			.map(str::to_owned),
	);
}

/// Rows of cells aligned in columns separated by two spaces; `right` marks
/// columns aligned to the end.
fn columns(rows: &[Vec<String>], right: &[bool], out: &mut Vec<String>) {
	let n = rows.iter().map(Vec::len).max().unwrap_or(0);
	let mut widths = vec![0; n];
	for row in rows {
		for (w, cell) in widths.iter_mut().zip(row) {
			*w = (*w).max(cell.chars().count());
		}
	}
	for row in rows {
		let mut line = String::new();
		for (i, cell) in row.iter().enumerate() {
			if i > 0 {
				line.push_str("  ");
			}
			let pad = widths[i] - cell.chars().count();
			if right.get(i).copied().unwrap_or(false) {
				line.push_str(&" ".repeat(pad));
				line.push_str(cell);
			} else {
				line.push_str(cell);
				line.push_str(&" ".repeat(pad));
			}
		}
		out.push(line);
	}
}

/// Pushes `inner` indented by two spaces.
fn indented(inner: Vec<String>, out: &mut Vec<String>) {
	out.extend(
		inner
			.into_iter()
			.map(|l| if l.is_empty() { l } else { format!("  {l}") }),
	);
}

/// Pushes `head` over `body` indented, or `body` alone without a head.
fn headed(head: String, body: Vec<String>, out: &mut Vec<String>) {
	if head.is_empty() {
		out.extend(body);
	} else {
		out.push(head);
		indented(body, out);
	}
}

/// A node's lines.
fn block<M>(node: &Node<M>, cols: usize, out: &mut Vec<String>) {
	if node.props.get("hidden") == Some(&Value::Bool(true)) {
		return;
	}
	match node.kind.as_str() {
		"col" => node.children.iter().for_each(|c| block(c, cols, out)),
		"status" => out.push(inline(node)),
		"row" => {
			let line = inline(node);
			if !line.is_empty() {
				out.push(line);
			}
		},
		"card" | "section" | "overlay" => {
			let mut head = panel_head(node);
			if let Some(status) = string(node, "status") {
				head = if head.is_empty() {
					format!("({status})")
				} else {
					format!("{head} ({status})")
				};
			}
			let mut inner = Vec::new();
			node
				.children
				.iter()
				.filter(|c| slot_of(node, c).is_none())
				.for_each(|c| block(c, cols, &mut inner));
			if let Some(foot) = slot(node, "foot") {
				block(foot, cols, &mut inner);
			}
			if head.is_empty() {
				out.extend(inner);
			} else {
				out.push(head);
				indented(inner, out);
			}
		},
		"rule" => {
			let width = cols.clamp(4, 80);
			let label = [text(node, "label"), text(node, "time")]
				.into_iter()
				.flatten()
				.collect::<Vec<_>>()
				.join(" · ");
			if label.is_empty() {
				out.push("-".repeat(width));
			} else {
				let side = width.saturating_sub(label.chars().count() + 2) / 2;
				out.push(format!("{} {label} {}", "-".repeat(side), "-".repeat(side)));
			}
		},
		"spacer" => out.push(String::new()),
		"ansi" => push_text(&strip(&text(node, "text").unwrap_or_default()), out),
		"rows" => {
			for line in list(node, "lines") {
				if let Some(line) = line.as_str() {
					out.push(strip(line));
				}
			}
		},
		"diff" => match node.props.get("hunks") {
			Some(Value::Array(hunks)) => {
				for hunk in hunks {
					for line in hunk
						.get("lines")
						.and_then(Value::as_array)
						.into_iter()
						.flatten()
					{
						out.push(line.as_str().unwrap_or_default().to_owned());
					}
				}
			},
			_ => push_text(&text(node, "text").unwrap_or_default(), out),
		},
		"editor" | "input" => {
			let prompt = text(node, "prompt").unwrap_or_default();
			let secret = node.props.get("secret") == Some(&Value::Bool(true));
			let body = text(node, "text")
				.map(|t| {
					if secret {
						"•".repeat(t.encode_utf16().count())
					} else {
						t
					}
				})
				.or_else(|| text(node, "placeholder"))
				.unwrap_or_default();
			push_text(&format!("{prompt}{body}"), out);
		},
		"chart" => out.extend(chart(node)),
		"kv" => {
			let rows: Vec<Vec<String>> = list(node, "items")
				.iter()
				.map(|i| {
					vec![i.get("k").map_or_default(value_text), i.get("v").map_or_default(value_text)]
				})
				.collect();
			if string(node, "layout") == Some("inline") {
				if !rows.is_empty() {
					out.push(
						rows
							.iter()
							.map(|row| row.join(" "))
							.collect::<Vec<_>>()
							.join(" · "),
					);
				}
			} else {
				columns(&rows, &[], out);
			}
		},
		"table" => table(node, out),
		"tree" => tree(list(node, "nodes"), 0, out),
		"json" => {
			let value = node.props.get("value").unwrap_or(&Value::Null);
			if let Ok(text) = serde_json::to_string_pretty(value) {
				out.extend(text.lines().map(str::to_owned));
			}
		},
		"list" => {
			for child in &node.children {
				out.push(format!("- {}", inline(child)));
			}
			if node.children.is_empty()
				&& let Some(empty) = text(node, "empty")
			{
				out.push(empty);
			}
		},
		"item" => out.push(format!("- {}", inline(node))),
		"picker" => {
			if let Some(title) = text(node, "title") {
				out.push(title);
			}
			for item in list(node, "items") {
				let label = item.get("label").map_or_default(value_text);
				out.push(format!("- {label}"));
			}
		},
		"checklist" => {
			for phase in list(node, "phases") {
				if let Some(title) = phase.get("title").map(value_text) {
					out.push(title);
				}
				for item in phase
					.get("items")
					.and_then(Value::as_array)
					.into_iter()
					.flatten()
				{
					let done = match item.get("status").and_then(Value::as_str) {
						Some("done") => "[x]",
						Some("active") => "[>]",
						Some("dropped") => "[-]",
						Some("blocked") => "[!]",
						_ => "[ ]",
					};
					let text = item.get("text").map_or_default(value_text);
					out.push(format!("  {done} {text}"));
				}
			}
		},
		"prefs" => prefs(node, out),
		"tool" | "agent" => {
			out.push(inline(node));
			let mut inner = Vec::new();
			node
				.children
				.iter()
				.for_each(|c| block(c, cols, &mut inner));
			indented(inner, out);
		},
		"select" => {
			let shown = match node.props.get("value") {
				None => text(node, "placeholder").unwrap_or_default(),
				Some(value) => list(node, "options")
					.iter()
					.find(|o| o.get("value") == Some(value))
					.and_then(|o| o.get("label").map(value_text))
					.filter(|l| !l.is_empty())
					.unwrap_or_else(|| value_text(value)),
			};
			out.push(labeled(text(node, "label"), shown));
		},
		"switch" => {
			let on = node.props.get("on") == Some(&Value::Bool(true));
			let label = text(node, "label").unwrap_or_default();
			out.push(format!("[{}] {label}", if on { 'x' } else { ' ' }));
		},
		"slider" => {
			let label = text(node, "label");
			let shown = match node.props.get("value") {
				None => None,
				Some(Value::Array(range)) => Some(
					range
						.iter()
						.filter_map(Value::as_f64)
						.map(|v| v.to_string())
						.collect::<Vec<_>>()
						.join("–"),
				),
				Some(value) => Some(num_text(value)),
			};
			out.push(match shown {
				Some(shown) => {
					let unit = string(node, "unit").unwrap_or_default();
					labeled(label, with_unit(&shown, unit, SLIDER_UNITS.contains(&unit)))
				},
				None => label.unwrap_or_default(),
			});
		},
		"field" => {
			let mut head = text(node, "label").unwrap_or_default();
			if !head.is_empty() && node.props.get("required") == Some(&Value::Bool(true)) {
				head.push_str(" *");
			}
			let flagged = ["error", "warning"].iter().find_map(|k| {
				node
					.props
					.get(*k)
					.filter(|v| !matches!(v, Value::Null | Value::Bool(false)) && v.as_str() != Some(""))
			});
			let note = match flagged {
				None => text(node, "hint").unwrap_or_default(),
				Some(Value::Bool(true)) => String::new(),
				Some(flag) => value_text(flag),
			};
			if !head.is_empty() {
				out.push(head);
			}
			node.children.iter().for_each(|c| block(c, cols, out));
			if !note.is_empty() {
				out.push(note);
			}
		},
		"otp" => {
			let length = number(node, "length").unwrap_or(6.0).clamp(1.0, 12.0) as usize;
			let typed: Vec<char> = string(node, "text")
				.unwrap_or_default()
				.chars()
				.take(length)
				.collect();
			let dot = match node.props.get("mask") {
				Some(Value::Bool(true)) => Some('•'),
				Some(Value::String(mask)) => mask.chars().next(),
				_ => None,
			};
			let mut code: String = typed.iter().map(|c| dot.unwrap_or(*c)).collect();
			code.extend(std::iter::repeat_n('_', length - typed.len()));
			out.push(code);
		},
		"buttons" => out.push(
			node
				.children
				.iter()
				.map(inline)
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" "),
		),
		"avatar" => out.push(text(node, "text").unwrap_or_default()),
		"avatars" => {
			let names = list(node, "items")
				.iter()
				.filter_map(|i| i.get("text").map(value_text))
				.filter(|t| !t.is_empty())
				.collect::<Vec<_>>()
				.join(", ");
			out.push(
				[names, text(node, "label").unwrap_or_default()]
					.into_iter()
					.filter(|s| !s.is_empty())
					.collect::<Vec<_>>()
					.join(" "),
			);
		},
		"breadcrumb" => {
			let sep = if string(node, "variant") == Some("path") {
				" / "
			} else {
				" › "
			};
			out.push(
				list(node, "items")
					.iter()
					.filter_map(|crumb| {
						let id = crumb
							.get("id")
							.filter(|id| id.is_string() || id.is_number())?;
						Some(
							crumb
								.get("label")
								.map(value_text)
								.filter(|l| !l.is_empty())
								.unwrap_or_else(|| value_text(id)),
						)
					})
					.collect::<Vec<_>>()
					.join(sep),
			);
		},
		"pager" => {
			let pages = number(node, "pages").filter(|p| *p != 0.0).or_else(|| {
				let total = number(node, "total")?;
				let size = number(node, "size").filter(|s| *s != 0.0)?;
				Some((total / size).ceil().max(1.0))
			});
			let page = number(node, "page")
				.unwrap_or(1.0)
				.min(pages.unwrap_or(f64::INFINITY))
				.max(1.0);
			out.push(match pages {
				Some(pages) => format!("Page {page} of {pages}"),
				None => format!("Page {page}"),
			});
		},
		"empty" => {
			let mut lines: Vec<String> = [text(node, "title"), text(node, "text")]
				.into_iter()
				.flatten()
				.collect();
			node
				.children
				.iter()
				.for_each(|c| block(c, cols, &mut lines));
			out.extend(lines.into_iter().filter(|l| !l.is_empty()));
		},
		"skeleton" => {},
		"calendar" | "date" => match node.props.get("value") {
			Some(value) => out.push(date_text(value)),
			None => {
				if node.kind == "date"
					&& let Some(placeholder) = text(node, "placeholder")
				{
					out.push(placeholder);
				}
			},
		},
		"menu" => out.push(format!(
			"[{}]",
			text(node, "text")
				.or_else(|| text(node, "title"))
				.unwrap_or_default()
		)),
		"menubar" => out.push(
			list(node, "menus")
				.iter()
				.filter_map(|menu| {
					let id = menu.get("id")?.as_str()?;
					Some(
						menu
							.get("label")
							.map(value_text)
							.filter(|l| !l.is_empty())
							.unwrap_or_else(|| id.to_owned()),
					)
				})
				.collect::<Vec<_>>()
				.join("  "),
		),
		"nav" => {
			let active = node.props.get("active");
			out.push(
				list(node, "items")
					.iter()
					.filter_map(|item| {
						let id = item.get("id").filter(|id| id.is_string())?;
						let label = item
							.get("label")
							.map(value_text)
							.filter(|l| !l.is_empty())
							.unwrap_or_else(|| value_text(id));
						let on = active == Some(id)
							|| item
								.get("items")
								.and_then(Value::as_array)
								.is_some_and(|subs| subs.iter().any(|s| s.get("id") == active));
						Some(if on { format!("[{label}]") } else { label })
					})
					.collect::<Vec<_>>()
					.join("  "),
			);
		},
		"split" | "scroll" => node.children.iter().for_each(|c| block(c, cols, out)),
		"carousel" => {
			if let Some(last) = node.children.len().checked_sub(1) {
				let at = number(node, "index").unwrap_or(0.0).clamp(0.0, last as f64) as usize;
				block(&node.children[at], cols, out);
			}
		},
		"message" => {
			let head = [text(node, "author"), text(node, "time")]
				.into_iter()
				.flatten()
				.collect::<Vec<_>>()
				.join(" ");
			let mut rest = Vec::new();
			node
				.children
				.iter()
				.filter(|c| slot_of(node, c).is_none())
				.for_each(|c| block(c, cols, &mut rest));
			let reactions = list(node, "reactions")
				.iter()
				.filter_map(|r| {
					let text = r.get("text").map(value_text).filter(|t| !t.is_empty())?;
					Some(match r.get("count").and_then(Value::as_f64) {
						Some(count) if count > 0.0 => format!("{text} {count}"),
						_ => text,
					})
				})
				.collect::<Vec<_>>()
				.join("  ");
			if !reactions.is_empty() {
				rest.push(reactions);
			}
			if let Some(status) = text(node, "status") {
				rest.push(status);
			}
			if !head.is_empty() {
				out.push(head);
			}
			indented(rest, out);
		},
		"attachment" => {
			out.push(format!("[attachment: {}]", string(node, "name").unwrap_or_default()));
		},
		"trace" => {
			let status = string(node, "status").map_or_default(|s| format!("({s})"));
			let title = text(node, "title")
				.or_else(|| text(node, "summary"))
				.unwrap_or_default();
			let head = [title, status]
				.into_iter()
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
			if !head.is_empty() {
				out.push(head);
			}
			let key = |c: &Node<M>| c.props.get("key").map(value_text);
			let mut ids = Vec::new();
			for (i, line) in list(node, "lines").iter().enumerate() {
				let id = line
					.get("id")
					.map(value_text)
					.filter(|id| !id.is_empty())
					.unwrap_or_else(|| i.to_string());
				let words = [line.get("title"), line.get("target")]
					.into_iter()
					.flatten()
					.map(value_text)
					.filter(|s| !s.is_empty())
					.collect::<Vec<_>>()
					.join(" ");
				out.push(match line.get("status").and_then(Value::as_str) {
					Some(state) => format!("- {words} ({state})"),
					None => format!("- {words}"),
				});
				if let Some(content) = node.children.iter().find(|c| key(c).as_ref() == Some(&id)) {
					let mut under = Vec::new();
					block(content, cols, &mut under);
					out.extend(
						under
							.into_iter()
							.map(|l| if l.is_empty() { l } else { format!("    {l}") }),
					);
				}
				ids.push(id);
			}
			node
				.children
				.iter()
				.filter(|c| !key(c).is_some_and(|k| ids.contains(&k)))
				.for_each(|c| block(c, cols, out));
		},
		"stacktrace" => {
			let error = node.props.get("error");
			let head = ["type", "message"]
				.iter()
				.filter_map(|k| error?.get(*k).map(value_text))
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(": ");
			if !head.is_empty() {
				out.push(head);
			}
			for frame in list(node, "frames") {
				let func = frame
					.get("fn")
					.map(value_text)
					.filter(|f| !f.is_empty())
					.unwrap_or_else(|| "<anonymous>".to_owned());
				let file = frame.get("file").map_or_default(value_text);
				out.push(if file.is_empty() {
					format!("  at {func}")
				} else {
					let at =
						[Some(file), frame.get("line").map(num_text), frame.get("col").map(num_text)]
							.into_iter()
							.flatten()
							.collect::<Vec<_>>()
							.join(":");
					format!("  at {func} ({at})")
				});
			}
		},
		"suggestions" => {
			for item in list(node, "items") {
				let text = item.get("text").map_or_default(value_text);
				if !text.is_empty() {
					out.push(format!("- {text}"));
				}
			}
		},
		"questions" => {
			for item in list(node, "items") {
				if !item.get("id").is_some_and(Value::is_string) {
					continue;
				}
				let choices: Vec<String> = item
					.get("choices")
					.and_then(Value::as_array)
					.map_or_default(Vec::as_slice)
					.iter()
					.map(|choice| {
						let label = choice
							.get("label")
							.map(value_text)
							.filter(|l| !l.is_empty())
							.unwrap_or_else(|| {
								choice
									.get("value")
									.map_or_else(|| value_text(choice), value_text)
							});
						format!("- {label}")
					})
					.collect();
				headed(item.get("prompt").map_or_default(value_text), choices, out);
			}
		},
		"el" => el_block(node, cols, out),
		_ => {
			let line = inline(node);
			if !line.is_empty() {
				push_text(&line, out);
			}
			node.children.iter().for_each(|c| block(c, cols, out));
		},
	}
}

/// A node as one line of text.
fn inline<M>(node: &Node<M>) -> String {
	if node.props.get("hidden") == Some(&Value::Bool(true)) {
		return String::new();
	}
	let joined = |sep: &str| {
		node
			.children
			.iter()
			.map(inline)
			.filter(|s| !s.is_empty())
			.collect::<Vec<_>>()
			.join(sep)
	};
	let parts: Vec<String> = match node.kind.as_str() {
		"card" | "section" | "overlay" => {
			let body = node
				.children
				.iter()
				.filter(|c| slot_of(node, c).is_none())
				.map(inline)
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
			return [panel_head(node), body]
				.into_iter()
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
		},
		"row" | "col" => {
			let head = text(node, "head").unwrap_or_default();
			return [head, joined(" ")]
				.into_iter()
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
		},
		"status" => return joined("  "),
		"badge" | "button" => vec![text(node, "text").map_or_default(|t| format!("[{t}]"))],
		"stats" => {
			return list(node, "items")
				.iter()
				.map(|item| {
					let label = item.get("label").map_or_default(value_text);
					let value = item.get("value").map_or_default(value_text);
					match item.get("sub").map(value_text) {
						Some(sub) if !sub.is_empty() => format!("{label}: {value} ({sub})"),
						_ => format!("{label}: {value}"),
					}
				})
				.collect::<Vec<_>>()
				.join("  ·  ");
		},
		"kbd" => vec![
			list(node, "keys")
				.iter()
				.map(value_text)
				.collect::<Vec<_>>()
				.join("+"),
		],
		"icon" => vec![string(node, "aria").unwrap_or_default().to_owned()],
		"image" => vec![format!("[image: {}]", string(node, "alt").unwrap_or("image"))],
		"media" => vec![match string(node, "rev") {
			Some(rev) => format!("[media: {} @ {rev}]", string(node, "path").unwrap_or("media")),
			None => format!("[media: {}]", string(node, "path").unwrap_or("media")),
		}],
		"spinner" => vec![text(node, "label").unwrap_or_else(|| "...".into())],
		"elapsed" => {
			vec![duration(
				number(node, "stopped")
					.or_else(|| number(node, "age"))
					.unwrap_or(0.0),
			)]
		},
		"rate" => {
			let v = number(node, "value").unwrap_or(0.0);
			vec![format!("{v}"), string(node, "unit").unwrap_or_default().to_owned()]
		},
		"progress" | "meter" => {
			let value = number(node, "value").or_else(|| {
				let parts = list(node, "parts");
				(!parts.is_empty()).then(|| parts.iter().filter_map(|p| p.get("value")?.as_f64()).sum())
			});
			vec![
				value.map_or_else(|| "[..........]".to_owned(), bar),
				text(node, "label").unwrap_or_default(),
				text(node, "total").unwrap_or_default(),
			]
		},
		"chart" | "select" | "switch" | "slider" | "field" | "otp" | "buttons" | "avatar"
		| "avatars" | "breadcrumb" | "pager" | "empty" | "skeleton" | "calendar" | "date"
		| "menu" | "menubar" | "nav" | "split" | "scroll" | "carousel" | "message" | "attachment"
		| "trace" | "stacktrace" | "suggestions" | "questions" => {
			let mut lines = Vec::new();
			block(node, 80, &mut lines);
			return lines
				.iter()
				.map(|l| l.trim())
				.filter(|l| !l.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
		},
		"effort" => vec![string(node, "level").unwrap_or_default().to_owned()],
		"item" => vec![
			text(node, "label").unwrap_or_default(),
			text(node, "detail").unwrap_or_default(),
			text(node, "value").unwrap_or_default(),
		],
		"tabs" => {
			let active = string(node, "active");
			vec![
				list(node, "items")
					.iter()
					.map(|t| {
						let label = t.get("label").map_or_default(value_text);
						if t.get("id").and_then(Value::as_str) == active {
							format!("[{label}]")
						} else {
							label
						}
					})
					.collect::<Vec<_>>()
					.join("  "),
			]
		},
		"seg" => vec![primary(node).unwrap_or_default(), joined(" ")],
		"toast" => {
			vec![text(node, "text").unwrap_or_default(), text(node, "sub").unwrap_or_default()]
		},
		"tool" => {
			let mut parts = vec![
				text(node, "title")
					.or_else(|| text(node, "name"))
					.unwrap_or_default(),
				text(node, "target").unwrap_or_default(),
			];
			parts.extend(list(node, "meta").iter().map(value_text));
			parts.push(string(node, "status").map_or_default(|s| format!("({s})")));
			parts
		},
		"agent" => vec![
			text(node, "name").unwrap_or_default(),
			text(node, "task").unwrap_or_default(),
			string(node, "status").map_or_default(|s| format!("({s})")),
		],
		"el" => return el_inline(node),
		_ => vec![primary(node).unwrap_or_default(), joined(" ")],
	};
	parts
		.into_iter()
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>()
		.join(" ")
}

/// A `table` as aligned columns.
fn table<M>(node: &Node<M>, out: &mut Vec<String>) {
	let cols: Vec<&Map<String, Value>> = list(node, "cols")
		.iter()
		.filter_map(Value::as_object)
		.collect();
	let right: Vec<bool> = cols
		.iter()
		.map(|c| c.get("align").and_then(Value::as_str) == Some("end"))
		.collect();
	let mut rows = Vec::new();
	if cols
		.iter()
		.any(|c| c.get("head").is_some_and(|h| !value_text(h).is_empty()))
	{
		rows.push(
			cols
				.iter()
				.map(|c| c.get("head").map_or_default(value_text))
				.collect(),
		);
	}
	for row in list(node, "rows") {
		let cells = row.get("cells");
		rows.push(
			cols
				.iter()
				.map(|c| {
					let id = c.get("id").and_then(Value::as_str).unwrap_or_default();
					cells
						.and_then(|cells| cells.get(id))
						.map_or_default(cell_text)
				})
				.collect(),
		);
	}
	columns(&rows, &right, out);
	if list(node, "rows").is_empty()
		&& let Some(empty) = text(node, "empty")
	{
		out.push(empty);
	}
}

/// A table cell as text: meters as bars, sparklines as blocks, badges and
/// buttons bracketed.
fn cell_text(cell: &Value) -> String {
	let Value::Object(cell) = cell else {
		return value_text(cell);
	};
	if let Some(meter) = cell.get("meter") {
		return meter
			.get("value")
			.and_then(Value::as_f64)
			.map_or_default(bar);
	}
	if let Some(spark) = cell.get("spark") {
		let values: Vec<f64> = spark
			.get("values")
			.and_then(Value::as_array)
			.map_or_default(|v| v.iter().filter_map(Value::as_f64).collect());
		let bound = |k| spark.get(k).and_then(Value::as_f64);
		return sparkline(&values, bound("min"), bound("max"));
	}
	if let Some(chip) = cell.get("badge").or_else(|| cell.get("button")) {
		return chip
			.get("text")
			.map_or_default(|t| format!("[{}]", value_text(t)));
	}
	cell.get("text").map_or_default(value_text)
}

/// The child filling slot `name` (`head`, `end`, `foot`) of a panel.
fn slot<'a, M>(node: &'a Node<M>, name: &str) -> Option<&'a Node<M>> {
	if !node.slots.contains(&name) {
		return None;
	}
	node
		.children
		.iter()
		.find(|c| c.props.get("key").and_then(Value::as_str) == Some(name))
}

/// The slot `child` fills in `node`, if any.
fn slot_of<'a, M>(node: &'a Node<M>, child: &Node<M>) -> Option<&'a str> {
	let key = child.props.get("key").and_then(Value::as_str)?;
	node.slots.iter().copied().find(|s| *s == key)
}

/// A panel's head line: the head (text or slotted node), its `meta` and
/// its `end` node.
fn panel_head<M>(node: &Node<M>) -> String {
	let head = slot(node, "head").map_or_else(|| text(node, "head").unwrap_or_default(), inline);
	[head, text(node, "meta").unwrap_or_default(), slot(node, "end").map_or_default(inline)]
		.into_iter()
		.filter(|s| !s.is_empty())
		.collect::<Vec<_>>()
		.join("  ")
}

/// `tree` items as nested bullets.
fn tree(items: &[Value], depth: usize, out: &mut Vec<String>) {
	for item in items {
		let label = item.get("label").map_or_default(value_text);
		out.push(format!("{}- {label}", "  ".repeat(depth)));
		if let Some(Value::Array(children)) = item.get("children") {
			tree(children, depth + 1, out);
		}
	}
}

/// A `prefs` page: its title, then each section's rows as `label: value`.
fn prefs<M>(node: &Node<M>, out: &mut Vec<String>) {
	out.push(string(node, "title").unwrap_or("Settings").to_owned());
	for section in list(node, "sections") {
		if let Some(title) = section.get("title").and_then(Value::as_str) {
			out.push(format!("  {title}"));
		}
		for row in section
			.get("rows")
			.and_then(Value::as_array)
			.into_iter()
			.flatten()
		{
			let id = row.get("id").and_then(Value::as_str).unwrap_or_default();
			let label = row.get("label").and_then(Value::as_str).unwrap_or(id);
			let value = row.get("control").map_or_default(|c| {
				["on", "value", "values", "label"]
					.iter()
					.find_map(|k| c.get(*k))
					.map_or_default(|v| match v {
						Value::Array(vs) => vs.iter().map(value_text).collect::<Vec<_>>().join(", "),
						Value::Bool(b) => if *b { "on" } else { "off" }.to_owned(),
						other => value_text(other),
					})
			});
			out.push(format!("    {label}: {value}"));
		}
	}
}

/// The tag of an `el` (`div` when absent).
fn tag<M>(node: &Node<M>) -> &str {
	string(node, "tag").unwrap_or("div")
}

/// Whether a child of an `el` starts a line of its own.
fn is_block<M>(node: &Node<M>) -> bool {
	node.kind != "el" || BLOCK_TAGS.contains(&tag(node))
}

/// An `el` (and its children) as one line.
fn el_inline<M>(node: &Node<M>) -> String {
	match tag(node) {
		"input" => {
			let on = node.props.get("checked") == Some(&Value::Bool(true));
			let radio = string(node, "type") == Some("radio");
			return match (radio, on) {
				(true, true) => "(*)",
				(true, false) => "( )",
				(false, true) => "[x]",
				(false, false) => "[ ]",
			}
			.to_owned();
		},
		"hr" => return "---".to_owned(),
		_ => {},
	}
	let mut parts = Vec::new();
	if let Some(t) = text(node, "text") {
		parts.push(t);
	}
	parts.extend(node.children.iter().map(inline).filter(|s| !s.is_empty()));
	let body = parts.join(" ");
	if tag(node) == "button" {
		format!("[ {body} ]")
	} else {
		body
	}
}

/// An `el` as lines: inline content joins, block children start lines.
fn el_block<M>(node: &Node<M>, cols: usize, out: &mut Vec<String>) {
	if node.props.get("hidden") == Some(&Value::Bool(true)) {
		return;
	}
	match tag(node) {
		"table" | "thead" | "tbody" => {
			let mut rows = Vec::new();
			collect_rows(node, &mut rows);
			columns(&rows, &[], out);
			return;
		},
		"tr" => {
			out.push(
				node
					.children
					.iter()
					.map(inline)
					.collect::<Vec<_>>()
					.join("  "),
			);
			return;
		},
		"hr" => {
			out.push("-".repeat(cols.clamp(4, 80)));
			return;
		},
		"label" | "button" => {
			out.push(el_inline(node));
			return;
		},
		_ => {},
	}
	let prefix = if tag(node) == "li" { "- " } else { "" };
	let mut inner = Vec::new();
	let mut line = text(node, "text").unwrap_or_default();
	for child in &node.children {
		if is_block(child) {
			if !line.is_empty() {
				inner.push(std::mem::take(&mut line));
			}
			if child.kind == "el" {
				el_block(child, cols, &mut inner);
			} else {
				block(child, cols, &mut inner);
			}
		} else {
			let part = inline(child);
			if !part.is_empty() {
				if !line.is_empty() && !line.ends_with(' ') {
					line.push(' ');
				}
				line.push_str(&part);
			}
		}
	}
	if !line.is_empty() {
		inner.push(line);
	}
	if matches!(tag(node), "ul" | "ol" | "blockquote" | "dd") {
		indented(inner, out);
	} else {
		for (i, l) in inner.into_iter().enumerate() {
			out.push(if i == 0 { format!("{prefix}{l}") } else { l });
		}
	}
}

/// The rows (`tr`) of an `el` table as cells.
fn collect_rows<M>(node: &Node<M>, rows: &mut Vec<Vec<String>>) {
	for child in &node.children {
		match (child.kind.as_str(), tag(child)) {
			("el", "tr") => rows.push(child.children.iter().map(inline).collect()),
			("el", "thead" | "tbody") => collect_rows(child, rows),
			_ => {},
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::ui::{self, Column, TableRow, TextAlign, html};

	#[test]
	fn cards_tables_and_lists_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::card()
				.head("Build")
				.status(ui::Status::Done)
				.child(ui::text("ok")),
			ui::table()
				.col(Column::new("n", "Name"))
				.col(Column::new("s", "Size").align(TextAlign::End))
				.row(TableRow::new("a").cell("n", "a.rs").cell("s", "12"))
				.row(TableRow::new("b").cell("n", "build.rs").cell("s", "3")),
			ui::list()
				.child(ui::item("one"))
				.child(ui::item("two").value("2")),
		]);
		assert_eq!(
			plain(view, 40),
			"Build (done)\n  ok\nName      Size\na.rs        12\nbuild.rs     3\n- one\n- two 2\n"
		);
	}

	#[test]
	fn headed_statuses_use_parentheses() {
		for kind in ["card", "section", "overlay"] {
			assert_eq!(
				plain(
					ui::node(kind)
						.prop("head", "cargo test")
						.prop("status", "done"),
					80
				),
				"cargo test (done)\n"
			);
			assert_eq!(plain(ui::node(kind).prop("status", "done"), 80), "(done)\n");
		}
		for kind in ["tool", "agent"] {
			assert_eq!(
				plain(
					ui::node(kind)
						.prop("name", "cargo test")
						.prop("status", "done"),
					80
				),
				"cargo test (done)\n"
			);
		}
	}

	#[test]
	fn el_blocks_break_lines_and_ansi_loses_escapes() {
		let view: View = View::new()
			.main([html::form()
				.child(html::p("Which size?"))
				.child(
					html::label()
						.child(html::radio("size", "s").checked(true))
						.child(html::span("Small")),
				)
				.child(html::button("Create"))])
			.dock([ui::ansi("\x1b[1;32mok\x1b[0m done\r\n")]);
		assert_eq!(plain(view, 40), "Which size?\n(*) Small\n[ Create ]\nok done\n");
	}

	#[test]
	fn bare_builder_infers_a_message_free_view() {
		assert_eq!(plain(ui::text("hello"), 80), "hello\n");
	}

	#[test]
	fn meters_inline_pairs_and_blocked_items_read_as_text() {
		let view = View::new().main(crate::nodes![
			ui::table().col(Column::new("usage", "Usage")).row(
				TableRow::new("a")
					.cell("usage", ui::MeterCell { value: Some(0.5), ..ui::MeterCell::default() })
			),
			ui::kv()
				.layout(ui::KvLayout::Inline)
				.item("branch", "main")
				.item("ahead", "2"),
			ui::checklist().prop(
				"phases",
				serde_json::json!([{"items": [
					{"text": "waiting", "status": "pending"},
					{"text": "working", "status": "active"},
					{"text": "stuck", "status": "blocked"},
					{"text": "finished", "status": "done"},
					{"text": "removed", "status": "dropped"}
				]}])
			),
		]);
		assert_eq!(
			plain(view, 80),
			"Usage\n[#####-----] 50%\nbranch main · ahead 2\n  [ ] waiting\n  [>] working\n  [!] \
			 stuck\n  [x] finished\n  [-] removed\n"
		);
	}

	#[test]
	fn panels_buttons_stats_and_rich_cells_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::card()
				.head("Jobs")
				.meta("3 running")
				.end(ui::button("Refresh").keys(["r"]))
				.child(ui::stats([
					ui::StatItem::new("Queued", "4").sub("2 new"),
					ui::StatItem::new("Failed", "1"),
				])),
			ui::table()
				.col(Column::new("n", "Name"))
				.col(Column::new("s", "State"))
				.col(Column::new("a", "Act").align(TextAlign::End))
				.col(Column::new("t", "Load"))
				.row(
					TableRow::new("a")
						.cell("t", ui::SparkCell::new(vec![0.0, 1.0, 2.0]).range(0.0, 2.0))
						.cell("n", ui::TextCell::new("api").dot(ui::Tone::Success))
						.cell("s", ui::BadgeSpec::new("live"))
						.cell("a", ui::ButtonCell::new("Approve", "approve")),
				),
			ui::table().col(Column::new("n", "Name")).empty("No jobs"),
			ui::overlay().head("Delete?").foot(ui::button("Delete")),
		]);
		assert_eq!(
			plain(view, 80),
			"Jobs  3 running  [Refresh]\n  Queued: 4 (2 new)  ·  Failed: 1\nName  State         Act  \
			 Load\napi   [live]  [Approve]  ▁▅█\nName\nNo jobs\nDelete?\n  [Delete]\n"
		);
	}

	#[test]
	fn form_controls_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::select()
				.label("Model")
				.value("haiku")
				.option(ui::SelectOption::new("haiku", "Haiku")),
			ui::select().placeholder("Pick one"),
			ui::switch("Wrap").on(true),
			ui::switch("Follow"),
			ui::slider().label("Opacity").value(72.0).unit("%"),
			ui::slider()
				.label("Latency")
				.value((200.0, 1200.0))
				.unit("ms"),
			ui::field("Name")
				.required(true)
				.hint("Shown to others")
				.error("Taken")
				.child(ui::input().text("cara")),
			ui::otp(6).text("123"),
			ui::otp(4).text("12").mask(true),
			ui::calendar().value(ui::DateValue::Range("2026-10-19".into(), Some("2026-10-30".into()))),
			ui::date().placeholder("Pick a date"),
			ui::menu().text("Release"),
			ui::input().text("hunter2").secret(true).reveal(true),
		]);
		assert_eq!(
			plain(view, 80),
			"Model: Haiku\nPick one\n[x] Wrap\n[ ] Follow\nOpacity: 72%\nLatency: 200–1200 ms\nName \
			 *\ncara\nTaken\n123___\n••__\n2026-10-19 – 2026-10-30\nPick a date\n[Release]\n•••••••\n"
		);
	}

	#[test]
	fn navigation_people_and_groups_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::breadcrumb()
				.crumb(ui::Crumb::new("home", "Home"))
				.crumb(ui::Crumb::new("src", "src")),
			ui::nav()
				.active("logs")
				.item(ui::NavItem::new("home", "Home"))
				.item(ui::NavItem::new("ops", "Ops").item(ui::NavSubItem::new("logs", "Logs"))),
			ui::menubar()
				.menu(ui::MenubarMenu::new("file", "File"))
				.menu(ui::MenubarMenu::new("edit", "Edit")),
			ui::pager().page(2u64).total(312u64).size(50u64),
			ui::avatars([ui::AvatarItem::new("Ann"), ui::AvatarItem::new("Bo")]).label("2 reviewers"),
			ui::buttons()
				.child(ui::button("Undo"))
				.child(ui::button("Redo")),
			ui::empty("No runs")
				.text("Start one")
				.child(ui::button("Run")),
			ui::skeleton(),
			ui::split().child(ui::text("left")).child(ui::text("right")),
			ui::carousel()
				.index(1u32)
				.child(ui::text("one"))
				.child(ui::text("two")),
			ui::rule().label("Resumed").time("14:02"),
		]);
		assert_eq!(
			plain(view, 24),
			"Home › src\nHome  [Ops]\nFile  Edit\nPage 2 of 7\nAnn, Bo 2 reviewers\n[Undo] \
			 [Redo]\nNo runs\nStart one\n[Run]\nleft\nright\ntwo\n--- Resumed · 14:02 ---\n"
		);
	}

	#[test]
	fn chat_and_work_kinds_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::message()
				.author("Cara")
				.time("14:02")
				.avatar(ui::avatar("Cara"))
				.reactions(vec![ui::Reaction::new("👍", 2), ui::Reaction::new("🎉", 0)])
				.status("Read")
				.child(ui::text("Shipped")),
			ui::attachment("report.pdf"),
			ui::trace()
				.title("Research")
				.status(ui::Status::Done)
				.line(
					ui::TraceLine::new("s1", "Searched")
						.target("tsp")
						.status(ui::Status::Done)
				)
				.child(ui::text("3 hits").key("s1")),
			ui::stacktrace(ui::StackError::new("TypeError", "x is undefined"))
				.frame(ui::Frame::new("main", "src/a.ts").at(3, 7))
				.frame(ui::Frame::new("", "")),
			ui::suggestions().item(ui::Suggestion::new("Explain this")),
			ui::questions().question(
				ui::Question::new("db", "Which database?")
					.choice("postgres")
					.choice(ui::QuestionChoice::new("sqlite", "SQLite"))
			),
		]);
		assert_eq!(
			plain(view, 80),
			"Cara 14:02\n  Shipped\n  👍 2  🎉\n  Read\n[attachment: report.pdf]\nResearch (done)\n- \
			 Searched tsp (done)\n    3 hits\nTypeError: x is undefined\n  at main (src/a.ts:3:7)\n  \
			 at <anonymous>\n- Explain this\nWhich database?\n  - postgres\n  - SQLite\n"
		);
	}

	#[test]
	fn charts_and_cites_read_as_text() {
		let view: View = View::new().main(crate::nodes![
			ui::chart(ui::ChartKind::Line).series(vec![
				ui::Point { value: 0.0, ..ui::Point::default() },
				ui::Point { value: 7.0, ..ui::Point::default() },
			]),
			ui::chart(ui::ChartKind::Area).multi_series(vec![
				ui::Series::new(vec![Some(0.0), None, Some(1.0)]).label("p50"),
				ui::Series::new(vec![Some(1.0), Some(0.0)]),
			]),
			ui::chart(ui::ChartKind::Donut).unit("ms").series(vec![
				ui::Point { value: 120.0, label: Some("api".into()), ..ui::Point::default() },
				ui::Point { value: 30.0, ..ui::Point::default() },
			]),
			ui::chart(ui::ChartKind::Donut)
				.unit("%")
				.series(vec![ui::Point { value: 64.0, ..ui::Point::default() }]),
			ui::chart(ui::ChartKind::Line).summary("p50 182 ms"),
			ui::text(crate::spans![
				ui::span("Rust"),
				ui::span(" ships").cite(ui::Cite::new("https://a.dev")),
				ui::span(" yearly")
					.cite(ui::Cite::new("https://b.dev").n(1))
					.cite(ui::Cite::new("https://a.dev")),
			]),
		]);
		assert_eq!(
			plain(view, 80),
			"▁█\np50 ▁█\n█▁\napi 120 ms  30 ms\n64%\np50 182 ms\nRust ships[2] yearly[1][2]\n"
		);
	}

	#[test]
	fn bars_clamp_and_round() {
		assert_eq!(bar(-1.0), "[----------] 0%");
		assert_eq!(bar(0.44), "[####------] 44%");
		assert_eq!(bar(2.0), "[##########] 100%");
	}
}
