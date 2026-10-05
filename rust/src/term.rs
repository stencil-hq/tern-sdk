//! The terminal a session talks through.
//!
//! [`Terminal`] lets tests and custom transports drive a session over
//! in-memory pipes; [`Tty`] is the process's own stdin/stdout in raw mode
//! (`poll(2)` on Unix, a console wait on Windows). `Tty` and tty detection
//! are available only on Unix and Windows; [`Terminal`] is portable.

#[cfg(any(unix, windows))]
use std::sync::atomic::{AtomicBool, Ordering};
use std::{io, time::Duration};

/// Process-global terminal ownership, acquired before touching platform state.
#[cfg(any(unix, windows))]
static OWNED: AtomicBool = AtomicBool::new(false);

/// Releases exclusive tty ownership after restoration (also on setup failure).
#[cfg(any(unix, windows))]
#[derive(Debug)]
struct Ownership;

#[cfg(any(unix, windows))]
impl Ownership {
	fn acquire() -> io::Result<Self> {
		OWNED
			.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
			.map(|_| Self)
			.map_err(|_| {
				io::Error::new(io::ErrorKind::AlreadyExists, "the process tty is already in use")
			})
	}
}

#[cfg(any(unix, windows))]
impl Drop for Ownership {
	fn drop(&mut self) {
		OWNED.store(false, Ordering::Release);
	}
}

/// A byte stream to a terminal with a timed read.
pub trait Terminal {
	/// Writes all of `bytes` and flushes.
	///
	/// # Errors
	/// When the output fails.
	fn write(&mut self, bytes: &[u8]) -> io::Result<()>;

	/// Reads available input into `buf`, waiting at most `timeout` (`None`
	/// waits for input). `Ok(None)` means the wait timed out; `Ok(Some(0))`
	/// that the input is closed.
	///
	/// # Errors
	/// When the input fails.
	fn read(&mut self, buf: &mut [u8], timeout: Option<Duration>) -> io::Result<Option<usize>>;

	/// Puts the terminal back as it was (leaves raw mode). Called once, when
	/// the session closes or fails to connect.
	///
	/// # Errors
	/// When the terminal can't be restored.
	fn restore(&mut self) -> io::Result<()> {
		Ok(())
	}

	/// Notes which input modes the session enabled, so a process killed by a
	/// signal can still undo them.
	fn modes(&mut self, _paste: bool, _kitty: bool) {}
}

/// Whether stdin and stdout are both terminals.
#[cfg(any(unix, windows))]
pub fn is_tty() -> bool {
	sys::is_tty()
}

/// Whether stdout is a terminal.
#[cfg(any(unix, windows))]
pub fn stdout_is_tty() -> bool {
	sys::stdout_is_tty()
}

/// The width of the terminal on stdout in columns, when it is one.
#[cfg(any(unix, windows))]
pub fn columns() -> Option<u16> {
	sys::columns()
}

/// The process's own terminal (stdin and stdout), in raw mode while it
/// lives.
#[cfg(any(unix, windows))]
#[derive(Debug)]
pub struct Tty {
	/// The platform state.
	inner:     sys::Raw,
	/// Restored already.
	restored:  bool,
	/// Held until restoration finishes.
	ownership: Option<Ownership>,
}

#[cfg(any(unix, windows))]
impl Tty {
	/// Switches the terminal to raw input: no echo, no line editing, no
	/// signals from keys (Ctrl+C is a key). Output processing stays on.
	///
	/// # Errors
	/// When stdin is not a terminal or its mode can't be changed.
	pub fn open() -> io::Result<Self> {
		let ownership = Ownership::acquire()?;
		Ok(Self { inner: sys::Raw::enter()?, restored: false, ownership: Some(ownership) })
	}
}

#[cfg(any(unix, windows))]
impl Terminal for Tty {
	fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
		use io::Write as _;
		let mut out = io::stdout().lock();
		out.write_all(bytes)?;
		out.flush()
	}

	fn read(&mut self, buf: &mut [u8], timeout: Option<Duration>) -> io::Result<Option<usize>> {
		self.inner.read(buf, timeout)
	}

	fn restore(&mut self) -> io::Result<()> {
		if self.restored {
			return Ok(());
		}
		self.restored = true;
		let result = self.inner.leave();
		self.ownership.take();
		result
	}

	fn modes(&mut self, paste: bool, kitty: bool) {
		sys::set_modes(paste, kitty);
	}
}

#[cfg(any(unix, windows))]
impl Drop for Tty {
	fn drop(&mut self) {
		if let Err(err) = self.restore() {
			tracing::warn!(%err, "could not restore the terminal");
		}
	}
}

#[cfg(all(test, any(unix, windows)))]
mod tests {
	use super::Ownership;

