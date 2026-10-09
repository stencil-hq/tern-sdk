//! Layer 6: one-call helpers. [`print`] shows a static view that stays in
//! the scrollback like command output; [`ask`] shows a form and returns the
//! submission; both fall back when TSP isn't available.

use std::io::Write as _;

use serde_json::{Map, Value};

use crate::{
	Error, Input, Options, Session, SurfaceOptions, plain::plain, term, ui::View, wire::Event,
};

/// Options of [`print`].
#[derive(Clone, Debug, Default)]
pub struct PrintOptions {
	/// A stylesheet for the view.
	pub css:      Option<String>,
	/// Text to write instead of the plain rendering when TSP isn't
	/// available.
	pub fallback: Option<String>,
}

impl PrintOptions {
	/// No stylesheet, the plain rendering as fallback.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the stylesheet.
	pub fn css(mut self, css: impl Into<String>) -> Self {
		self.css = Some(css.into());
		self
	}

	/// Sets the fallback text.
	pub fn fallback(mut self, text: impl Into<String>) -> Self {
		self.fallback = Some(text.into());
		self
	}
}

/// Shows `view` as static output that stays in the scrollback.
///
/// In Tern it is a `flow` surface opened with `listen:false`, its sheet, one
/// frame, then `x` with `keep`. Without TSP (or when stdout isn't a tty) it
/// writes the plain rendering, or `fallback`, to stdout.
///
/// # Errors
/// When I/O fails or two siblings share an id.
///
/// # Example
/// ```no_run
/// use tern_sdk::{PrintOptions, ui, ui::View};
///
/// let view: View = View::new().main([ui::badge("deployed").tone("success")]);
/// tern_sdk::print(view, PrintOptions::new())?;
/// tern_sdk::print(
/// 	ui::card()
/// 		.head("cargo test")
/// 		.status(ui::Status::Done)
/// 		.child(ui::md("All **42** tests pass.")),
/// 	PrintOptions::new(),
/// )?;
/// # Ok::<(), tern_sdk::Error>(())
/// ```
pub fn print(view: impl Into<View>, options: PrintOptions) -> Result<(), Error> {
	let view = view.into();
	let session = if term::stdout_is_tty() {
		Session::<()>::connect(Options::new().bracketed_paste(false).kitty_keyboard(false))?
	} else {
		None
	};
	let Some(mut session) = session else {
		let text = options
			.fallback
			.unwrap_or_else(|| plain(view, term::columns().map_or(80, usize::from)));
		let mut out = std::io::stdout().lock();
		out.write_all(text.as_bytes())?;
		if !text.ends_with('\n') {
			out.write_all(b"\n")?;
		}
		out.flush()?;
		return Ok(());
	};
	let sf = session.open(SurfaceOptions::flow().listen(false))?;
	if let Some(css) = &options.css {
		session.stylesheet(sf, "main", Some(css))?;
	}
	session.render(sf, view)?;
	session.close()
}

/// Options of [`ask`].
#[derive(Clone, Debug)]
pub struct AskOptions {
	/// A stylesheet for the form.
	pub css:    Option<String>,
	/// The action that submits (default `submit`).
	pub submit: String,
}

impl Default for AskOptions {
	fn default() -> Self {
		Self { css: None, submit: "submit".into() }
	}
}

impl AskOptions {
	/// No stylesheet, submitted by `submit`.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the stylesheet.
	pub fn css(mut self, css: impl Into<String>) -> Self {
		self.css = Some(css.into());
		self
	}

	/// Sets the action that submits.
	pub fn submit(mut self, act: impl Into<String>) -> Self {
		self.submit = act.into();
		self
	}
}

/// A submitted form: the `action` event that submitted it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Submission {
	/// The node that submitted (the button).
	pub id:     String,
	/// The action (`submit`).
	pub act:    String,
	/// The form's values.
	pub values: Map<String, Value>,
}

/// What [`ask`] got.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
	/// The user submitted the form.
	Submitted(Submission),
	/// The user pressed Escape, Ctrl+C or Ctrl+D.
	Cancelled,
	/// TSP isn't available here: ask another way.
	Unsupported,
}

