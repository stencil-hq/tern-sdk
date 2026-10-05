//! Layer 5: the session. Detection and the `hello` handshake, raw mode and
//! input modes, surfaces, credit-based flow control, blobs, event routing
//! to handlers, recording and a clean exit.
//!
//! A session is synchronous: [`Session::next`] waits for input with a
//! timeout and returns keys, unhandled events and the messages of the
//! handlers that events were routed to. Acks are handled inside: a
//! `render` while out of credits only stores the view, and the frame that
//! carries the difference goes out when an ack returns credit.
//!
//! # Example
//! ```no_run
//! use std::time::Duration;
//!
//! use tern_sdk::{Input, Options, Session, SurfaceOptions, ui};
//!
//! #[derive(Clone)]
//! enum Msg {
//! 	Go,
//! }
//!
//! # fn main() -> Result<(), tern_sdk::Error> {
//! let Some(mut session) = Session::<Msg>::connect(Options::new())? else {
//! 	println!("no TSP here");
//! 	return Ok(());
//! };
//! let sf = session.open(SurfaceOptions::flow())?;
//! session.render(sf, ui::html::button("Go").on_click(Msg::Go))?;
//! while let Some(input) = session.next(None)? {
//! 	match input {
//! 		Input::Msg(Msg::Go, _) => break,
//! 		Input::Key(key) if key.is("escape") => break,
//! 		_ => {},
//! 	}
//! }
//! session.close()
//! # }
//! ```

use std::{
	collections::{HashSet, VecDeque},
	path::PathBuf,
	time::{Duration, Instant},
};

use serde_json::Value;

#[cfg(any(unix, windows))]
use crate::term::Tty;
use crate::{
	Error,
	input::{Item, Parser},
	keys::{Decoder, Key},
	reconcile::{Doc, Routes},
	record::Recorder,
	term::{self, Terminal},
	ui::View,
	wire::{
		Blob, Cell, Close, Encoder, Event, Frame, Message, Mode, Op, Open, Palette, Query, Reply,
		RevealAt, ScrollBy, Sheet,
	},
};

/// Undecided input is released as keys after this long without input.
const FLUSH_AFTER: Duration = Duration::from_millis(30);
/// How long close drains input so late replies never reach the shell.
const DRAIN: Duration = Duration::from_millis(50);
/// How long [`Session::blobs`] waits for its reply.
const QUERY_TIMEOUT: Duration = Duration::from_secs(2);

/// How to connect.
#[derive(Clone, Debug)]
pub struct Options {
	/// The program's name (default: the executable's).
	pub app:             Option<String>,
	/// The program's version.
	pub version:         Option<String>,
	/// Program features (`edit`, `undo`, `send`).
	pub features:        Vec<String>,
	/// How long to wait for the `hello` reply.
	pub timeout:         Duration,
	/// Enable bracketed paste while connected.
	pub bracketed_paste: bool,
	/// Push the kitty keyboard flag 1 while connected.
	pub kitty_keyboard:  bool,
	/// Record every message as JSONL (default: `TERN_TSP_RECORD`).
	pub record:          Option<PathBuf>,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			app:             None,
			version:         None,
			features:        Vec::new(),
			timeout:         Duration::from_secs(1),
			bracketed_paste: true,
			kitty_keyboard:  true,
			record:          std::env::var_os("TERN_TSP_RECORD")
				.filter(|v| !v.is_empty())
				.map(PathBuf::from),
		}
	}
}

impl Options {
	/// The defaults.
	pub fn new() -> Self {
		Self::default()
	}

	/// Sets the program's name.
	pub fn app(mut self, app: impl Into<String>) -> Self {
		self.app = Some(app.into());
		self
	}

	/// Sets the program's version.
	pub fn version(mut self, version: impl Into<String>) -> Self {
		self.version = Some(version.into());
		self
	}

	/// Adds a program feature ([`crate::wire::feature`]).
	pub fn feature(mut self, feature: impl Into<String>) -> Self {
		self.features.push(feature.into());
		self
	}