	#[test]
	fn tty_ownership_is_exclusive_and_released_by_drop() {
		let first = Ownership::acquire().expect("first owner");
		assert!(Ownership::acquire().is_err());
		drop(first);
		assert!(Ownership::acquire().is_ok());
	}
}

/// The bytes that undo the input modes a session enabled.
pub(crate) const fn mode_reset(paste: bool, kitty: bool) -> &'static [u8] {
	match (paste, kitty) {
		(true, true) => b"\x1b[<u\x1b[?2004l",
		(false, true) => b"\x1b[<u",
		(true, false) => b"\x1b[?2004l",
		(false, false) => b"",
	}
}

#[cfg(unix)]
mod sys {
	use std::{
		cell::UnsafeCell,
		io,
		mem::MaybeUninit,
		sync::atomic::{AtomicBool, AtomicU8, Ordering},
		time::Duration,
	};

	/// The termios to restore from a signal handler.
	struct Saved(UnsafeCell<MaybeUninit<libc::termios>>);

	// SAFETY: written once before `ARMED` is set (release) and only read after
	// `ARMED` is seen set (acquire); never written while armed.
	unsafe impl Sync for Saved {}

	/// The termios saved by the live [`Raw`].
	static SAVED: Saved = Saved(UnsafeCell::new(MaybeUninit::uninit()));
	/// Whether a signal should restore `SAVED`.
	static ARMED: AtomicBool = AtomicBool::new(false);
	/// Input modes enabled: bit 0 bracketed paste, bit 1 kitty keys.
	static MODES: AtomicU8 = AtomicU8::new(0);

	/// The signals that restore the terminal before the process ends.
	const SIGNALS: [libc::c_int; 4] = [libc::SIGTERM, libc::SIGHUP, libc::SIGINT, libc::SIGQUIT];

	/// Raw mode on stdin, with the previous state to restore.
	#[derive(Debug)]
	pub(super) struct Raw {
		/// The termios before raw mode.
		saved:    libc::termios,
		/// The signal dispositions before ours.
		previous: Vec<(libc::c_int, libc::sigaction)>,
	}

	pub(super) fn is_tty() -> bool {
		// SAFETY: isatty only inspects the descriptors.
		unsafe { libc::isatty(0) == 1 && libc::isatty(1) == 1 }
	}

	pub(super) fn stdout_is_tty() -> bool {
		// SAFETY: isatty only inspects the descriptor.
		unsafe { libc::isatty(1) == 1 }
	}

	pub(super) fn columns() -> Option<u16> {
		let mut size = MaybeUninit::<libc::winsize>::zeroed();
		// SAFETY: TIOCGWINSZ writes a winsize through the pointer.
		let ok = unsafe { libc::ioctl(1, libc::TIOCGWINSZ, size.as_mut_ptr()) } == 0;
		// SAFETY: zero-initialized, and filled in when the call succeeded.
		let size = unsafe { size.assume_init() };
		(ok && size.ws_col > 0).then_some(size.ws_col)
	}

	pub(super) fn set_modes(paste: bool, kitty: bool) {
		MODES.store(u8::from(paste) | (u8::from(kitty) << 1), Ordering::Release);
	}

	/// Restores the terminal and undoes the modes, then dies of the signal.
	extern "C" fn on_signal(sig: libc::c_int) {
		if ARMED.load(Ordering::Acquire) {
			let modes = MODES.load(Ordering::Acquire);
			let reset = super::mode_reset(modes & 1 != 0, modes & 2 != 0);
			// SAFETY: write and tcsetattr are async-signal-safe; `SAVED` was
			// initialized before `ARMED` was set.
			unsafe {
				libc::write(1, reset.as_ptr().cast(), reset.len());
				libc::tcsetattr(0, libc::TCSANOW, (*SAVED.0.get()).as_ptr());
			}
		}
		// SAFETY: signal and raise are async-signal-safe.
		unsafe {
			libc::signal(sig, libc::SIG_DFL);
			libc::raise(sig);
		}
	}

