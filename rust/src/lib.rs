//! Talk to Tern over the Tern Surface Protocol (TSP).
//!
//! A program running in a terminal pane describes its UI as a tree of
//! nodes; Tern lays it out, draws and animates it natively. Outside Tern the
//! same program falls back to plain text. Six layers, each usable without
//! the ones above it:
//!
//! 1. [`wire`]: constants, typed messages both ways, the encoder (framing,
//!    chunking, blobs) and the decoder.
//! 2. [`input`] and [`keys`]: the streaming input parser that splits TSP out of
//!    the pty's input, and the key decoder.
//! 3. [`ui`]: typed builders for every kind and prop, styled spans, and
//!    handlers mapping events to the program's messages.
//! 4. [`reconcile`]: views diffed into frame ops with derived ids, exactly as
//!    Tern's plugin worker does it.
//! 5. [`Session`]: the handshake, raw mode, surfaces, flow control, blobs,
//!    event routing, recording and a clean exit.
//! 6. [`print`], [`ask`] and [`plain`]: one-call helpers and the plain-text
//!    fallback.
//!
//! Process-tty APIs (`term::Tty`, `Session::connect`, `print` and `ask`)
//! require Unix or Windows. The wire, input, node and reconcile layers
//! also build on `wasm32-unknown-unknown`.
//!
//! # Example
//! ```no_run
//! use tern_sdk::{PrintOptions, ui, ui::View};
//!
//! let view: View = View::new().main([ui::card().head("Deploy").status(ui::Status::Done).child(
//! 	ui::kv()
//! 		.item("service", "api-gateway")
//! 		.item("region", "eu-west-1"),
//! )]);
//! // A native card in Tern; plain text anywhere else.
//! tern_sdk::print(view, PrintOptions::new())?;
//! # Ok::<(), tern_sdk::Error>(())
//! ```

#[cfg(any(unix, windows))]
mod helpers;
pub mod input;
pub mod keys;
mod plain;
pub mod reconcile;
mod record;
mod session;
pub mod term;
pub mod ui;
pub mod wire;

#[cfg(any(unix, windows))]
pub use helpers::{Answer, AskOptions, PrintOptions, Submission, ask, ask_with, print};
pub use keys::Key;
pub use plain::plain;
pub use session::{Capabilities, Input, Options, Session, Surface, SurfaceOptions};
pub use ui::{Node, View};
pub use wire::{Event, Mode, Op};

/// What can go wrong talking TSP.
#[derive(Debug, thiserror::Error)]
pub enum Error {
	/// Terminal or file I/O failed.
	#[error("i/o: {0}")]
	Io(#[from] std::io::Error),
	/// JSON could not be encoded or decoded.
	#[error("json: {0}")]
	Json(#[from] serde_json::Error),
	/// Two siblings of a view share a key; nothing was sent.
	#[error("{path}: duplicate child key {key:?}")]
	DuplicateId {
		/// The parent node's derived id.
		path: String,
		/// The duplicate key (explicit or positional).
		key:  String,
	},
	/// A view or node in wire form is malformed.
	#[error("{0}")]
	InvalidView(String),
	/// A blob over 16 MiB.
	#[error("a blob of {0} bytes is over the 16 MiB limit")]
	BlobTooLarge(usize),
	/// The surface is closed (by the program, or `gone`).
	#[error("surface {0:?} is closed")]
	SurfaceClosed(String),
	/// No answer came in time.
	#[error("timed out waiting for {0}")]
	Timeout(&'static str),
	/// The terminal closed its input.
	#[error("the terminal closed its input")]
	InputClosed,
}
