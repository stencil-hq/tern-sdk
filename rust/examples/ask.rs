//! `ask`: the "Which size?" form from the protocol book. In Tern a click on
//! a radio picks a size and Create submits; the program prints the answer.
//! Anywhere else it asks on the command line.

use std::io::{self, BufRead as _, Write as _};

use tern_sdk::{
	Answer, AskOptions,
	ui::{
		Actions, View,
		html::{self, El},
	},
};

/// The form's look, from the protocol book.
const CSS: &str = ".ask{display:flex;flex-direction:column;gap:8px;max-width:460px;padding:10px \
                   12px;border-radius:8px;background:var(--card);box-shadow:inset 0 0 0 1px \
                   var(--l2)} .ask p{margin:0} .ask .sizes{display:flex;gap:14px} .ask \
                   :checked+span{color:var(--accent)} .ask \
                   button{align-self:flex-start;padding:3px \
                   12px;border-radius:6px;background:var(--accent-fill);color:#fff}";

/// The sizes: value and label.
const SIZES: [(&str, &str); 3] = [("s", "Small"), ("m", "Medium"), ("l", "Large")];

/// One radio with its label.
fn choice(value: &str, label: &str) -> El<()> {
	html::label()
		.key(value)
		.child(html::radio("size", value).key("r"))
		.child(html::span(label).key("t"))
}

/// The form.
fn view() -> View<()> {
	let sizes = SIZES.iter().map(|(value, label)| choice(value, label));
	View::new().main([html::form()
		.key("ask")
		.class("ask")
		.child(html::p("Which size?").key("q"))
		.child(html::div().key("sizes").class("sizes").children(sizes))
		.child(
			html::button("Create")
				.key("go")
				.actions(Actions::new().click("submit")),
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
	let answer = tern_sdk::ask(view(), AskOptions::new().css(CSS))?;
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
