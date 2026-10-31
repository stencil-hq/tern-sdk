//! `print`: the current directory as a native `table` (sizes end-aligned,
//! mono), with no stylesheet. In Tern it stays in the scrollback as a native
//! table; anywhere else it prints as aligned plain text.

use std::fs;

use tern_sdk::{
	PrintOptions,
	ui::{self, Column, TableRow, TextAlign, TextCell, View},
};

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

/// The directory listing as a `table`: folders first with a folder icon,
/// sizes end-aligned in the mono face.
fn view(entries: &[Entry]) -> View {
	let rows = entries.iter().map(|e| {
		let (name, icon) = if e.dir {
			(format!("{}/", e.name), "folder")
		} else {
			(e.name.clone(), "file")
		};
		let size = if e.dir { "-".to_owned() } else { human(e.size) };
		TableRow::new(e.name.clone())
			.cell("name", TextCell::new(name).icon(icon))
			.cell("size", size)
	});
	View::new().main([ui::table()
		.col(Column::new("name", "Name").grow(1.0))
		.col(Column::new("size", "Size").align(TextAlign::End).mono(true))
		.rows(rows.collect::<Vec<_>>())])
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn a_file_named_head_does_not_collide_with_the_header() {
		let view = view(&[Entry { name: "head".into(), dir: false, size: 12 }]);
		assert!(tern_sdk::reconcile::Doc::from_view(view.clone()).is_ok());
		assert_eq!(tern_sdk::plain(view, 80), "Name  Size\nhead    12\n");
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
	tern_sdk::print(view(&entries), PrintOptions::new())
}
