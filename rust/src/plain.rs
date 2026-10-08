//! The plain-text fallback: a view rendered as readable text without
//! escapes, for output outside Tern. Text kinds are their text, rows join
//! on one line, cards and sections are a head line over an indented body,
//! tables and `kv` are aligned columns, lists are bullets, progress and
//! meters are `[####------] 40%`, motion kinds are their label, and `el` is
//! its text with block tags on lines of their own. It is not a copy of
//! Tern's layout.

use serde_json::{Map, Value};

use crate::ui::{Node, View, node::spans_text};

/// The width of a plain progress bar, in cells.
const BAR: usize = 10;

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
	let view = view.into();
	let mut lines = Vec::new();
	for node in view.regions().into_iter().flatten() {
		block(node, cols, &mut lines);
	}
	let mut out = String::new();
	for line in lines {
		out.push_str(line.trim_end());
		out.push('\n');
	}
	out
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
		.map_or(&[], Vec::as_slice)
}

/// A node's primary text: `spans` when they are a list, else `text`.
fn primary<M>(node: &Node<M>) -> Option<String> {
	match node.props.get("spans") {
		Some(Value::Array(spans)) => Some(spans_text(spans)),
		_ => text(node, "text"),
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
			let mut head = text(node, "head").unwrap_or_default();
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
				.for_each(|c| block(c, cols, &mut inner));
			if head.is_empty() {
				out.extend(inner);
			} else {
				out.push(head);
				indented(inner, out);
			}
		},
		"rule" => {
			let width = cols.clamp(4, 80);
			match text(node, "label") {
				Some(label) => {
					let side = width.saturating_sub(label.chars().count() + 2) / 2;
					out.push(format!("{} {label} {}", "-".repeat(side), "-".repeat(side)));
				},
				None => out.push("-".repeat(width)),
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
			let body = text(node, "text")
				.or_else(|| text(node, "placeholder"))
				.unwrap_or_default();
			push_text(&format!("{prompt}{body}"), out);
		},
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
		"row" | "col" | "card" | "section" | "overlay" => {
			let head = text(node, "head").unwrap_or_default();
			return [head, joined(" ")]
				.into_iter()
				.filter(|s| !s.is_empty())
				.collect::<Vec<_>>()
				.join(" ");
		},
		"status" => return joined("  "),
		"badge" => vec![text(node, "text").map_or_default(|t| format!("[{t}]"))],
		"kbd" => vec![
			list(node, "keys")
				.iter()
				.map(value_text)
				.collect::<Vec<_>>()
				.join("+"),
		],
		"icon" => vec![string(node, "aria").unwrap_or_default().to_owned()],
		"image" => vec![format!("[image: {}]", string(node, "alt").unwrap_or("image"))],
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
		"chart" => vec![text(node, "summary").unwrap_or_default()],
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
					match cells.and_then(|cells| cells.get(id)) {
						Some(Value::Object(cell)) => cell
							.get("meter")
							.and_then(|meter| meter.get("value"))
							.and_then(Value::as_f64)
							.map_or_default(bar),
						Some(cell) => value_text(cell),
						None => String::new(),
					}
				})
				.collect(),
		);
	}
	columns(&rows, &right, out);
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
			let value = row.get("control").map_or_else(String::new, |c| {
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
	fn bars_clamp_and_round() {
		assert_eq!(bar(-1.0), "[----------] 0%");
		assert_eq!(bar(0.44), "[####------] 44%");
		assert_eq!(bar(2.0), "[##########] 100%");
	}
}