	/// Sets the handshake timeout.
	pub const fn timeout(mut self, timeout: Duration) -> Self {
		self.timeout = timeout;
		self
	}

	/// Turns bracketed paste on or off.
	pub const fn bracketed_paste(mut self, on: bool) -> Self {
		self.bracketed_paste = on;
		self
	}

	/// Turns the kitty keyboard flag on or off.
	pub const fn kitty_keyboard(mut self, on: bool) -> Self {
		self.kitty_keyboard = on;
		self
	}

	/// Records to `path` (or not).
	pub fn record(mut self, path: Option<PathBuf>) -> Self {
		self.record = path;
		self
	}
}

/// What the terminal draws and supports, kept current by `resize`,
/// `theme` and `motion` events.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Capabilities {
	/// The terminal (`tern`).
	pub term:          String,
	/// Its version.
	pub version:       String,
	/// Every kind it draws.
	pub kinds:         Vec<String>,
	/// Terminal features.
	pub features:      Vec<String>,
	/// The largest body per message.
	pub apc:           usize,
	/// Frames that may be unacknowledged.
	pub credits:       u32,
	/// The pane's width in columns.
	pub cols:          u32,
	/// The cell size in pixels.
	pub cell:          Option<Cell>,
	/// Whether the terminal shows its dark appearance.
	pub dark:          bool,
	/// Whether Reduce Motion is on.
	pub reduce_motion: bool,
	/// Whether the user's system reads a 12-hour clock (`3:05 PM`); false
	/// (24-hour) on terminals that don't say.
	pub hour12:        bool,
}

impl Capabilities {
	/// Whether the terminal draws `kind`.
	pub fn has_kind(&self, kind: &str) -> bool {
		self.kinds.iter().any(|k| k == kind)
	}

	/// Whether the terminal has `feature`.
	pub fn has_feature(&self, feature: &str) -> bool {
		self.features.iter().any(|f| f == feature)
	}
}

/// How to open a surface.
#[derive(Clone, Debug)]
pub struct SurfaceOptions {
	/// The id (default `s1`, `s2`, … per session).
	pub id:     Option<String>,
	/// The mode.
	pub mode:   Mode,
	/// Names the pane until the program sets its own title.
	pub title:  Option<String>,
	/// `data-surface` for stylesheets.
	pub role:   Option<String>,
	/// Whether the program reads its events (default `true`).
	pub listen: bool,
	/// Reopen a closed inline surface with this id.
	pub adopt:  bool,
	/// Keep it on screen when the session closes it (default `true`).
	pub keep:   bool,
}

impl Default for SurfaceOptions {
	fn default() -> Self {
		Self {
			id:     None,
			mode:   Mode::default(),
			title:  None,
			role:   None,
			listen: true,
			adopt:  false,
			keep:   true,
		}
	}
}

impl SurfaceOptions {
	/// A surface of `mode`, listening, kept on close.
	pub fn new(mode: Mode) -> Self {
		Self { mode, ..Self::default() }
	}

	/// An inline surface (a session program).
	pub fn inline() -> Self {
		Self::new(Mode::Inline)
	}

	/// A screen surface (a full-screen view).
	pub fn screen() -> Self {
		Self::new(Mode::Screen)
	}

	/// A flow surface (command output that stays in the scrollback).
	pub fn flow() -> Self {
		Self::new(Mode::Flow)
	}

	/// Sets the id.
	pub fn id(mut self, id: impl Into<String>) -> Self {
		self.id = Some(id.into());
		self
	}

	/// Sets the title.
	pub fn title(mut self, title: impl Into<String>) -> Self {
		self.title = Some(title.into());
		self
	}

	/// Sets the role.
	pub fn role(mut self, role: impl Into<String>) -> Self {
		self.role = Some(role.into());
		self
	}

	/// Sets whether the program listens (no acks or events when not).
	pub const fn listen(mut self, listen: bool) -> Self {
		self.listen = listen;
		self
	}

