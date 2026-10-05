//! Recording: with `TERN_TSP_RECORD=<file>`, every logical message in both
//! directions is appended as one JSONL line
//! `{"t":ms,"dir":"out"|"in","verb":"f","params":{…},"body":{…}}`, which
//! Tern's `surface-play` scenario command replays.

use std::{
	fs::{File, OpenOptions},
	io::{self, Write as _},
	path::Path,
	time::{SystemTime, UNIX_EPOCH},
};

use serde_json::{Map, Value, json};

/// An open recording.
#[derive(Debug)]
pub struct Recorder {
	/// The JSONL file, opened for appending.
	file: File,
}

impl Recorder {
	/// Opens (creating) `path` for appending.
	pub fn open(path: &Path) -> io::Result<Self> {
		Ok(Self { file: OpenOptions::new().create(true).append(true).open(path)? })
	}

	/// Appends one message.
	pub fn log(&mut self, dir: &str, verb: &str, params: &[(&str, &str)], body: Value) {
		let t = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_or(0, |d| d.as_millis());
		let params: Map<String, Value> = params
			.iter()
			.map(|(k, v)| ((*k).to_owned(), Value::String((*v).to_owned())))
			.collect();
		let line = json!({"t": t, "dir": dir, "verb": verb, "params": params, "body": body});
		if let Err(err) = writeln!(self.file, "{line}") {
			tracing::warn!(%err, "could not append to the TSP recording");
		}
	}
}
