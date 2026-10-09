//! The session driven over in-memory pipes by a scripted terminal: the
//! handshake, flow control, routing, recording and the close sequence.

use std::{cell::RefCell, collections::VecDeque, io, rc::Rc, time::Duration};

use serde_json::{Value, json};
use tern_sdk::{
	Error, Input, Options, Session, SurfaceOptions, Waker, nodes,
	term::Terminal,
	ui::{self, View, html},
	wire::Event,
};

/// Answers the terminal gives to what the program writes.
type Responder = Box<dyn FnMut(&[u8]) -> Vec<Vec<u8>>>;

/// The terminal's side of the pipes.
#[derive(Default)]
struct Pipes {
	/// Input chunks the program will read, in order.
	input:    VecDeque<Vec<u8>>,
	/// Everything the program wrote.
	output:   Vec<u8>,
	/// Whether the program restored the terminal.
	restored: bool,
	/// Writes whose byte prefixes fail once, after recording their attempted
	/// bytes.
	fail:     VecDeque<Vec<u8>>,
	/// The modes whose restoration was armed before writes.
	modes:    (bool, bool),
}

/// A terminal that answers writes with scripted input.
struct Script {
	/// Shared with the test.
	pipes:   Rc<RefCell<Pipes>>,
	/// Called on every write.
	respond: Responder,
	/// What the session gets from `waker()`.
	waker:   Option<Waker>,
}

impl Terminal for Script {
	fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
		let replies = (self.respond)(bytes);
		let mut pipes = self.pipes.borrow_mut();
		pipes.output.extend_from_slice(bytes);
		if pipes
			.fail
			.front()
			.is_some_and(|prefix| bytes.starts_with(prefix))
		{
			pipes.fail.pop_front();
			return Err(io::Error::other("scripted write failure"));
		}
		pipes.input.extend(replies);
		Ok(())
	}

	fn read(&mut self, buf: &mut [u8], timeout: Option<Duration>) -> io::Result<Option<usize>> {
		let next = self.pipes.borrow_mut().input.pop_front();
		let Some(chunk) = next else {
			std::thread::sleep(
				timeout
					.unwrap_or(Duration::ZERO)
					.min(Duration::from_millis(1)),
			);
			return Ok(None);
		};
		let n = chunk.len().min(buf.len());
		buf[..n].copy_from_slice(&chunk[..n]);
		if n < chunk.len() {
			self
				.pipes
				.borrow_mut()
				.input
				.push_front(chunk[n..].to_vec());
		}
		Ok(Some(n))
	}

	fn modes(&mut self, paste: bool, kitty: bool) {
		self.pipes.borrow_mut().modes = (paste, kitty);
	}

	fn restore(&mut self) -> io::Result<()> {
		self.pipes.borrow_mut().restored = true;
		Ok(())
	}

	fn waker(&self) -> Option<Waker> {
		self.waker.clone()
	}
}

/// A `hello` reply granting `credits`.
fn hello(credits: u32) -> Vec<u8> {
	let body = json!({"r": "hello", "v": 1, "term": "tern", "ver": "0.4.3",
		"kinds": ["col", "text", "el"], "features": ["flow", "styles"],
		"apc": 65536, "credits": credits, "cols": 100, "dark": false, "reduceMotion": false});
	format!("\x1b_tsp;r;{body}\x1b\\").into_bytes()
}

/// An event as Tern writes it.
fn event(body: &Value) -> Vec<u8> {
	format!("\x1b_tsp;e;{body}\x1b\\").into_bytes()
}

/// The DA1 answer.
const DA1: &[u8] = b"\x1b[?62;52;c";

/// A terminal that answers `hello` with `answer` (in order).
fn terminal(answer: Vec<Vec<u8>>) -> (Script, Rc<RefCell<Pipes>>) {
	let pipes = Rc::new(RefCell::new(Pipes::default()));
	let mut answer = Some(answer);
	let respond: Responder = Box::new(move |bytes| {
		if bytes.windows(7).any(|w| w == b"tsp;q;{") {
			answer.take().unwrap_or_default()
		} else {
			Vec::new()
		}
	});
	(Script { pipes: Rc::clone(&pipes), respond, waker: None }, pipes)
}