/// Asks one question: shows `view` (a form) and returns its submission.
///
/// The form is a `flow` surface that stays in the scrollback. The answer is
/// the first `action` whose `act` is the submit action, or
/// [`Answer::Cancelled`] on Escape, Ctrl+C or Ctrl+D. Without TSP it is
/// [`Answer::Unsupported`].
///
/// # Errors
/// When I/O fails or two siblings share an id.
///
/// # Example
/// ```no_run
/// use tern_sdk::{
/// 	Answer, AskOptions,
/// 	ui::{Actions, View, html},
/// };
///
/// let view: View = View::new().main([html::form()
/// 	.child(
/// 		html::label()
/// 			.child(html::checkbox("notify", "on"))
/// 			.child(html::span("Notify me")),
/// 	)
/// 	.child(html::button("OK").actions(Actions::new().click("submit")))]);
/// match tern_sdk::ask(view, AskOptions::new())? {
/// 	Answer::Submitted(s) => println!("{:?}", s.values),
/// 	Answer::Cancelled => println!("cancelled"),
/// 	Answer::Unsupported => println!("no TSP"),
/// }
/// # Ok::<(), tern_sdk::Error>(())
/// ```
pub fn ask<M>(view: impl Into<View<M>>, options: AskOptions) -> Result<Answer, Error> {
	ask_with(view, options, |_| None)
}

/// [`ask`], with the form's handlers running while it waits: each message
/// goes to `update`, and a view it returns is rendered in place.
///
/// # Errors
/// When I/O fails or two siblings share an id.
pub fn ask_with<M>(
	view: impl Into<View<M>>,
	options: AskOptions,
	update: impl FnMut(M) -> Option<View<M>>,
) -> Result<Answer, Error> {
	let Some(mut session) = Session::<M>::connect(Options::new())? else {
		return Ok(Answer::Unsupported);
	};
	let answer = ask_session(&mut session, view.into(), options, update)?;
	session.close()?;
	Ok(answer)
}

/// Runs a question on an established connection.
fn ask_session<M>(
	session: &mut Session<M>,
	view: View<M>,
	options: AskOptions,
	mut update: impl FnMut(M) -> Option<View<M>>,
) -> Result<Answer, Error> {
	let sf = session.open(SurfaceOptions::flow())?;
	if let Some(css) = &options.css {
		session.stylesheet(sf, "main", Some(css))?;
	}
	session.render(sf, view)?;
	let answer = loop {
		let Some(input) = session.next(None)? else {
			continue;
		};
		let (msg, event) = match input {
			Input::Key(key) => {
				if key.is("escape") || key.is("ctrl+c") || key.is("ctrl+d") {
					break Answer::Cancelled;
				}
				continue;
			},
			Input::Clipboard(_) => continue,
			Input::Event(event) => (None, event),
			Input::Msg(msg, event) => (Some(msg), event),
		};
		if let Some(next) = msg.and_then(&mut update) {
			session.render(sf, next)?;
		}
		if let Event::Action(a) = &event
			&& a.act == options.submit
		{
			break Answer::Submitted(Submission {
				id:     a.id.clone(),
				act:    a.act.clone(),
				values: a.values.clone().unwrap_or_default(),
			});
		}
		if !session.is_open(sf) {
			break Answer::Cancelled;
		}
	};
	Ok(answer)
}

#[cfg(test)]
mod tests {
	use std::{collections::VecDeque, io, time::Duration};

	use super::{Answer, AskOptions, ask_session};
	use crate::{Options, Session, term::Terminal, ui};

	struct Script(VecDeque<Vec<u8>>);

	impl Terminal for Script {
		fn write(&mut self, _: &[u8]) -> io::Result<()> {
			Ok(())
		}

		fn read(&mut self, buf: &mut [u8], _: Option<Duration>) -> io::Result<Option<usize>> {
			let Some(bytes) = self.0.pop_front() else {
				return Ok(None);
			};
			buf[..bytes.len()].copy_from_slice(&bytes);
			Ok(Some(bytes.len()))
		}
	}

	#[test]
	fn ask_runs_submit_handler_before_returning_submission() {
		let term = Script(VecDeque::from([
			b"\x1b_tsp;r;{\"r\":\"hello\",\"v\":1,\"credits\":2}\x1b\\".to_vec(),
			b"\x1b_tsp;e;{\"ev\":\"action\",\"sf\":\"s1\",\"id\":\"main.0\",\"act\":\"submit\",\"values\":{\"size\":\"l\"}}\x1b\\".to_vec(),
		]));
		let mut session = Session::with_terminal(term, Options::new().record(None))
			.unwrap()
			.unwrap();
		let view = ui::html::button("Submit")
			.actions(ui::Actions::new().click("submit"))
			.on_click(42)
			.into();
		let mut handled = false;
		let answer = ask_session(&mut session, view, AskOptions::new(), |msg| {
			assert_eq!(msg, 42);
			handled = true;
			Some(ui::text("Submitted").into())
		})
		.unwrap();
		assert!(handled);
		assert!(
			matches!(answer, Answer::Submitted(s) if s.act == "submit" && s.values["size"] == "l")
		);
	}
}