	impl Raw {
		pub(super) fn enter() -> io::Result<Self> {
			let mut termios = MaybeUninit::<libc::termios>::uninit();
			// SAFETY: tcgetattr fills the termios when it succeeds.
			if unsafe { libc::tcgetattr(0, termios.as_mut_ptr()) } != 0 {
				return Err(io::Error::last_os_error());
			}
			// SAFETY: initialized by the successful tcgetattr.
			let saved = unsafe { termios.assume_init() };
			let mut raw = saved;
			raw.c_iflag &= !(libc::IGNBRK
				| libc::BRKINT
				| libc::PARMRK
				| libc::ISTRIP
				| libc::INLCR
				| libc::IGNCR
				| libc::ICRNL
				| libc::IXON);
			raw.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON | libc::ISIG | libc::IEXTEN);
			raw.c_cflag &= !(libc::CSIZE | libc::PARENB);
			raw.c_cflag |= libc::CS8;
			raw.c_cc[libc::VMIN] = 1;
			raw.c_cc[libc::VTIME] = 0;
			// SAFETY: not armed, so no handler reads SAVED while it is written.
			unsafe { (*SAVED.0.get()).write(saved) };
			ARMED.store(true, Ordering::Release);
			let mut previous = Vec::with_capacity(SIGNALS.len());
			for sig in SIGNALS {
				// SAFETY: a zeroed sigaction is valid; sigaction fills `old`.
				unsafe {
					let mut action: libc::sigaction = std::mem::zeroed();
					action.sa_sigaction = on_signal as extern "C" fn(libc::c_int) as usize;
					libc::sigemptyset(&mut action.sa_mask);
					let mut old: libc::sigaction = std::mem::zeroed();
					if libc::sigaction(sig, &action, &mut old) != 0 {
						let err = io::Error::last_os_error();
						let mut state = Self { saved, previous };
						let _ = state.leave();
						return Err(err);
					}
					previous.push((sig, old));
				}
			}
			let mut state = Self { saved, previous };
			// SAFETY: restoration is armed before this valid termios is applied.
			if unsafe { libc::tcsetattr(0, libc::TCSANOW, &raw) } != 0 {
				let err = io::Error::last_os_error();
				let _ = state.leave();
				return Err(err);
			}
			Ok(state)
		}

		#[allow(clippy::unused_self, reason = "the Windows version reads through its handle")]
		pub(super) fn read(
			&self,
			buf: &mut [u8],
			timeout: Option<Duration>,
		) -> io::Result<Option<usize>> {
			let ms = timeout
				.map_or(-1, |t| libc::c_int::try_from(t.as_millis()).unwrap_or(libc::c_int::MAX));
			let mut fd = libc::pollfd { fd: 0, events: libc::POLLIN, revents: 0 };
			// SAFETY: one valid pollfd.
			let n = unsafe { libc::poll(&mut fd, 1, ms) };
			if n < 0 {
				let err = io::Error::last_os_error();
				return if err.kind() == io::ErrorKind::Interrupted {
					Ok(None)
				} else {
					Err(err)
				};
			}
			if n == 0 {
				return Ok(None);
			}
			// SAFETY: reads at most buf.len() bytes into buf.
			let got = unsafe { libc::read(0, buf.as_mut_ptr().cast(), buf.len()) };
			if got < 0 {
				let err = io::Error::last_os_error();
				return match err.kind() {
					io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock => Ok(None),
					_ => Err(err),
				};
			}
			Ok(Some(got as usize))
		}

		pub(super) fn leave(&mut self) -> io::Result<()> {
			// SAFETY: saved is the termios tcgetattr returned.
			let result = if unsafe { libc::tcsetattr(0, libc::TCSANOW, &self.saved) } == 0 {
				Ok(())
			} else {
				Err(io::Error::last_os_error())
			};
			ARMED.store(false, Ordering::Release);
			MODES.store(0, Ordering::Release);
			for (sig, old) in self.previous.drain(..) {
				// SAFETY: restores the disposition sigaction gave us.
				unsafe { libc::sigaction(sig, &old, std::ptr::null_mut()) };
			}
			result
		}
	}
}

#[cfg(windows)]
mod sys {
	use std::{io, time::Duration};

	use windows_sys::Win32::{
		Foundation::{HANDLE, WAIT_OBJECT_0},
		Storage::FileSystem::ReadFile,
		System::{
			Console::{
				CONSOLE_SCREEN_BUFFER_INFO, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
				ENABLE_PROCESSED_INPUT, ENABLE_PROCESSED_OUTPUT, ENABLE_VIRTUAL_TERMINAL_INPUT,
				ENABLE_VIRTUAL_TERMINAL_PROCESSING, GetConsoleMode, GetConsoleScreenBufferInfo,
				GetStdHandle, INPUT_RECORD, KEY_EVENT, PeekConsoleInputW, ReadConsoleInputW,
				STD_INPUT_HANDLE, STD_OUTPUT_HANDLE, SetConsoleMode,
			},
			Threading::{INFINITE, WaitForSingleObject},
		},
	};

	/// Raw mode on the console, with the previous modes to restore.
	#[derive(Debug)]
	pub(super) struct Raw {
		/// The console input handle.
		input:    HANDLE,
		/// The console output handle.
		output:   HANDLE,
		/// The input mode before.
		in_mode:  u32,
		/// The output mode before.
		out_mode: u32,
	}

	/// The console mode of `handle`, when it is a console.
	fn mode(handle: HANDLE) -> Option<u32> {
		let mut mode = 0;
		// SAFETY: GetConsoleMode writes a u32.
		(unsafe { GetConsoleMode(handle, &mut mode) } != 0).then_some(mode)
	}