	/// Reopens a closed inline surface with this id.
	pub const fn adopt(mut self, adopt: bool) -> Self {
		self.adopt = adopt;
		self
	}

	/// Sets whether the session's close keeps the surface on screen.
	pub const fn keep(mut self, keep: bool) -> Self {
		self.keep = keep;
		self
	}
}

/// A surface of a session, as [`Session::open`] returned it (only valid
/// with that session; another session's handle panics).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Surface(usize);

/// What [`Session::next`] returns.
#[derive(Debug)]
pub enum Input<M> {
	/// A key press or a paste.
	Key(Key),
	/// An event no handler took.
	Event(Event),
	/// The message of the handler an event was routed to, with the event.
	Msg(M, Event),
}

/// One open (or closed) surface.
struct State<M> {
	/// The surface id.
	id:         String,
	/// Whether acks come.
	listen:     bool,
	/// Keep on the session's close.
	keep:       bool,
	/// Still open.
	open:       bool,
	/// The last frame's sequence number.
	seq:        u64,
	/// The highest frame acked.
	acked:      u64,
	/// The view as last sent.
	sent:       Doc,
	/// An adopted surface from outside this session needs its unknown main
	/// removed.
	reset_main: bool,
	/// A view rendered while out of credits.
	pending:    Option<Doc>,
	/// View ops and raw ops waiting for credit.
	queued:     Vec<Op>,
	/// The handlers of the view last rendered.
	routes:     Routes<M>,
}

impl<M> State<M> {
	/// Whether a frame must wait for an ack.
	const fn blocked(&self, credits: u32) -> bool {
		self.listen && self.seq.saturating_sub(self.acked) >= credits as u64
	}
}

/// The writing half: the terminal, the encoder and the recording.
struct Out {
	/// The terminal.
	term:   Box<dyn Terminal>,
	/// Chunks under the `apc` limit.
	enc:    Encoder,
	/// The recording, when on.
	record: Option<Recorder>,
}

impl Out {
	/// Encodes, records and writes one message.
	fn send(&mut self, message: &Message) -> Result<(), Error> {
		let bytes = self.enc.encode(message)?;
		if let Some(record) = &mut self.record {
			let body = match message {
				Message::Blob(_) => Value::String(message.body()?),
				_ => serde_json::from_str(&message.body()?)?,
			};
			record.log("out", message.verb(), &message.params(), body);
		}
		self.term.write(&bytes)?;
		Ok(())
	}

	/// Sends pending changes, committing their state only after the write
	/// succeeds.
	fn flush<M>(&mut self, state: &mut State<M>) -> Result<(), Error> {
		let mut ops = state
			.pending
			.as_ref()
			.map_or_else(Vec::new, |doc| state.sent.ops(doc, &state.id));
		if state.reset_main && (state.pending.is_some() || !state.queued.is_empty()) {
			ops.insert(0, Op::Del { id: "main".into() });
		}
		let queued_at = ops.len();
		ops.append(&mut state.queued);
		if !ops.is_empty() {
			let mut message = Message::Frame(Frame { sf: state.id.clone(), s: state.seq + 1, ops });
			if let Err(err) = self.send(&message) {
				if let Message::Frame(frame) = &mut message {
					state.queued = frame.ops.split_off(queued_at);
				}
				return Err(err);
			}
			state.seq += 1;
			state.reset_main = false;
		}
		if let Some(doc) = state.pending.take() {
			state.sent = doc;
		}
		Ok(())
	}
}

/// A connection to a terminal that speaks TSP. Dropping it closes it
/// (surfaces, modes, tty).
pub struct Session<M = ()> {
	/// The writing half.
	out:      Out,
	/// Splits TSP out of the input.
	parser:   Parser,
	/// Turns key bytes into keys.
	keys:     Decoder,
	/// What the terminal supports.
	caps:     Capabilities,
	/// Every surface opened, by [`Surface`] index.
	surfaces: Vec<State<M>>,
	/// The number of the next default surface id.
	next_id:  u32,
	/// Inputs ready for [`Session::next`].
	inbox:    VecDeque<Input<M>>,
	/// Replies not yet claimed.
	replies:  VecDeque<Reply>,
	/// Blob ids sent this session.
	blobs:    HashSet<String>,
	/// The input modes enabled: bracketed paste, kitty keys.
	modes:    (bool, bool),
	/// When input last arrived.
	last_in:  Instant,
	/// Closed already.
	closed:   bool,
}

