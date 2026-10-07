"""Session layer: detection and the `hello` handshake, raw mode, surfaces,
credit-based flow control, blobs, event routing, recording and a clean exit."""

from __future__ import annotations

import atexit
import codecs
import ctypes
import json
import os
import signal
import sys
import threading
import time
from collections import deque
from collections.abc import Iterator, Mapping, Sequence
from dataclasses import dataclass, field
from pathlib import Path
from types import FrameType, TracebackType
from typing import IO, Any, Final, TypeAlias

from . import wire
from .input import Da1, InputParser
from .keys import Key, KeyDecoder
from .reconcile import AnyView, Region, Tree, View, ViewLike, as_view
from .ui._node import route
from .wire import (
    AckEvent,
    ActionEvent,
    BlobsReply,
    Cell,
    Event,
    GoneEvent,
    HelloReply,
    Json,
    Mode,
    MotionEvent,
    Op,
    Outgoing,
    Reply,
    ResizeEvent,
    RevealAt,
    ScrollBy,
    ThemeEvent,
)

InputItem: TypeAlias = Key | Event
"""What the program reads from a session: a key or an unhandled event."""
Stream: TypeAlias = int | IO[bytes]
"""A file descriptor or a binary stream."""

FLUSH_AFTER: Final = 0.03
"""Seconds without input after which held prefixes are released as keys."""
DRAIN_FOR: Final = 0.05
"""Seconds of input drained on close, so late replies never reach the shell."""

_PASTE_ON: Final = b"\x1b[?2004h"
_PASTE_OFF: Final = b"\x1b[?2004l"
_KITTY_PUSH: Final = b"\x1b[>1u"
_KITTY_POP: Final = b"\x1b[<u"
_DA1_QUERY: Final = b"\x1b[c"
_TTY_LOCK = threading.Lock()


@dataclass
class Capabilities:
    """What the terminal said in its `hello` reply, kept current by
    `resize`, `theme` and `motion` events."""

    version: int = wire.VERSION
    term: str | None = None
    ver: str | None = None
    kinds: tuple[str, ...] = wire.KINDS
    features: tuple[str, ...] = ()
    apc: int = wire.APC_LIMIT
    credits: int = wire.CREDITS
    cols: int | None = None
    cell: Cell | None = None
    dark: bool | None = None
    reduce_motion: bool = False
    hour12: bool = False
    """The user's system reads a 12-hour clock (`3:05 PM`); False (24-hour) on terminals that don't say."""

    @classmethod
    def from_reply(cls, reply: HelloReply) -> Capabilities:
        """The capabilities a `hello` reply announces (defaults for what it leaves out)."""
        return cls(
            version=reply.v if reply.v is not None else wire.VERSION,
            term=reply.term,
            ver=reply.ver,
            kinds=reply.kinds or wire.KINDS,
            features=reply.features,
            apc=reply.apc if reply.apc and reply.apc > 0 else wire.APC_LIMIT,
            credits=reply.credits if reply.credits and reply.credits > 0 else wire.CREDITS,
            cols=reply.cols,
            cell=reply.cell,
            dark=reply.dark,
            reduce_motion=bool(reply.reduce_motion),
            hour12=bool(reply.hour12),
        )

    def has(self, feature: str) -> bool:
        """Whether the terminal lists `feature`."""
        return feature in self.features

    def draws(self, kind: str) -> bool:
        """Whether the terminal draws node kind `kind`."""
        return kind in self.kinds


# ---------------------------------------------------------------------------
# I/O and the tty


def _fileno(stream: Stream) -> int:
    return stream if isinstance(stream, int) else stream.fileno()


class _Reader:
    """Reads input with a timeout from a file descriptor."""

    def __init__(self, fd: int) -> None:
        self.fd = fd
        self._win: _WinWait | None = None
        if sys.platform == "win32":
            self._win = _WinWait(fd)

    def read(self, timeout: float | None) -> bytes | None:
        """Bytes read, `b""` at end of input, `None` when nothing came in time."""
        if self._win is not None:
            return self._win.read(timeout)
        import select

        try:
            ready, _, _ = select.select([self.fd], [], [], timeout)
        except InterruptedError:
            return None
        if not ready:
            return None
        try:
            return os.read(self.fd, 65536)
        except BlockingIOError:
            return None
        except OSError:
            return b""