	pub(super) fn is_tty() -> bool {
		// SAFETY: GetStdHandle has no preconditions.
		let (input, output) =
			unsafe { (GetStdHandle(STD_INPUT_HANDLE), GetStdHandle(STD_OUTPUT_HANDLE)) };
		mode(input).is_some() && mode(output).is_some()
	}

	pub(super) fn stdout_is_tty() -> bool {
		// SAFETY: GetStdHandle has no preconditions.
		mode(unsafe { GetStdHandle(STD_OUTPUT_HANDLE) }).is_some()
	}

	pub(super) fn columns() -> Option<u16> {
		// SAFETY: a zeroed buffer info is valid; the call fills it.
		unsafe {
			let mut info: CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
			if GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &mut info) == 0 {
				return None;
			}
			u16::try_from(info.srWindow.Right - info.srWindow.Left + 1).ok()
		}
	}

	pub(super) const fn set_modes(_paste: bool, _kitty: bool) {}

	impl Raw {
		pub(super) fn enter() -> io::Result<Self> {
			// SAFETY: GetStdHandle has no preconditions.
			let (input, output) =
				unsafe { (GetStdHandle(STD_INPUT_HANDLE), GetStdHandle(STD_OUTPUT_HANDLE)) };
			let in_mode = mode(input).ok_or_else(io::Error::last_os_error)?;
			let out_mode = mode(output).ok_or_else(io::Error::last_os_error)?;
			let raw_in = (in_mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT))
				| ENABLE_VIRTUAL_TERMINAL_INPUT;
			let raw_out = out_mode | ENABLE_PROCESSED_OUTPUT | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
			// SAFETY: valid console handles and modes.
			unsafe {
				if SetConsoleMode(input, raw_in) == 0 {
					return Err(io::Error::last_os_error());
				}
				if SetConsoleMode(output, raw_out) == 0 {
					let err = io::Error::last_os_error();
					SetConsoleMode(input, in_mode);
					return Err(err);
				}
			}
			Ok(Self { input, output, in_mode, out_mode })
		}

		/// Whether a key press is queued; other console events are discarded,
		/// since reading bytes would block on them.
		fn key_ready(&self) -> io::Result<bool> {
			loop {
				let mut record = INPUT_RECORD::default();
				let mut n = 0;
				// SAFETY: one record buffer.
				if unsafe { PeekConsoleInputW(self.input, &mut record, 1, &mut n) } == 0 {
					return Err(io::Error::last_os_error());
				}
				if n == 0 {
					return Ok(false);
				}
				// SAFETY: EventType says which union field is set.
				let key_down = u32::from(record.EventType) == KEY_EVENT
					&& unsafe {
						let key = record.Event.KeyEvent;
						key.bKeyDown != 0
							&& (key.uChar.UnicodeChar != 0
								|| matches!(key.wVirtualKeyCode,
								0x08 | 0x09 | 0x0d | 0x1b | 0x20..=0x28 | 0x2d | 0x2e | 0x70..=0x87))
					};
				if key_down {
					return Ok(true);
				}
				// SAFETY: one record buffer.
				if unsafe { ReadConsoleInputW(self.input, &mut record, 1, &mut n) } == 0 {
					return Err(io::Error::last_os_error());
				}
			}
		}

		pub(super) fn read(
			&self,
			buf: &mut [u8],
			timeout: Option<Duration>,
		) -> io::Result<Option<usize>> {
			let ms =
				timeout.map_or(INFINITE, |t| u32::try_from(t.as_millis()).unwrap_or(INFINITE - 1));
			// SAFETY: a valid handle.
			if unsafe { WaitForSingleObject(self.input, ms) } != WAIT_OBJECT_0 {
				return Ok(None);
			}
			if !self.key_ready()? {
				return Ok(None);
			}
			let mut got = 0;
			let len = u32::try_from(buf.len()).unwrap_or(u32::MAX);
			// SAFETY: reads at most len bytes into buf.
			if unsafe { ReadFile(self.input, buf.as_mut_ptr(), len, &mut got, std::ptr::null_mut()) }
				== 0
			{
				return Err(io::Error::last_os_error());
			}
			Ok(Some(got as usize))
		}

		pub(super) fn leave(&mut self) -> io::Result<()> {
			// SAFETY: valid console handles and the modes they had.
			unsafe {
				let input = SetConsoleMode(self.input, self.in_mode);
				let input_error = (input == 0).then(io::Error::last_os_error);
				let output = SetConsoleMode(self.output, self.out_mode);
				if let Some(err) = input_error {
					return Err(err);
				}
				if output == 0 {
					return Err(io::Error::last_os_error());
				}
			}
			Ok(())
		}
	}
}