impl<M> Session<M> {
	/// Connects to the process's terminal (Unix and Windows only): `Ok(None)`
	/// when TSP isn't available (`TERN_TSP=0`, stdin or stdout not a tty,
	/// inside tmux, screen or zellij, or the terminal answers DA1 first or not
	/// in time).
	///
	/// # Errors
	/// When the tty can't be switched to raw mode or I/O fails.
	#[cfg(any(unix, windows))]
	pub fn connect(options: Options) -> Result<Option<Self>, Error> {
		if !available() {
			return Ok(None);
		}
		let tty = Tty::open()?;
		Self::with_terminal(tty, options)
	}

	/// Runs the handshake over `term` (already in raw mode): writes `hello`
	/// and a DA1 request and waits for the reply. `Ok(None)`, with the
	/// terminal restored, when DA1 answers first or nothing answers in time.
	///
	/// # Errors
	/// When I/O fails or the recording can't be opened.
	pub fn with_terminal(
		term: impl Terminal + 'static,
		options: Options,
	) -> Result<Option<Self>, Error> {
		let record = options.record.as_deref().map(Recorder::open).transpose()?;
		let mut session = Self {
			out:      Out { term: Box::new(term), enc: Encoder::default(), record },
			parser:   Parser::new(),
			keys:     Decoder::new(),
			caps:     Capabilities::default(),
			surfaces: Vec::new(),
			next_id:  1,
			inbox:    VecDeque::new(),
			replies:  VecDeque::new(),
			blobs:    HashSet::new(),
			modes:    (false, false),
			last_in:  Instant::now(),
			closed:   false,
		};
		let app = options.app.clone().or_else(exe_name);
		match session.handshake(app, &options) {
			Ok(true) => {},
			Ok(false) => {
				session.closed = true;
				session.out.term.restore()?;
				return Ok(None);
			},
			Err(err) => {
				session.closed = true;
				let _ = session.out.term.restore();
				return Err(err);
			},
		}
		let (paste, kitty) = (options.bracketed_paste, options.kitty_keyboard);
		let mut enable = Vec::new();
		if paste {
			enable.extend_from_slice(b"\x1b[?2004h");
		}
		if kitty {
			enable.extend_from_slice(b"\x1b[>1u");
		}
		session.modes = (paste, kitty);
		session.out.term.modes(paste, kitty);
		if !enable.is_empty() {
			session.out.term.write(&enable)?;
		}
		Ok(Some(session))
	}

	/// Sends `hello` + DA1 and waits: whether the hello reply came first.
	fn handshake(&mut self, app: Option<String>, options: &Options) -> Result<bool, Error> {
		let hello =
			Query::Hello { app, ver: options.version.clone(), features: options.features.clone() };
		let mut bytes = self.out.enc.encode(&Message::Query(hello.clone()))?;
		bytes.extend_from_slice(b"\x1b[c");
		if let Some(record) = &mut self.out.record {
			record.log("out", "q", &[], serde_json::to_value(&hello)?);
		}
		self.out.term.write(&bytes)?;
		let deadline = Instant::now() + options.timeout;
		let mut buf = [0u8; 4096];
		loop {
			let left = deadline.saturating_duration_since(Instant::now());
			if left.is_zero() {
				return Ok(false);
			}
			let Some(n) = self.out.term.read(&mut buf, Some(left))? else {
				continue;
			};
			if n == 0 {
				return Ok(false);
			}
			self.last_in = Instant::now();
			let mut accepted = false;
			for item in self.parser.feed(&buf[..n]) {
				match item {
					Item::Reply(Reply::Hello(hello)) if !accepted => {
						self.record_in("r", hello.raw.clone());
						self.caps = Capabilities {
							term:          hello.term,
							version:       hello.ver,
							kinds:         hello.kinds,
							features:      hello.features,
							apc:           hello.apc,
							credits:       hello.credits,
							cols:          hello.cols,
							cell:          hello.cell,
							dark:          hello.dark,
							reduce_motion: hello.reduce_motion,
							hour12:        hello.hour12,
						};
						self.out.enc.set_limit(self.caps.apc);
						accepted = true;
					},
					Item::Da1 if !accepted => return Ok(false),
					other => self.item(other)?,
				}
			}
			if accepted {
				return Ok(true);
			}
		}
	}

