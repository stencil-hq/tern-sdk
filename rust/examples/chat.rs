//! An `inline` surface: a transcript in `main`, an `editor` in `dock` that
//! the program edits from keys. Enter sends, Escape quits.

use tern_sdk::{
	Input, Options, Session, SurfaceOptions, nodes,
	ui::{self, Side, Tone, View},
	wire::feature,
};

/// What the editor's handlers report.
#[derive(Clone, Debug)]
enum Msg {
	/// Tern submits text into the composer (`send`).
	Send(String),
	/// A click asked for the keys.
	Focus,
}

/// The editor's id: `dock` region, key `ed`.
const EDITOR: &str = "dock.ed";

/// The chat's state.
#[derive(Default)]
struct Chat {
	/// Sent messages and replies: (who, text).
	transcript: Vec<(&'static str, String)>,
	/// The draft.
	draft:      String,
	/// The caret, in chars.
	cursor:     usize,
}

impl Chat {
	/// The caret's byte offset in the draft.
	fn byte(&self, chars: usize) -> usize {
		self
			.draft
			.char_indices()
			.nth(chars)
			.map_or(self.draft.len(), |(i, _)| i)
	}

	/// Inserts typed or pasted text at the caret.
	fn insert(&mut self, text: &str) {
		let at = self.byte(self.cursor);
		self.draft.insert_str(at, text);
		self.cursor += text.chars().count();
	}

	/// Sends `text`, and the program's reply.
	fn send(&mut self, text: String) {
		if text.trim().is_empty() {
			return;
		}
		let reply = format!("You said *{}* ({} chars).", text.trim(), text.chars().count());
		self.transcript.push(("you", text));
		self.transcript.push(("bot", reply));
	}

	/// The view: the transcript, then the composer and a status strip.
	fn view(&self) -> View<Msg> {
		let messages = self.transcript.iter().enumerate().map(|(i, (who, text))| {
			let node = ui::md(text.clone()).key(i);
			if *who == "you" {
				ui::card().key(i).tone("user").child(node.key("t"))
			} else {
				ui::card().key(i).variant("bare").child(node.key("t"))
			}
		});
		let cursor: usize = self
			.draft
			.chars()
			.take(self.cursor)
			.map(char::len_utf16)
			.sum();
		View::new().main(ui::col().children(messages)).dock(nodes![
			ui::editor()
				.key("ed")
				.text(self.draft.clone())
				.cursor(cursor)
				.placeholder("Message (Enter sends, Escape quits)")
				.sendable(true)
				.on_send(|text| Msg::Send(text.to_owned()))
				.on_focus(Msg::Focus),
			ui::status()
				.key("bar")
				.child(ui::seg("chat").icon("message"))
				.child(
					ui::seg(format!("{} messages", self.transcript.len()))
						.side(Side::Right)
						.tone(if self.transcript.is_empty() {
							Tone::Muted
						} else {
							Tone::Success
						}),
				),
		])
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn composer_edits_codepoints_and_reports_utf16_cursors() {
		let mut chat = Chat::default();
		chat.insert("a😀é");
		assert_eq!(chat.cursor, 3);
		assert_eq!(chat.byte(2), 5);
		chat.cursor = 2;
		chat.insert("界");
		assert_eq!(chat.draft, "a😀界é");
		let view = chat.view();
		let editor = &view.regions()[1].unwrap().child_nodes()[0];
		assert_eq!(editor.props()["cursor"], 4);
		let at = chat.byte(chat.cursor - 1);
		chat.draft.remove(at);
		chat.cursor -= 1;
		assert_eq!(chat.draft, "a😀é");
		let at = chat.byte(chat.cursor);
		chat.draft.remove(at);
		assert_eq!(chat.draft, "a😀");
	}
}

fn main() -> Result<(), tern_sdk::Error> {
	let Some(mut session) = Session::connect(Options::new().feature(feature::SEND))? else {
		println!("chat needs a terminal that speaks TSP (Tern).");
		return Ok(());
	};
	let sf = session.open(SurfaceOptions::inline().title("chat"))?;
	let mut chat = Chat::default();
	session.render(sf, chat.view())?;
	session.focus(sf, Some(EDITOR))?;
	while let Some(input) = session.next(None)? {
		match input {
			Input::Key(key) => {
				if key.is("escape") || key.is("ctrl+c") || key.is("ctrl+d") {
					break;
				}
				match key.name.as_str() {
					"enter" if !key.shift && !key.alt => {
						let text = std::mem::take(&mut chat.draft);
						chat.cursor = 0;
						chat.send(text);
					},
					"enter" => chat.insert("\n"),
					"backspace" if chat.cursor > 0 => {
						let at = chat.byte(chat.cursor - 1);
						chat.draft.remove(at);
						chat.cursor -= 1;
					},
					"delete" if chat.cursor < chat.draft.chars().count() => {
						let at = chat.byte(chat.cursor);
						chat.draft.remove(at);
					},
					"left" => chat.cursor = chat.cursor.saturating_sub(1),
					"right" => chat.cursor = (chat.cursor + 1).min(chat.draft.chars().count()),
					"home" => chat.cursor = 0,
					"end" => chat.cursor = chat.draft.chars().count(),
					"paste" => chat.insert(key.text.as_deref().unwrap_or_default()),
					_ => match key.typed() {
						Some(text) => chat.insert(text),
						None => continue,
					},
				}
			},
			Input::Msg(Msg::Send(text), _) => chat.send(text),
			Input::Msg(Msg::Focus, _) => session.focus(sf, Some(EDITOR))?,
			Input::Event(_) => continue,
		}
		session.render(sf, chat.view())?;
	}
	session.close()?;
	println!("Chat closed after {} messages.", chat.transcript.len());
	Ok(())
}