/// A connected session with `credits`.
fn connected<M>(credits: u32) -> (Session<M>, Rc<RefCell<Pipes>>) {
	let (term, pipes) = terminal(vec![hello(credits), DA1.to_vec()]);
	let session = Session::with_terminal(term, Options::new().app("test").record(None))
		.expect("handshake runs")
		.expect("TSP answers");
	pipes.borrow_mut().output.clear();
	(session, pipes)
}

/// The messages written (verb and JSON body), in order.
fn written(pipes: &Rc<RefCell<Pipes>>) -> Vec<(String, Value)> {
	let out = String::from_utf8(std::mem::take(&mut pipes.borrow_mut().output)).expect("UTF-8");
	out.split("\x1b_tsp;")
		.skip(1)
		.map(|m| {
			let m = &m[..m.find("\x1b\\").expect("ST")];
			let (verb, body) = m.split_once(';').expect("verb");
			(verb.to_owned(), serde_json::from_str(body).expect("JSON body"))
		})
		.collect()
}

/// The frames among `messages`.
fn frames(messages: &[(String, Value)]) -> Vec<&Value> {
	messages
		.iter()
		.filter(|(v, _)| v == "f")
		.map(|(_, b)| b)
		.collect()
}

/// A view whose `main` is one text node.
fn text_view(text: &str) -> View {
	View::new().main([ui::text(text).key("t")])
}

#[test]
fn a_wake_ends_an_unbounded_wait() {
	let (mut term, _pipes) = terminal(vec![hello(4), DA1.to_vec()]);
	term.waker = Some(Waker::new().expect("waker"));
	let mut session: Session = Session::with_terminal(term, Options::new().app("test").record(None))
		.expect("handshake runs")
		.expect("TSP answers");
	let waker = session.waker().expect("the terminal's waker");
	std::thread::spawn(move || waker.wake())
		.join()
		.expect("wakes");
	assert!(session.next(None).expect("reads").is_none());
}

#[test]
fn handshake_reads_the_reply_and_enables_modes() {
	let (term, pipes) = terminal(vec![hello(3), DA1.to_vec()]);
	let session: Session = Session::with_terminal(term, Options::new().app("demo").record(None))
		.expect("handshake runs")
		.expect("TSP answers");
	assert_eq!(session.caps().credits, 3);
	assert_eq!(session.caps().cols, 100);
	assert!(session.caps().has_feature("flow"));
	let out = String::from_utf8(pipes.borrow().output.clone()).expect("UTF-8");
	assert_eq!(
		out,
		"\x1b_tsp;q;{\"q\":\"hello\",\"v\":[1],\"app\":\"demo\"}\x1b\\\x1b[c\x1b[?2004h\x1b[>1u"
	);
	assert!(!pipes.borrow().restored);
}

#[test]
fn hour12_follows_the_hello_and_defaults_to_24_hour() {
	let older: Session<()> = connected::<()>(1).0;
	assert!(!older.caps().hour12);
	let body = json!({"r": "hello", "v": 1, "term": "tern", "kinds": ["col"], "hour12": true});
	let reply = format!("\x1b_tsp;r;{body}\x1b\\").into_bytes();
	let (term, _pipes) = terminal(vec![reply, DA1.to_vec()]);
	let session: Session = Session::with_terminal(term, Options::new().record(None))
		.expect("handshake runs")
		.expect("TSP answers");
	assert!(session.caps().hour12);
}

#[test]
fn da1_first_means_no_tsp_and_restores_the_tty() {
	let (term, pipes) = terminal(vec![DA1.to_vec(), hello(2)]);
	let session = Session::<()>::with_terminal(term, Options::new().record(None)).expect("runs");
	assert!(session.is_none());
	assert!(pipes.borrow().restored);
	// Nothing after the query: no modes were enabled.
	assert!(String::from_utf8_lossy(&pipes.borrow().output).ends_with("\x1b[c"));
}

#[test]
fn silence_times_out_and_restores_the_tty() {
	let (term, pipes) = terminal(Vec::new());
	let options = Options::new()
		.timeout(Duration::from_millis(40))
		.record(None);
	let session = Session::<()>::with_terminal(term, options).expect("runs");
	assert!(session.is_none());
	assert!(pipes.borrow().restored);
}