	/// What the terminal supports.
	pub const fn caps(&self) -> &Capabilities {
		&self.caps
	}

	/// Opens a surface.
	///
	/// # Errors
	/// When writing fails.
	pub fn open(&mut self, options: SurfaceOptions) -> Result<Surface, Error> {
		let id = options.id.clone().unwrap_or_else(|| {
			let id = format!("s{}", self.next_id);
			self.next_id += 1;
			id
		});
		self.out.send(&Message::Open(Open {
			id:     id.clone(),
			mode:   Some(options.mode),
			title:  options.title,
			role:   options.role,
			listen: (!options.listen).then_some(false),
			adopt:  options.adopt.then_some(true),
		}))?;
		let previous = options
			.adopt
			.then(|| {
				self
					.surfaces
					.iter()
					.rposition(|state| state.id == id && !state.open)
			})
			.flatten();
		let (sent, seq) = previous.map_or_else(
			|| (Doc::new(), 0),
			|i| {
				let state = &mut self.surfaces[i];
				(std::mem::take(&mut state.sent), state.seq)
			},
		);
		self.surfaces.push(State {
			id,
			listen: options.listen,
			keep: options.keep,
			open: true,
			seq,
			acked: seq,
			sent,
			reset_main: options.adopt && previous.is_none(),
			pending: None,
			queued: Vec::new(),
			routes: Routes::default(),
		});
		Ok(Surface(self.surfaces.len() - 1))
	}

	/// The id of `sf`.
	///
	/// # Panics
	/// When `sf` comes from another session.
	pub fn surface_id(&self, sf: Surface) -> &str {
		&self.surfaces[sf.0].id
	}

	/// Whether `sf` is still open (not closed, and not `gone`).
	///
	/// # Panics
	/// When `sf` comes from another session.
	pub fn is_open(&self, sf: Surface) -> bool {
		self.surfaces[sf.0].open
	}

	/// Renders `view` on `sf`: sends the difference from the last view sent,
	/// or, while out of credits, stores it for the frame an ack releases.
	/// Its handlers replace the last view's. A bare node or list is `main`'s
	/// children.
	///
	/// # Errors
	/// [`Error::DuplicateId`] (nothing is sent), a closed surface, or I/O.
	pub fn render(&mut self, sf: Surface, view: impl Into<View<M>>) -> Result<(), Error> {
		let (doc, routes) = Doc::build(view.into())?;
		let credits = self.caps.credits;
		let state = open(&mut self.surfaces, sf)?;
		state.routes = routes;
		state.pending = Some(doc);
		if state.blocked(credits) {
			// Pick up acks already waiting, which may release the frame.
			return self.pump(Duration::ZERO);
		}
		self.out.flush(state)
	}

	/// Sends `ops` on `sf` in one frame, after anything waiting, or queues
	/// them while out of credits (the raw escape hatch).
	///
	/// # Errors
	/// A closed surface, or I/O.
	pub fn send(&mut self, sf: Surface, ops: Vec<Op>) -> Result<(), Error> {
		let credits = self.caps.credits;
		let state = open(&mut self.surfaces, sf)?;
		state.queued.extend(ops);
		if state.blocked(credits) {
			return Ok(());
		}
		self.out.flush(state)
	}