class _ConsoleKey(ctypes.Structure):
    _fields_ = [
        ("down", ctypes.c_int32),
        ("repeat", ctypes.c_uint16),
        ("virtual", ctypes.c_uint16),
        ("scan", ctypes.c_uint16),
        ("char", ctypes.c_uint16),
        ("control", ctypes.c_uint32),
    ]


class _ConsoleEvent(ctypes.Union):
    _fields_ = [("key", _ConsoleKey), ("padding", ctypes.c_byte * 16)]


class _ConsoleRecord(ctypes.Structure):
    _fields_ = [("kind", ctypes.c_uint16), ("event", _ConsoleEvent)]


def _console_kernel() -> Any:
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)  # type: ignore[attr-defined,unused-ignore]
    kernel.GetConsoleMode.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
    kernel.SetConsoleMode.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel.WaitForSingleObject.argtypes = [ctypes.c_void_p, ctypes.c_uint32]
    kernel.ReadConsoleInputW.argtypes = [
        ctypes.c_void_p,
        ctypes.POINTER(_ConsoleRecord),
        ctypes.c_uint32,
        ctypes.POINTER(ctypes.c_uint32),
    ]
    return kernel


class _WinWait:
    """Windows input: waits on a console handle, or reads a pipe on a thread."""

    def __init__(self, fd: int) -> None:
        import ctypes
        import msvcrt

        self.fd = fd
        self._kernel = _console_kernel()
        self._decoder = codecs.getincrementaldecoder("utf-16-le")("replace")
        self._handle = msvcrt.get_osfhandle(fd)  # type: ignore[attr-defined,unused-ignore]
        mode = ctypes.c_uint32()
        self._console = bool(self._kernel.GetConsoleMode(self._handle, ctypes.byref(mode)))
        self._chunks: deque[bytes] = deque()
        self._ready = threading.Event()
        if not self._console:
            threading.Thread(target=self._pump, daemon=True).start()

    def _pump(self) -> None:
        while True:
            try:
                data = os.read(self.fd, 65536)
            except OSError:
                data = b""
            self._chunks.append(data)
            self._ready.set()
            if not data:
                return

    def read(self, timeout: float | None) -> bytes | None:
        if self._console:
            deadline = None if timeout is None else time.monotonic() + timeout
            records = (_ConsoleRecord * 256)()
            count = ctypes.c_uint32()
            while True:
                remaining = None if deadline is None else max(deadline - time.monotonic(), 0)
                ms = 0xFFFFFFFF if remaining is None else int(remaining * 1000)
                if self._kernel.WaitForSingleObject(self._handle, ms) != 0:
                    return None
                if not self._kernel.ReadConsoleInputW(self._handle, records, len(records), ctypes.byref(count)):
                    raise OSError("ReadConsoleInputW failed")
                chars = bytearray()
                for record in records[: count.value]:
                    key = record.event.key
                    if record.kind == 1 and key.down and key.char:
                        chars.extend(key.char.to_bytes(2, "little") * key.repeat)
                text = self._decoder.decode(bytes(chars))
                if text:
                    return text.encode("utf-8")
                if deadline is not None and time.monotonic() >= deadline:
                    return None
        if not self._chunks and not self._ready.wait(timeout):
            return None
        data = self._chunks.popleft()
        if not self._chunks:
            self._ready.clear()
        return data


class _Writer:
    """Writes bytes to a file descriptor or a binary stream."""

    def __init__(self, out: Stream) -> None:
        self._out = out

    def write(self, data: bytes) -> None:
        if isinstance(self._out, int):
            if self._out == 1 and sys.stdout is not None:
                sys.stdout.flush()
            view = memoryview(data)
            while view:
                n = os.write(self._out, view)
                view = view[n:]
        else:
            self._out.write(data)
            self._out.flush()