#[test]
fn renders_while_blocked_coalesce_into_one_frame_on_ack() {
	let (mut session, pipes) = connected::<()>(2);
	let sf = session.open(SurfaceOptions::flow()).expect("open");
	session.render(sf, text_view("a")).expect("render 1");
	session.render(sf, text_view("ab")).expect("render 2");
	session
		.render(sf, text_view("abc"))
		.expect("render 3, blocked");
	session
		.render(sf, text_view("abcd"))
		.expect("render 4, blocked");
	session.focus(sf, Some("main.t")).expect("focus, blocked");
	let sent = written(&pipes);
	let f = frames(&sent);
	assert_eq!(f.len(), 2, "two credits, two frames");
	assert_eq!(f[0]["s"], 1);
	assert_eq!(f[1]["ops"], json!([["text", "main.t", "append", "b"]]));

	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev": "ack", "sf": "s1", "s": 1})));
	assert!(
		session
			.next(Some(Duration::from_millis(20)))
			.expect("next")
			.is_none(),
		"acks are consumed"
	);
	let sent = written(&pipes);
	let f = frames(&sent);
	assert_eq!(f.len(), 1, "one frame carries both renders and the queued op");
	assert_eq!(f[0]["s"], 3);
	assert_eq!(f[0]["ops"], json!([["text", "main.t", "append", "cd"], ["focus", "main.t"]]));

	// An unchanged view sends no frame, even with credit.
	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev": "ack", "sf": "s1", "s": 3})));
	session.pump(Duration::from_millis(10)).expect("pump");
	session.render(sf, text_view("abcd")).expect("render");
	assert!(frames(&written(&pipes)).is_empty());
}

#[test]
fn listen_false_sends_without_acks() {
	let (mut session, pipes) = connected::<()>(1);
	let sf = session
		.open(SurfaceOptions::flow().listen(false))
		.expect("open");
	for text in ["a", "ab", "abc"] {
		session.render(sf, text_view(text)).expect("render");
	}
	let sent = written(&pipes);
	assert_eq!(sent[0], ("o".to_owned(), json!({"id": "s1", "mode": "flow", "listen": false})));
	let seqs: Vec<&Value> = frames(&sent).iter().map(|f| &f["s"]).collect();
	assert_eq!(seqs, [&json!(1), &json!(2), &json!(3)]);
}

#[derive(Clone, Debug, PartialEq)]
enum Msg {
	Go,
	Rerun,
	Notify(bool),
}

#[test]
fn events_route_to_handlers_and_the_rest_reach_the_loop() {
	let (mut session, pipes) = connected::<Msg>(2);
	let sf = session.open(SurfaceOptions::inline()).expect("open");
	let view = View::new().main([html::form()
		.key("f")
		.child(
			html::button("Go")
				.key("go")
				.on_click(Msg::Go)
				.on_menu("rerun", Msg::Rerun),
		)
		.child(
			html::checkbox("notify", "on")
				.key("n")
				.on_change(|c| Msg::Notify(c.checked == Some(true))),
		)]);
	session.render(sf, view).expect("render");
	let sent = written(&pipes);
	let add = &frames(&sent)[0]["ops"][0];
	assert_eq!(add[4]["c"][0]["c"][0]["p"]["actions"], json!({"click": "click", "menu": ["rerun"]}));

	for body in [
		json!({"ev": "action", "sf": "s1", "id": "main.f.go", "act": "click", "values": {"notify": false}}),
		json!({"ev": "action", "sf": "s1", "id": "main.f.go", "act": "rerun"}),
		json!({"ev": "change", "sf": "s1", "id": "main.f.n", "value": "on", "checked": true}),
		json!({"ev": "action", "sf": "s1", "id": "main.f.go", "act": "other"}),
		json!({"ev": "action", "sf": "s2", "id": "main.f.go", "act": "click"}),
	] {
		pipes.borrow_mut().input.push_back(event(&body));
	}
	pipes.borrow_mut().input.push_back(b"q".to_vec());
	let mut got = Vec::new();
	while let Some(input) = session.next(Some(Duration::from_millis(20))).expect("next") {
		got.push(input);
	}
	assert!(matches!(&got[0], Input::Msg(Msg::Go, Event::Action(a)) if a.values.is_some()));
	assert!(matches!(&got[1], Input::Msg(Msg::Rerun, _)));
	assert!(matches!(&got[2], Input::Msg(Msg::Notify(true), _)));
	assert!(matches!(&got[3], Input::Event(Event::Action(a)) if a.act == "other"));
	assert!(matches!(&got[4], Input::Event(_)), "another surface's event is not routed");
	assert!(matches!(&got[5], Input::Key(k) if k.is("q")));
	assert_eq!(got.len(), 6);
}