	/// Moves the caret to field `id`, or nowhere.
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn focus(&mut self, sf: Surface, id: Option<&str>) -> Result<(), Error> {
		self.send(sf, vec![Op::Focus { id: id.map(str::to_owned) }])
	}

	/// Scrolls node `id` into view.
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn reveal(&mut self, sf: Surface, id: &str, at: RevealAt) -> Result<(), Error> {
		self.send(sf, vec![Op::Reveal { id: id.to_owned(), at }])
	}

	/// Scrolls the container around node `id`.
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn scroll(&mut self, sf: Surface, id: &str, by: ScrollBy) -> Result<(), Error> {
		self.send(sf, vec![Op::Scroll { id: id.to_owned(), by }])
	}

	/// Hints that node `id` won't change soon.
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn settle(&mut self, sf: Surface, id: &str) -> Result<(), Error> {
		self.send(sf, vec![Op::Settle { id: id.to_owned() }])
	}

	/// Hands the pane back to the grid.
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn suspend(&mut self, sf: Surface) -> Result<(), Error> {
		self.send(sf, vec![Op::Suspend])
	}

	/// Takes the pane again after [`suspend`](Self::suspend).
	///
	/// # Errors
	/// As [`send`](Self::send).
	pub fn resume(&mut self, sf: Surface) -> Result<(), Error> {
		self.send(sf, vec![Op::Resume])
	}

	/// Installs or replaces stylesheet `name` of `sf`; `None` removes it.
	///
	/// # Errors
	/// A closed surface, or I/O.
	pub fn stylesheet(&mut self, sf: Surface, name: &str, css: Option<&str>) -> Result<(), Error> {
		let id = open(&mut self.surfaces, sf)?.id.clone();
		self.out.send(&Message::Sheet(Sheet {
			sf:   Some(id),
			name: name.to_owned(),
			css:  css.map(str::to_owned),
		}))
	}

	/// Sends the program palette of `sf` (its `sf` is filled in).
	///
	/// # Errors
	/// A closed surface, or I/O.
	pub fn palette(&mut self, sf: Surface, palette: Palette) -> Result<(), Error> {
		let id = open(&mut self.surfaces, sf)?.id.clone();
		self
			.out
			.send(&Message::Palette(Palette { sf: Some(id), ..palette }))
	}

	/// Closes `sf`: sends what still waits (credits or not), then `x`.
	///
	/// # Errors
	/// A closed surface, or I/O.
	pub fn close_surface(&mut self, sf: Surface, keep: bool) -> Result<(), Error> {
		let state = open(&mut self.surfaces, sf)?;
		let frame = self.out.flush(state);
		let id = state.id.clone();
		let close = self.out.send(&Message::Close(Close { id, keep }));
		if close.is_ok() {
			state.open = false;
			state.pending = None;
			state.queued.clear();
			if keep {
				state.sent.retain_main();
			} else {
				state.sent = Doc::new();
			}
		}
		frame.and(close)
	}

	/// Sends a blob once per session and returns its id.
	///
	/// # Errors
	/// [`Error::BlobTooLarge`] over 16 MiB, or I/O.
	pub fn blob(&mut self, data: Vec<u8>, mime: Option<&str>) -> Result<String, Error> {
		let blob = Blob::new(data, mime.map(str::to_owned))?;
		let id = blob.id.clone();
		if !self.blobs.contains(&id) {
			self.out.send(&Message::Blob(blob))?;
			self.blobs.insert(id.clone());
		}
		Ok(id)
	}

	/// Asks which of `ids` Tern still holds. Input arriving meanwhile waits
	/// for [`next`](Self::next).
	///
	/// # Errors
	/// [`Error::Timeout`] without a reply in 2 s, or I/O.
	pub fn blobs(&mut self, ids: Vec<String>) -> Result<Vec<String>, Error> {
		self.out.send(&Message::Query(Query::Blobs { ids }))?;
		let deadline = Instant::now() + QUERY_TIMEOUT;
		loop {
			if let Some(at) = self
				.replies
				.iter()
				.position(|r| matches!(r, Reply::Blobs(_)))
				&& let Some(Reply::Blobs(reply)) = self.replies.remove(at)
			{
				return Ok(reply.have);
			}
			let left = deadline.saturating_duration_since(Instant::now());
			if left.is_zero() {
				return Err(Error::Timeout("the blobs reply"));
			}
			self.read(Some(left))?;
		}
	}