class _Tty:
    """Raw input mode on a tty: no echo, no line editing, no signal keys;
    output processing stays on so the program's own prints keep working."""

    def __init__(self, fd: int, out_fd: int | None) -> None:
        self.fd = fd
        self.out_fd = out_fd
        self._saved: Any = None

    def enter(self) -> None:
        if sys.platform == "win32":
            self._saved = []
            _win_console_raw(self.fd, self.out_fd, self._saved)
            return
        import termios

        saved = termios.tcgetattr(self.fd)
        mode = list(saved)
        mode[0] &= ~(termios.BRKINT | termios.ICRNL | termios.INPCK | termios.ISTRIP | termios.IXON)
        mode[2] = (mode[2] & ~(termios.CSIZE | termios.PARENB)) | termios.CS8
        mode[3] &= ~(termios.ECHO | termios.ICANON | termios.IEXTEN | termios.ISIG)
        cc = list(mode[6])
        cc[termios.VMIN] = 1
        cc[termios.VTIME] = 0
        mode[6] = cc
        self._saved = saved
        termios.tcsetattr(self.fd, termios.TCSANOW, mode)

    def restore(self) -> None:
        if self._saved is None:
            return
        saved, self._saved = self._saved, None
        if sys.platform == "win32":
            _win_console_restore(saved)
            return
        import termios

        try:
            termios.tcsetattr(self.fd, termios.TCSANOW, saved)
        except termios.error:
            pass


def _win_console_raw(fd: int, out_fd: int | None, saved: list[tuple[int, int]]) -> None:
    """Switches to VT input, saving each mode before changing it."""
    import ctypes
    import msvcrt

    kernel = _console_kernel()
    handle = msvcrt.get_osfhandle(fd)  # type: ignore[attr-defined,unused-ignore]
    mode = ctypes.c_uint32()
    try:
        if kernel.GetConsoleMode(handle, ctypes.byref(mode)):
            saved.append((handle, mode.value))
            raw = (mode.value | 0x0200) & ~(0x0001 | 0x0002 | 0x0004)
            if not kernel.SetConsoleMode(handle, raw):
                raise OSError("SetConsoleMode input failed")
        if out_fd is not None:
            out = msvcrt.get_osfhandle(out_fd)  # type: ignore[attr-defined,unused-ignore]
            if kernel.GetConsoleMode(out, ctypes.byref(mode)):
                saved.append((out, mode.value))
                if not kernel.SetConsoleMode(out, mode.value | 0x0004):
                    raise OSError("SetConsoleMode output failed")
    except BaseException:
        _win_console_restore(saved)
        raise


def _win_console_restore(saved: list[tuple[int, int]]) -> None:
    kernel = _console_kernel()
    for handle, mode in saved:
        kernel.SetConsoleMode(handle, mode)


class _Recorder:
    """Appends every logical message to a JSONL file (`TERN_TSP_RECORD`)."""

    def __init__(self, path: str) -> None:
        self.path = path

    def record(self, direction: str, verb: str, params: Sequence[tuple[str, str]], body: Json) -> None:
        line = {"t": int(time.time() * 1000), "dir": direction, "verb": verb, "params": dict(params), "body": body}
        try:
            with open(self.path, "a", encoding="utf-8") as f:
                f.write(json.dumps(line, separators=(",", ":"), ensure_ascii=False) + "\n")
        except OSError:
            pass


# ---------------------------------------------------------------------------
# Connecting


def _default_app() -> str:
    name = Path(sys.argv[0]).stem if sys.argv and sys.argv[0] not in ("", "-c") else ""
    return name or Path(sys.executable).stem or "python"


def available() -> bool:
    """Whether detection allows TSP at all: not `TERN_TSP=0`, stdin and stdout
    ttys, and not inside tmux, screen or zellij."""
    env = os.environ
    if env.get("TERN_TSP") == "0":
        return False
    if env.get("TMUX") or env.get("STY") or env.get("ZELLIJ"):
        return False
    try:
        return os.isatty(sys.stdin.fileno()) and os.isatty(sys.stdout.fileno())
    except (AttributeError, ValueError, OSError):
        return False


def connect(
    app: str | None = None,
    *,
    version: str | None = None,
    features: Sequence[wire.ProgramFeature] = (),
    timeout: float = 1.0,
    paste: bool = True,
    kitty: bool = True,
) -> Session | None:
    """Connects to Tern on stdin/stdout, or returns `None` when TSP isn't
    available (`TERN_TSP=0`, not a tty, a multiplexer, DA1 first, or no
    answer within `timeout` seconds). Use the session as a context manager:

        session = tern_sdk.connect(app="deploy")
        if session is None:
            ...  # plain output
        else:
            with session:
                ...
    """
    if not available():
        return None
    return start(
        sys.stdin.fileno(),
        sys.stdout.fileno(),
        app=app,
        version=version,
        features=features,
        timeout=timeout,
        paste=paste,
        kitty=kitty,
    )