#[test]
fn a_lone_escape_becomes_a_key_after_quiet() {
	let (mut session, pipes) = connected::<()>(2);
	pipes.borrow_mut().input.push_back(b"\x1b".to_vec());
	let input = session
		.next(Some(Duration::from_millis(200)))
		.expect("next");
	assert!(matches!(input, Some(Input::Key(k)) if k.is("escape")));
}

#[test]
fn duplicate_ids_reject_the_view_and_send_nothing() {
	let (mut session, pipes) = connected::<()>(2);
	let sf = session.open(SurfaceOptions::flow()).expect("open");
	written(&pipes);
	let view: View = View::new().main(nodes![ui::text("1").key("a"), ui::text("2").key("a")]);
	assert!(matches!(session.render(sf, view), Err(Error::DuplicateId { path, key })
			if path == "main" && key == "a"));
	assert!(written(&pipes).is_empty());
}

#[test]
fn gone_closes_the_surface() {
	let (mut session, pipes) = connected::<()>(2);
	let sf = session.open(SurfaceOptions::inline()).expect("open");
	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev": "gone", "sf": "s1", "ids": ["s1"]})));
	assert!(matches!(
		session.next(Some(Duration::from_millis(20))),
		Ok(Some(Input::Event(Event::Gone(_))))
	));
	assert!(!session.is_open(sf));
	assert!(matches!(session.render(sf, text_view("x")), Err(Error::SurfaceClosed(_))));
}

#[test]
fn close_flushes_closes_surfaces_undoes_modes_and_restores() {
	let (mut session, pipes) = connected::<()>(1);
	let a = session.open(SurfaceOptions::flow()).expect("open");
	let _b = session
		.open(SurfaceOptions::screen().id("full").keep(false))
		.expect("open");
	session.render(a, text_view("1")).expect("render");
	session.render(a, text_view("12")).expect("render, blocked");
	written(&pipes);
	session.close().expect("close");
	let out = String::from_utf8_lossy(&pipes.borrow().output).into_owned();
	assert!(out.ends_with("\x1b[<u\x1b[?2004l"), "modes undone last: {out:?}");
	let sent = written(&pipes);
	assert_eq!(sent[0].0, "f", "the stored view goes out before x");
	assert_eq!(sent[0].1["ops"], json!([["text", "main.t", "append", "2"]]));
	assert_eq!(sent[1], ("x".to_owned(), json!({"id": "s1", "keep": true})));
	assert_eq!(sent[2], ("x".to_owned(), json!({"id": "full", "keep": false})));
	assert!(pipes.borrow().restored);
}

#[test]
fn recording_logs_both_directions() {
	let path = std::env::temp_dir().join(format!("tern-sdk-record-{}.jsonl", std::process::id()));
	let _ = std::fs::remove_file(&path);
	let (term, pipes) = terminal(vec![hello(2), DA1.to_vec()]);
	let mut session: Session =
		Session::with_terminal(term, Options::new().app("rec").record(Some(path.clone())))
			.expect("runs")
			.expect("TSP");
	let sf = session.open(SurfaceOptions::flow()).expect("open");
	session.render(sf, text_view("x")).expect("render");
	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev": "theme", "dark": true})));
	session.next(Some(Duration::from_millis(20))).expect("next");
	drop(session);
	let lines: Vec<Value> = std::fs::read_to_string(&path)
		.expect("recording")
		.lines()
		.map(|l| serde_json::from_str(l).expect("JSONL"))
		.collect();
	let _ = std::fs::remove_file(&path);
	let summary: Vec<(String, String)> = lines
		.iter()
		.map(|l| (l["dir"].as_str().unwrap().to_owned(), l["verb"].as_str().unwrap().to_owned()))
		.collect();
	let expect = [("out", "q"), ("in", "r"), ("out", "o"), ("out", "f"), ("in", "e"), ("out", "x")];
	assert_eq!(summary, expect.map(|(d, v)| (d.to_owned(), v.to_owned())));
	assert_eq!(lines[4]["body"], json!({"ev": "theme", "dark": true}));
	assert!(lines[0]["t"].is_u64());
}