	/// Waits up to `timeout` (`None`: until something arrives) for the next
	/// key, unhandled event, or handler message. `Ok(None)` when the time is
	/// up.
	///
	/// # Errors
	/// [`Error::InputClosed`] at the end of input, or I/O.
	pub fn next(&mut self, timeout: Option<Duration>) -> Result<Option<Input<M>>, Error> {
		let deadline = timeout.map(|t| Instant::now() + t);
		loop {
			if let Some(input) = self.inbox.pop_front() {
				return Ok(Some(input));
			}
			if deadline.is_some_and(|d| Instant::now() >= d) && !self.read(Some(Duration::ZERO))? {
				return Ok(self.inbox.pop_front());
			}
			let left = deadline.map(|d| d.saturating_duration_since(Instant::now()));
			self.read(left)?;
		}
	}

	/// Processes input for up to `timeout` without returning it: acks
	/// release waiting frames; keys and events wait for
	/// [`next`](Self::next).
	///
	/// # Errors
	/// [`Error::InputClosed`] at the end of input, or I/O.
	pub fn pump(&mut self, timeout: Duration) -> Result<(), Error> {
		let deadline = Instant::now() + timeout;
		loop {
			let left = deadline.saturating_duration_since(Instant::now());
			if !self.read(Some(left))? && Instant::now() >= deadline {
				return Ok(());
			}
		}
	}

	/// Reads once, waiting at most `timeout` but releasing undecided input
	/// after [`FLUSH_AFTER`] of quiet. Whether any input was handled.
	fn read(&mut self, timeout: Option<Duration>) -> Result<bool, Error> {
		let held = self.parser.pending() || self.keys.pending();
		let flush_in = held.then(|| FLUSH_AFTER.saturating_sub(self.last_in.elapsed()));
		let wait = match (timeout, flush_in) {
			(Some(t), Some(f)) => Some(t.min(f)),
			(t, f) => t.or(f),
		};
		let mut buf = [0u8; 8192];
		match self.out.term.read(&mut buf, wait)? {
			Some(0) => Err(Error::InputClosed),
			Some(n) => {
				self.last_in = Instant::now();
				for item in self.parser.feed(&buf[..n]) {
					self.item(item)?;
				}
				Ok(true)
			},
			None => {
				if held && self.last_in.elapsed() >= FLUSH_AFTER {
					for item in self.parser.flush() {
						self.item(item)?;
					}
					for key in self.keys.flush() {
						self.inbox.push_back(Input::Key(key));
					}
					return Ok(true);
				}
				Ok(false)
			},
		}
	}

	/// Handles one parsed input item.
	fn item(&mut self, item: Item) -> Result<(), Error> {
		match item {
			Item::Keys(bytes) => {
				for key in self.keys.feed(&bytes) {
					self.inbox.push_back(Input::Key(key));
				}
			},
			Item::Reply(reply) => {
				self.record_in("r", reply.raw().clone());
				self.replies.push_back(reply);
			},
			Item::Event(event) => {
				self.record_in("e", event.raw().clone());
				self.event(event)?;
			},
			Item::Da1 => {},
		}
		Ok(())
	}

	/// Records an incoming message.
	fn record_in(&mut self, verb: &str, raw: serde_json::Map<String, Value>) {
		if let Some(record) = &mut self.out.record {
			record.log("in", verb, &[], Value::Object(raw));
		}
	}

	/// The newest surface with id `sf`.
	fn by_id(&self, sf: &str) -> Option<usize> {
		self.surfaces.iter().rposition(|s| s.id == sf)
	}