def start(
    input: Stream,
    output: Stream,
    *,
    app: str | None = None,
    version: str | None = None,
    features: Sequence[wire.ProgramFeature] = (),
    timeout: float = 1.0,
    paste: bool = True,
    kitty: bool = True,
) -> Session | None:
    """Runs the `hello` handshake on the given input and output without the
    environment checks of `connect` (for tests and programs that own their
    pty). A tty input is switched to raw mode before `hello` and restored
    when the handshake fails."""
    in_fd = _fileno(input)
    out_fd = output if isinstance(output, int) else None
    tty = _Tty(in_fd, out_fd) if os.isatty(in_fd) else None
    session = Session(_Reader(in_fd), _Writer(output), tty, paste=paste, kitty=kitty)
    session._install()
    try:
        if tty is not None:
            tty.enter()
        ok = session._handshake(app if app is not None else _default_app(), version, features, timeout)
    except BaseException:
        try:
            session.close()
        except BaseException:
            pass
        raise
    if not ok:
        session.close()
        return None
    return session


# ---------------------------------------------------------------------------
# Session and surfaces


class Session:
    """A connection to Tern: surfaces, flow control, blobs and the input loop.

    Get one from `connect` (or `start`); `close` (or leaving its `with`
    block, process exit, SIGTERM or SIGHUP) closes its surfaces, undoes the
    keyboard modes, drains late replies and restores the tty."""

    def __init__(self, reader: _Reader, writer: _Writer, tty: _Tty | None, *, paste: bool, kitty: bool) -> None:
        self.caps = Capabilities()
        """What the terminal supports, kept current by events."""
        self._reader = reader
        self._writer = writer
        self._tty = tty
        self._paste = paste
        self._kitty = kitty
        self._parser = InputParser()
        self._keys = KeyDecoder()
        self._queue: deque[InputItem] = deque()
        self._surfaces: dict[str, Surface] = {}
        self._retired: dict[str, Surface] = {}
        self._last_input = time.monotonic()
        self._modes_enabled = False
        self._owns_tty = False
        self._next_surface = 0
        self._chunk_ids = wire.ChunkIds()
        self._blobs_sent: set[str] = set()
        self._blobs_reply: BlobsReply | None = None
        self._eof = False
        self._closed = False
        self._signals: dict[int, Any] = {}
        record = os.environ.get("TERN_TSP_RECORD")
        self._recorder = _Recorder(record) if record else None

    # -- lifecycle

    def _handshake(self, app: str, version: str | None, features: Sequence[str], timeout: float) -> bool:
        self._send(wire.hello(app, ver=version, features=features), extra=_DA1_QUERY)
        deadline = time.monotonic() + timeout
        early: list[Any] = []
        while True:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return False
            data = self._reader.read(remaining)
            if data is None:
                continue
            if not data:
                return False
            self._last_input = time.monotonic()
            items = self._parser.feed(data)
            for n, item in enumerate(items):
                if isinstance(item, Da1):
                    return False
                if isinstance(item, HelloReply):
                    self._record_in("r", item.raw)
                    self.caps = Capabilities.from_reply(item)
                    self._process(early + items[n + 1 :])
                    modes = (_PASTE_ON if self._paste else b"") + (_KITTY_PUSH if self._kitty else b"")
                    if modes:
                        self._modes_enabled = True
                        self._writer.write(modes)
                    return True
                early.append(item)

    def _install(self) -> None:
        if self._tty is not None:
            if not _TTY_LOCK.acquire(blocking=False):
                raise RuntimeError("a session already owns the tty")
            self._owns_tty = True
        atexit.register(self.close)
        if threading.current_thread() is not threading.main_thread():
            return
        for name in ("SIGINT", "SIGTERM", "SIGHUP"):
            signum = getattr(signal, name, None)
            if signum is None:
                continue
            try:
                self._signals[signum] = signal.signal(signum, self._on_signal)
            except (ValueError, OSError):
                pass

    def _on_signal(self, signum: int, frame: FrameType | None) -> None:
        previous = self._signals.get(signum, signal.SIG_DFL)
        try:
            self.close()
        except BaseException:
            pass
        if callable(previous):
            previous(signum, frame)
        else:
            signal.signal(signum, previous if previous is not None else signal.SIG_DFL)
            os.kill(os.getpid(), signum)

    @property
    def closed(self) -> bool:
        """Whether the session is closed."""
        return self._closed

    def close(self) -> None:
        """Closes open surfaces (keeping each as it says), undoes the keyboard
        modes, drains input for 50 ms and restores the tty. Idempotent."""
        if self._closed:
            return
        self._closed = True
        error: BaseException | None = None
        try:
            for surface in list(self._surfaces.values()):
                try:
                    surface.close()
                except BaseException as exc:
                    error = error or exc
            if self._modes_enabled:
                for mode in ((_KITTY_POP if self._kitty else b""), (_PASTE_OFF if self._paste else b"")):
                    if mode:
                        try:
                            self._writer.write(mode)
                        except BaseException as exc:
                            error = error or exc
            deadline = time.monotonic() + DRAIN_FOR
            try:
                while (remaining := deadline - time.monotonic()) > 0 and not self._eof:
                    data = self._reader.read(remaining)
                    if data == b"":
                        self._eof = True
                    elif data:
                        for item in self._parser.feed(data):
                            if isinstance(item, (Reply, Event)):
                                self._record_in("r" if isinstance(item, Reply) else "e", item.raw)
            except BaseException as exc:
                error = error or exc
        finally:
            try:
                if self._tty is not None:
                    self._tty.restore()
            finally:
                if self._owns_tty:
                    self._owns_tty = False
                    _TTY_LOCK.release()
                atexit.unregister(self.close)
                for signum, previous in self._signals.items():
                    try:
                        signal.signal(signum, previous if previous is not None else signal.SIG_DFL)
                    except (ValueError, OSError):
                        pass
                self._signals.clear()
        if error is not None:
            raise error

    def __enter__(self) -> Session:
        return self

    def __exit__(
        self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: TracebackType | None
    ) -> None:
        self.close()

    # -- output

    def _send(self, message: Outgoing, *, extra: bytes = b"") -> None:
        if self._recorder is not None:
            self._recorder.record("out", message.verb, message.params, message.payload)
        self._writer.write(message.encode(limit=self.caps.apc, chunk=self._chunk_ids) + extra)

    def write(self, data: bytes) -> None:
        """Writes raw bytes to the terminal, in order with TSP messages."""
        self._writer.write(data)

    def _record_in(self, verb: str, raw: Json) -> None:
        if self._recorder is not None:
            self._recorder.record("in", verb, (), raw)

    def open(
        self,
        id: str | None = None,
        *,
        mode: Mode = "inline",
        title: str | None = None,
        role: str | None = None,
        listen: bool = True,
        adopt: bool | None = None,
        keep: bool = True,
    ) -> Surface:
        """Opens a surface (`o`); ids default to `s1`, `s2`, …. `keep` is how
        the surface closes when the session does."""
        if id is None:
            self._next_surface += 1
            id = f"s{self._next_surface}"
        self._send(
            wire.open_surface(id, mode=mode, title=title, role=role, listen=None if listen else False, adopt=adopt)
        )
        surface = Surface(self, id, mode, listen, keep)
        previous = self._retired.pop(id, None)
        if adopt:
            if previous is not None:
                surface._sent = previous._sent
                surface._seq = surface._acked = previous._seq
                surface._nodes = surface._sent.nodes()
            else:
                surface._adopt_unknown = True
        self._surfaces[id] = surface
        return surface

    def blob(self, data: bytes, mime: str | None = None) -> str:
        """Sends a blob once per session and returns its id (SHA-256 hex)."""
        id = wire.blob_id(data)
        if id not in self._blobs_sent:
            self._send(wire.blob(data, mime))
            self._blobs_sent.add(id)
        return id

    def blobs(self, ids: Sequence[str], *, timeout: float = 1.0) -> list[str]:
        """Which of `ids` Tern still holds (empty when no answer comes in time)."""
        self._blobs_reply = None
        self._send(wire.blobs_query(ids))
        deadline = time.monotonic() + timeout
        while self._blobs_reply is None and not self._eof and not self._closed:
            remaining = deadline - time.monotonic()
            if remaining <= 0:
                return []
            self._read_once(remaining)
        reply, self._blobs_reply = self._blobs_reply, None
        return list(reply.have) if reply is not None else []

    def surface(self, id: str) -> Surface | None:
        """The open surface with id `id`."""
        return self._surfaces.get(id)

    # -- input

    def poll(self, timeout: float | None = None) -> InputItem | None:
        """The next key or unhandled event, waiting up to `timeout` seconds
        (`None` waits indefinitely); `None` on timeout or end of input.
        Handlers on rendered nodes run here and consume their events."""
        deadline = None if timeout is None else time.monotonic() + timeout
        if self._queue and not self._eof and not self._closed:
            # Buffered keys must not starve acks that release pending frames.
            self._read_once(0)
        first = True
        while not self._queue:
            if self._eof or self._closed:
                return None
            remaining = None if deadline is None else max(deadline - time.monotonic(), 0)
            if not first and remaining == 0:
                return None
            first = False
            self._read_once(remaining)
        return self._queue.popleft()

    def input(self, timeout: float | None = None) -> Iterator[InputItem]:
        """Keys and unhandled events as they arrive; ends at end of input,
        on close, or after `timeout` seconds without any."""
        while (item := self.poll(timeout)) is not None:
            yield item

    def _read_once(self, remaining: float | None) -> None:
        held = self._parser.pending or self._keys.pending
        wait = remaining
        idle_remaining = max(self._last_input + FLUSH_AFTER - time.monotonic(), 0)
        if held and (remaining is None or remaining > idle_remaining):
            wait = idle_remaining
        data = self._reader.read(wait)
        if data is None:
            if held and time.monotonic() >= self._last_input + FLUSH_AFTER:
                self._process(self._parser.flush())
                self._queue.extend(self._keys.flush())
                self._last_input = time.monotonic()
            return
        if not data:
            self._eof = True
            return
        self._last_input = time.monotonic()
        self._process(self._parser.feed(data))

    def _process(self, items: Sequence[Any]) -> None:
        for item in items:
            if isinstance(item, bytes):
                self._queue.extend(self._keys.feed(item))
            elif isinstance(item, Reply):
                self._record_in("r", item.raw)
                if isinstance(item, HelloReply):
                    self.caps = Capabilities.from_reply(item)
                elif isinstance(item, BlobsReply):
                    self._blobs_reply = item
            elif isinstance(item, Event):
                self._record_in("e", item.raw)
                if not self._event(item):
                    self._queue.append(item)

    def _event(self, event: Event) -> bool:
        """Applies an event; `True` when it is consumed."""
        surface = self._surfaces.get(event.sf) if event.sf is not None else None
        if isinstance(event, AckEvent):
            if surface is not None and event.s is not None:
                surface._ack(event.s)
            return True
        if isinstance(event, ResizeEvent):
            if event.cols is not None:
                self.caps.cols = event.cols
            if event.cell is not None:
                self.caps.cell = event.cell
        elif isinstance(event, ThemeEvent):
            if event.dark is not None:
                self.caps.dark = event.dark
        elif isinstance(event, MotionEvent):
            if event.reduce is not None:
                self.caps.reduce_motion = event.reduce
        elif isinstance(event, GoneEvent):
            for id in event.ids:
                gone = self._surfaces.pop(id, None)
                if gone is not None:
                    gone._closed = True
        elif surface is not None:
            node_id = getattr(event, "id", None)
            tree = surface._nodes.get(node_id) if isinstance(node_id, str) else None
            fn = route(tree.handlers, event) if tree is not None else None
            if fn is not None:
                fn(event)
                return not (isinstance(event, ActionEvent) and event.act == surface._submit)
        return False