#[test]
fn handshake_keeps_trailing_input_and_obeys_same_read_order() {
	let mut batch = hello(2);
	batch.extend_from_slice(DA1);
	batch.extend_from_slice(b"x\x1b[200~paste\x1b[201~");
	batch.extend(event(&json!({"ev":"theme","dark":true})));
	let (term, _) = terminal(vec![batch]);
	let mut session = Session::<()>::with_terminal(term, Options::new().record(None))
		.unwrap()
		.unwrap();
	assert!(session.caps().dark);
	assert!(matches!(session.next(Some(Duration::ZERO)).unwrap(), Some(Input::Key(k)) if k.is("x")));
	assert!(
		matches!(session.next(Some(Duration::ZERO)).unwrap(), Some(Input::Key(k)) if k.name == "paste" && k.text.as_deref() == Some("paste"))
	);
	assert!(matches!(
		session.next(Some(Duration::ZERO)).unwrap(),
		Some(Input::Event(Event::Theme(_)))
	));

	let mut rejected = DA1.to_vec();
	rejected.extend(hello(2));
	let (term, pipes) = terminal(vec![rejected]);
	assert!(
		Session::<()>::with_terminal(term, Options::new().record(None))
			.unwrap()
			.is_none()
	);
	assert!(pipes.borrow().restored);
}

#[test]
fn default_surface_listens_and_is_kept() {
	let (mut session, pipes) = connected::<()>(1);
	let sf = session.open(SurfaceOptions::default()).unwrap();
	session.render(sf, text_view("a")).unwrap();
	session.render(sf, text_view("ab")).unwrap();
	assert_eq!(frames(&written(&pipes)).len(), 1);
	session.close().unwrap();
	let messages = written(&pipes);
	assert_eq!(messages.last().unwrap(), &("x".into(), json!({"id":"s1","keep":true})));
}

#[test]
fn adopting_reuses_main_and_sequence_but_not_live_regions() {
	let (mut session, pipes) = connected::<()>(2);
	let sf = session.open(SurfaceOptions::inline().id("saved")).unwrap();
	let view = text_view("a")
		.dock([ui::text("dock")])
		.layer([ui::text("layer")]);
	session.render(sf, view).unwrap();
	session.close_surface(sf, true).unwrap();
	written(&pipes);
	let adopted = session
		.open(SurfaceOptions::inline().id("saved").adopt(true))
		.unwrap();
	session
		.render(adopted, text_view("ab").dock([ui::text("new dock")]))
		.unwrap();
	let sent = written(&pipes);
	let frame = frames(&sent)[0];
	assert_eq!(frame["s"], 2);
	assert_eq!(frame["ops"][0], json!(["text", "main.t", "append", "b"]));
	assert_eq!(frame["ops"][1][0], "add");
	assert_eq!(frame["ops"][1][1], "dock");
	assert_eq!(frame["ops"].as_array().unwrap().len(), 2);
}

#[test]
fn adopting_unknown_surface_resets_main_once() {
	let (mut session, pipes) = connected::<()>(2);
	let sf = session
		.open(SurfaceOptions::inline().id("unknown").adopt(true))
		.unwrap();
	session.render(sf, text_view("a")).unwrap();
	session.render(sf, text_view("ab")).unwrap();
	let sent = written(&pipes);
	let frames = frames(&sent);
	assert_eq!(frames[0]["s"], 1);
	assert_eq!(frames[0]["ops"][0], json!(["del", "main"]));
	assert_eq!(frames[0]["ops"][1][0], "add");
	assert_eq!(frames[1]["ops"], json!([["text", "main.t", "append", "b"]]));
}