	/// Handles an event: state updates, flow control, then routing.
	fn event(&mut self, event: Event) -> Result<(), Error> {
		match &event {
			Event::Ack(ack) => {
				if let Some(i) = self.by_id(&ack.sf) {
					let credits = self.caps.credits;
					let state = &mut self.surfaces[i];
					state.acked = state.acked.max(ack.s);
					if state.open && !state.blocked(credits) {
						self.out.flush(state)?;
					}
				}
				return Ok(());
			},
			Event::Resize(r) => {
				self.caps.cols = r.cols;
				if r.cell.is_some() {
					self.caps.cell = r.cell;
				}
			},
			Event::Theme(t) => self.caps.dark = t.dark,
			Event::Motion(m) => self.caps.reduce_motion = m.reduce,
			Event::Gone(g) => {
				for state in &mut self.surfaces {
					if g.ids.contains(&state.id) && g.sf.as_ref().is_none_or(|sf| *sf == state.id) {
						state.open = false;
					}
				}
			},
			_ => {},
		}
		let msg = event
			.sf()
			.and_then(|sf| self.by_id(sf))
			.and_then(|i| self.surfaces[i].routes.route(&event));
		self.inbox.push_back(match msg {
			Some(msg) => Input::Msg(msg, event),
			None => Input::Event(event),
		});
		Ok(())
	}

	/// Closes the session: open surfaces (each as its options say), the
	/// input modes, a short drain of late input, and the tty.
	///
	/// # Errors
	/// The first I/O error; every step still runs.
	pub fn close(mut self) -> Result<(), Error> {
		self.shutdown()
	}

	/// The close sequence (idempotent).
	fn shutdown(&mut self) -> Result<(), Error> {
		if self.closed {
			return Ok(());
		}
		self.closed = true;
		let mut first: Option<Error> = None;
		let mut note = |r: Result<(), Error>| {
			if let Err(err) = r
				&& first.is_none()
			{
				first = Some(err);
			}
		};
		for i in 0..self.surfaces.len() {
			if self.surfaces[i].open {
				let keep = self.surfaces[i].keep;
				note(self.close_surface(Surface(i), keep));
			}
		}
		let reset = term::mode_reset(self.modes.0, self.modes.1);
		if !reset.is_empty() {
			note(self.out.term.write(reset).map_err(Error::from));
		}
		self.out.term.modes(false, false);
		let until = Instant::now() + DRAIN;
		let mut buf = [0u8; 4096];
		loop {
			let left = until.saturating_duration_since(Instant::now());
			if left.is_zero() {
				break;
			}
			match self.out.term.read(&mut buf, Some(left)) {
				Ok(Some(n)) if n > 0 => {},
				_ => break,
			}
		}
		note(self.out.term.restore().map_err(Error::from));
		first.map_or(Ok(()), Err)
	}
}

impl<M> Drop for Session<M> {
	fn drop(&mut self) {
		if let Err(err) = self.shutdown() {
			tracing::warn!(%err, "closing the TSP session failed");
		}
	}
}

/// The state of `sf`, when it is open.
fn open<M>(surfaces: &mut [State<M>], sf: Surface) -> Result<&mut State<M>, Error> {
	let state = &mut surfaces[sf.0];
	if state.open {
		Ok(state)
	} else {
		Err(Error::SurfaceClosed(state.id.clone()))
	}
}

/// Whether the environment allows TSP on the process's terminal.
#[cfg(any(unix, windows))]
fn available() -> bool {
	let set = |name: &str| std::env::var_os(name).is_some_and(|v| !v.is_empty());
	if std::env::var_os("TERN_TSP").is_some_and(|v| v == "0") {
		return false;
	}
	if set("TMUX") || set("STY") || set("ZELLIJ") {
		return false;
	}
	term::is_tty()
}

/// The executable's file name, without extension.
fn exe_name() -> Option<String> {
	let exe = std::env::current_exe().ok()?;
	Some(exe.file_stem()?.to_string_lossy().into_owned())
}