@dataclass(eq=False)
class Surface:
    """A document shown in the pane. `render` sends only the difference from
    the last view sent, within the terminal's credits; ops on a closed
    surface do nothing."""

    session: Session
    id: str
    mode: Mode
    listen: bool
    keep: bool = True
    _closed: bool = field(default=False, init=False)
    _seq: int = field(default=0, init=False)
    _acked: int = field(default=0, init=False)
    _sent: View = field(default_factory=View, init=False)
    _view: View | None = field(default=None, init=False)
    _queued: list[Op] = field(default_factory=list, init=False)
    _nodes: dict[str, Tree] = field(default_factory=dict, init=False)
    _adopt_unknown: bool = field(default=False, init=False)
    _submit: str | None = field(default=None, init=False)

    @property
    def closed(self) -> bool:
        """Whether the surface is closed (by `close`, the session or a `gone`)."""
        return self._closed

    @property
    def blocked(self) -> bool:
        """Whether every credit is in use, so changes wait for an `ack`."""
        return self.listen and self._seq - self._acked >= self.session.caps.credits

    def render(
        self,
        view: AnyView = None,
        *,
        main: Region | None = None,
        dock: Region | None = None,
        layer: Region | None = None,
    ) -> None:
        """Shows `view` (`{main?, dock?, layer?}`, or a node or list of nodes
        as `main`'s children), or the regions given by keyword (a node given
        for a region is its root, whose own kind and props Tern doesn't draw;
        a list is wrapped in a `col`). Raises `ReconcileError`
        when two siblings share an id, sending nothing."""
        if view is not None and (main is not None or dock is not None or layer is not None):
            raise TypeError("render takes a view or region keywords, not both")
        regions: ViewLike
        if view is None:
            regions = {k: v for k, v in (("main", main), ("dock", dock), ("layer", layer)) if v is not None}
        else:
            regions = as_view(view)
        built = View.build(regions)
        if self._closed:
            return
        self._view = built
        self._nodes = built.nodes()
        self._flush()

    def send(self, ops: Sequence[Op]) -> None:
        """Queues raw frame ops, sent after the view's difference."""
        if self._closed:
            return
        self._queued.extend(ops)
        self._flush()

    def focus(self, id: str | None) -> None:
        """Gives the caret to an `editor` or `input` (by id), or to none."""
        self.send([wire.Ops.focus(id)])

    def reveal(self, id: str, at: RevealAt = "nearest") -> None:
        """Scrolls a node into view."""
        self.send([wire.Ops.reveal(id, at)])

    def scroll(self, id: str, by: ScrollBy) -> None:
        """Scrolls the nearest scroll container at or above a node."""
        self.send([wire.Ops.scroll(id, by)])

    def settle(self, id: str) -> None:
        """Hints that a subtree is unlikely to change soon."""
        self.send([wire.Ops.settle(id)])

    def suspend(self) -> None:
        """Hands the pane back to the grid (an external editor, a shell command)."""
        self.send([wire.Ops.suspend()])

    def resume(self) -> None:
        """Takes the pane again after `suspend`."""
        self.send([wire.Ops.resume()])

    def stylesheet(self, name: str, css: str | None) -> None:
        """Installs or replaces stylesheet `name`; `None` removes it."""
        if not self._closed:
            self.session._send(wire.stylesheet(self.id, name, css))

    def palette(
        self,
        *,
        dark: Mapping[str, str] | None = None,
        light: Mapping[str, str] | None = None,
        name: Mapping[str, str] | None = None,
    ) -> None:
        """Sends the program palette for this surface."""
        if not self._closed:
            self.session._send(wire.palette(self.id, dark=dark, light=light, name=name))

    def close(self, keep: bool | None = None) -> None:
        """Sends what is still pending as a last frame (even without credit,
        so a kept surface shows its final view), then closes the surface;
        `keep` (default the surface's own) leaves its `main` in the scrollback."""
        if self._closed:
            return
        kept = self.keep if keep is None else keep
        error: BaseException | None = None
        try:
            self._flush(force=True)
        except BaseException as exc:
            error = exc
        try:
            self.session._send(wire.close_surface(self.id, keep=kept))
        except BaseException as exc:
            error = error or exc
        finally:
            self._closed = True
            self._sent = View(main=self._sent.main) if kept else View()
            self.session._surfaces.pop(self.id, None)
            self.session._retired[self.id] = self
        if error is not None:
            raise error

    def __enter__(self) -> Surface:
        return self

    def __exit__(
        self, exc_type: type[BaseException] | None, exc: BaseException | None, tb: TracebackType | None
    ) -> None:
        self.close()

    def _ack(self, s: int) -> None:
        if s > self._acked:
            self._acked = s
        self._flush()

    def _flush(self, *, force: bool = False) -> None:
        if self._closed or (self.blocked and not force):
            return
        ops: list[Op] = [["del", "main"]] if self._adopt_unknown else []
        if self._view is not None:
            ops.extend(self._sent.ops(self._view, self.id))
        ops.extend(self._queued)
        if ops:
            seq = self._seq + 1
            self.session._send(wire.frame(self.id, seq, ops))
            self._seq = seq
            self._adopt_unknown = False
        if self._view is not None:
            self._sent, self._view = self._view, None
        self._queued = []
