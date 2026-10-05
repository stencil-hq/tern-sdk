//! A `flow` surface: a running card with a spinner and an elapsed timer,
//! log lines streamed into it, a progress bar, then done. Ctrl+C cancels.
//! Outside Tern the log prints as plain lines.

use std::time::{Duration, Instant};

use tern_sdk::{
	Input, Options, Session, SurfaceOptions, nodes,
	ui::{self, Bound, Extent, Status, View},
};

/// The steps of the pretend build.
const STEPS: [&str; 8] = [
	"Resolving dependencies",
	"Fetching 214 crates",
	"Compiling serde v1.0.228",
	"Compiling tern-sdk v0.1.0",
	"Linking target/debug/demo",
	"Running 42 tests",
	"Packaging demo-0.1.0.tar.gz",
	"Uploading to the registry",
];

/// How long each step takes.
const STEP: Duration = Duration::from_millis(350);

/// Where the build is.
struct Build {
	/// Log lines so far.
	log:    String,
	/// Steps done.
	done:   usize,
	/// When it finished, in ms since the start.
	took:   Option<u64>,
	/// The final status.
	status: Status,
}

/// The card for `build`.
fn view(build: &Build) -> View {
	let fraction = build.done as f64 / STEPS.len() as f64;
	let running = build.took.is_none();
	let activity: ui::Node = if running {
		ui::spinner()
			.style("dots")
			.label(STEPS[build.done.min(STEPS.len() - 1)])
			.into()
	} else {
		ui::icon(if build.status == Status::Done {
			"check"
		} else {
			"x"
		})
		.tone(if build.status == Status::Done {
			"success"
		} else {
			"error"
		})
		.into()
	};
	let mut timer = ui::elapsed().age(0);
	if let Some(took) = build.took {
		timer = timer.stopped(took);
	}
	View::new().main([ui::card()
		.key("build")
		.head("Building demo")
		.status(build.status.clone())
		.child(
			ui::row()
				.key("head")
				.gap("sm")
				.children(nodes![activity.key("a"), timer.key("t")]),
		)
		.child(
			ui::ansi(&build.log)
				.key("log")
				.follow(true)
				.max(Bound::h(Extent::Lines(8.0))),
		)
		.child(ui::progress().key("bar").value(fraction).label(format!(
			"{}/{}",
			build.done,
			STEPS.len()
		)))])
}

/// Waits `time` while handling input: whether Ctrl+C asked to stop.
fn wait(session: &mut Session, time: Duration) -> Result<bool, tern_sdk::Error> {
	let until = Instant::now() + time;
	loop {
		let left = until.saturating_duration_since(Instant::now());
		match session.next(Some(left))? {
			None => return Ok(false),
			Some(Input::Key(key)) if key.is("ctrl+c") => return Ok(true),
			Some(_) => {},
		}
	}
}

fn main() -> Result<(), tern_sdk::Error> {
	let Some(mut session) = Session::connect(Options::new())? else {
		for (i, step) in STEPS.iter().enumerate() {
			println!("[{}/{}] {step}", i + 1, STEPS.len());
			std::thread::sleep(STEP);
		}
		println!("Built demo.");
		return Ok(());
	};
	let start = Instant::now();
	let sf = session.open(SurfaceOptions::flow().title("progress"))?;
	let mut build =
		Build { log: String::new(), done: 0, took: None, status: Status::Running };
	session.render(sf, view(&build))?;
	for step in STEPS {
		if wait(&mut session, STEP)? {
			build.status = Status::Cancelled;
			break;
		}
		build.log.push_str("\x1b[32m✓\x1b[0m ");
		build.log.push_str(step);
		build.log.push_str("\r\n");
		build.done += 1;
		session.render(sf, view(&build))?;
	}
	if build.status == Status::Running {
		build.status = Status::Done;
	}
	let took = start.elapsed();
	build.took = Some(took.as_millis() as u64);
	session.render(sf, view(&build))?;
	session.close()?;
	match build.status {
		Status::Done => println!("Built demo in {:.1}s.", took.as_secs_f64()),
		_ => println!("Build cancelled."),
	}
	Ok(())
}