#[test]
fn failed_frame_preserves_view_ops_sequence_and_credit() {
	let (mut session, pipes) = connected::<()>(1);
	let sf = session.open(SurfaceOptions::flow()).unwrap();
	pipes.borrow_mut().fail.push_back(b"\x1b_tsp;f;".to_vec());
	assert!(session.render(sf, text_view("a")).is_err());
	let failed = written(&pipes);
	assert_eq!(frames(&failed)[0]["s"], 1);
	session.focus(sf, Some("main.t")).unwrap();
	let retry = written(&pipes);
	assert_eq!(frames(&retry)[0]["s"], 1);
	assert_eq!(frames(&retry)[0]["ops"][0][0], "add");
	assert_eq!(frames(&retry)[0]["ops"][1], json!(["focus", "main.t"]));

	session.render(sf, text_view("ab")).unwrap();
	session.focus(sf, None).unwrap();
	pipes.borrow_mut().fail.push_back(b"\x1b_tsp;f;".to_vec());
	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev":"ack","sf":"s1","s":1})));
	assert!(session.pump(Duration::ZERO).is_err(), "ack write errors reach the caller");
	let failed = written(&pipes);
	assert_eq!(frames(&failed)[0]["s"], 2);
	session.send(sf, Vec::new()).unwrap();
	let retry = written(&pipes);
	assert_eq!(frames(&retry)[0]["s"], 2);
	assert_eq!(
		frames(&retry)[0]["ops"],
		json!([["text", "main.t", "append", "b"], ["focus", null]])
	);
}

#[test]
fn close_attempts_x_and_restores_even_when_final_frame_and_resets_fail() {
	let (mut session, pipes) = connected::<()>(1);
	let sf = session.open(SurfaceOptions::flow()).unwrap();
	session.render(sf, text_view("a")).unwrap();
	session.render(sf, text_view("ab")).unwrap();
	written(&pipes);
	pipes
		.borrow_mut()
		.fail
		.extend([b"\x1b_tsp;f;".to_vec(), b"\x1b[<u".to_vec()]);
	assert!(session.close().is_err());
	assert!(pipes.borrow().restored);
	assert_eq!(pipes.borrow().modes, (false, false));
	assert_eq!(written(&pipes)[1].0, "x");
}

#[test]
fn failed_mode_setup_resets_attempted_modes_and_restores() {
	let (term, pipes) = terminal(vec![hello(2)]);
	pipes.borrow_mut().fail.push_back(b"\x1b[?2004h".to_vec());
	assert!(Session::<()>::with_terminal(term, Options::new().record(None)).is_err());
	let pipes = pipes.borrow();
	assert!(pipes.restored);
	assert!(pipes.output.ends_with(b"\x1b[<u\x1b[?2004l"));
	assert_eq!(pipes.modes, (false, false));
}

#[test]
fn short_polls_do_not_flush_input_and_zero_polls_dispatch_ready_acks() {
	let (mut session, pipes) = connected::<()>(1);
	pipes.borrow_mut().input.push_back(b"\x1b".to_vec());
	assert!(session.next(Some(Duration::ZERO)).unwrap().is_none());
	assert!(
		session
			.next(Some(Duration::from_millis(1)))
			.unwrap()
			.is_none()
	);
	pipes.borrow_mut().input.push_back(b"[A".to_vec());
	assert!(
		matches!(session.next(Some(Duration::ZERO)).unwrap(), Some(Input::Key(k)) if k.is("up"))
	);
	let sf = session.open(SurfaceOptions::flow()).unwrap();
	session.render(sf, text_view("a")).unwrap();
	session.render(sf, text_view("ab")).unwrap();
	written(&pipes);
	pipes
		.borrow_mut()
		.input
		.push_back(event(&json!({"ev":"ack","sf":"s1","s":1})));
	session.pump(Duration::ZERO).unwrap();
	assert_eq!(frames(&written(&pipes))[0]["s"], 2);
}
