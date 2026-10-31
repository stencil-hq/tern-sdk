//! `ask`: the "Which size?" form from the protocol book. In Tern a click on
//! a radio picks a size and Create submits; the program prints the answer.
//! Anywhere else it asks on the command line.

use std::io::{self, BufRead as _, Write as _};

use tern_sdk::{
	Answer, AskOptions, Font,
	ui::{
		self, Actions, Bound, Extent, View,
		html::{self, El},
	},
};

/// The sizes: value and label.
const SIZES: [(&str, &str); 3] = [("s", "Small"), ("m", "Medium"), ("l", "Large")];

/// One radio with its label.
fn choice(value: &str, label: &str) -> El<()> {
	html::label()
		.key(value)
		.child(html::radio("size", value).key("r"))
		.child(html::span(label).key("t"))
}

/// The form: a card of radios and a primary Create button, styled by the
/// kinds alone.
fn view() -> View<()> {
	let sizes = SIZES.iter().map(|(value, label)| choice(value, label));
	View::new().main([html::form().key("ask").child(
		ui::card()
			.key("card")
			.head("Which size?")
			.max(Bound::w(Extent::Ch(48.0)))
			.child(ui::row().key("sizes").gap("lg").children(sizes))
			.child(
				ui::button("Create")
					.key("go")
					.variant("primary")
					.actions(Actions::new().click("submit")),
			),
	)])
}

/// The label of size `value`.
fn label(value: &str) -> &str {
	SIZES
		.iter()
		.find(|(v, _)| *v == value)
		.map_or(value, |(_, l)| l)
}

/// Asks on the command line when TSP isn't available.
fn ask_plainly() -> io::Result<()> {
	print!("Which size? [s/m/l] ");
	io::stdout().flush()?;
	let mut line = String::new();
	if io::stdin().lock().read_line(&mut line)? == 0 {
		println!("\nNo size picked.");
		return Ok(());
	}
	match line.trim() {
		v @ ("s" | "m" | "l") => println!("Size: {} ({v})", label(v)),
		_ => println!("No size picked."),
	}
	Ok(())
}

fn main() -> Result<(), tern_sdk::Error> {
	let answer = tern_sdk::ask(view(), AskOptions::new().font(Font::Sans))?;
	match answer {
		Answer::Submitted(submission) => {
			let size = submission.values.get("size").and_then(|v| v.as_str());
			match size {
				Some(v) => println!("Size: {} ({v})", label(v)),
				None => println!("No size picked."),
			}
		},
		Answer::Cancelled => println!("Cancelled."),
		Answer::Unsupported => ask_plainly()?,
	}
	Ok(())
}
