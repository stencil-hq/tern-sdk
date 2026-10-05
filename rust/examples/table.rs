//! `print`: the current directory as a table, sizes right-aligned by a
//! stylesheet. In Tern it stays in the scrollback as a native table;
//! anywhere else it prints as aligned plain text.

use std::fs;

use tern_sdk::{
	PrintOptions,
	ui::{View, html},
};

/// The table's look: header muted, sizes right-aligned.
const CSS: &str = ".ls{border-spacing:0} .ls th{text-align:left;padding:0 18px 3px \
                   0;color:var(--sf-c-muted);font-weight:500} .ls td{padding:1px 18px 1px 0} .ls \
                   .size{text-align:right} .ls .dir{color:var(--sf-c-accent)}";

/// A size as `312`, `2.1K`, `14K`, `3.4M`.
fn human(bytes: u64) -> String {
	const UNITS: [&str; 4] = ["K", "M", "G", "T"];
	if bytes < 1024 {
		return bytes.to_string();
	}
	let mut size = bytes as f64 / 1024.0;
	let mut unit = 0;
	while size >= 1024.0 && unit < UNITS.len() - 1 {
		size /= 1024.0;
		unit += 1;
	}
	if size < 10.0 {
		format!("{size:.1}{}", UNITS[unit])
	} else {
		format!("{size:.0}{}", UNITS[unit])
	}
}

/// One entry: its name, whether it is a directory, and its size.
struct Entry {
	/// The file name.
	name: String,
	/// A directory.
	dir:  bool,
	/// The size in bytes.
	size: u64,
}

/// The directory listing as a table.
fn view(entries: &[Entry]) -> View {
	let head = html::tr()
		.child(html::th("Name"))
		.child(html::th("Size").class("size"));
	let rows = entries.iter().map(|e| {
		let name = if e.dir {
			format!("{}/", e.name)
		} else {
			e.name.clone()
		};
		let size = if e.dir { "-".to_owned() } else { human(e.size) };
		html::tr()
			.child(html::td(name).class(if e.dir { "dir" } else { "file" }))
			.child(html::td(size).class("size"))
	});
	View::new().main([html::table()
		.class("ls")
		.child(html::thead().child(head))
		.child(html::tbody().children(rows))])
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_file_named_head_does_not_collide_with_the_header() {
		let view = view(&[Entry { name: "head".into(), dir: false, size: 12 }]);
		assert!(tern_sdk::reconcile::Doc::from_view(view.clone()).is_ok());
		assert_eq!(tern_sdk::plain(view, 80), "Name  Size\nhead  12\n");
	}
}

fn main() -> Result<(), tern_sdk::Error> {
	let mut entries: Vec<Entry> = fs::read_dir(".")?
		.filter_map(Result::ok)
		.filter_map(|e| {
			let meta = e.metadata().ok()?;
			Some(Entry {
				name: e.file_name().to_string_lossy().into_owned(),
				dir:  meta.is_dir(),
				size: meta.len(),
			})
		})
		.collect();
	entries.sort_by(|a, b| b.dir.cmp(&a.dir).then_with(|| a.name.cmp(&b.name)));
	tern_sdk::print(view(&entries), PrintOptions::new().css(CSS))
}
