"""Streaming input parser: splits TSP replies, events and DA1 answers out of
the pty's input and passes every other byte through as key bytes."""

from __future__ import annotations

import json
from dataclasses import dataclass
from typing import Final, TypeAlias

from .wire import Event, Reply, decode_event, decode_reply, split

MAX_MESSAGE: Final = 32 << 20
"""A TSP message growing past this many bytes unterminated is dropped."""

_ESC: Final = 0x1B
_BEL: Final = 0x07
_APC_PREFIX: Final = b"tsp;"
_OSC_PREFIX: Final = b"877;tsp;"
_PASTE_START: Final = b"\x1b[200~"
_PASTE_END: Final = b"\x1b[201~"


@dataclass(frozen=True, slots=True)
class Da1:
    """A DA1 answer (`ESC [ ? … c`)."""


DA1: Final = Da1()
"""The DA1 item (every DA1 answer is equal to it)."""

InputItem: TypeAlias = bytes | Reply | Event | Da1
"""What the parser yields: key bytes, a reply, an event or a DA1 answer."""

_HOLD: Final = -1


class InputParser:
    """Takes input bytes as they arrive and yields items in order.

    Undecided prefixes (a lone ESC, `ESC _ ts`, a partial CSI) are held until
    more bytes decide them or `flush` releases them as keys; a recognized TSP
    message is never flushed.
    """

    def __init__(self) -> None:
        self._buf = bytearray()
        self._paste = False
        self._discard = False
        self._resume = 0  # where the terminator scan of a held TSP message resumes

    @property
    def pending(self) -> bool:
        """Whether bytes are held waiting for more input."""
        return bool(self._buf)

    def feed(self, data: bytes) -> list[InputItem]:
        """Parses `data`, returning the items it completes."""
        self._buf += data
        return self._run(final=False)

    def flush(self) -> list[InputItem]:
        """Releases undecided prefixes as keys (not a TSP message in progress)."""
        return self._run(final=True)

    def _run(self, *, final: bool) -> list[InputItem]:
        buf = self._buf
        out: list[InputItem] = []
        n = len(buf)
        i = 0
        keys = 0  # start of the pending run of key bytes
        if self._discard:
            i = self._skip(buf, final)
            if self._discard:
                del buf[:i]
                return out
            keys = i
        while i < n:
            if self._paste:
                j = buf.find(_PASTE_END, i)
                if j >= 0:
                    i = j + len(_PASTE_END)
                    self._paste = False
                    continue
                hold = _partial_suffix(buf, max(i, n - len(_PASTE_END) + 1), _PASTE_END)
                i = n - hold
                break
            if buf[i] != _ESC:
                j = buf.find(b"\x1b", i)
                i = n if j < 0 else j
                continue
            kind, end = self._classify(buf, i, final)
            if kind == "hold":
                break
            if kind == "keys":
                i = end
                continue
            if keys < i:
                out.append(bytes(buf[keys:i]))
            if kind == "da1":
                out.append(DA1)
            elif kind == "tsp":
                item = _decode(buf, i, end)
                if item is not None:
                    out.append(item)
            elif kind == "discard":
                self._discard = True
                k = self._skip_from(buf, end, final)
                if self._discard:
                    del buf[:k]
                    return out
                end = k
            i = keys = end
        if keys < i:
            out.append(bytes(buf[keys:i]))
        del buf[:i]
        return out

    def _skip(self, buf: bytearray, final: bool) -> int:
        return self._skip_from(buf, 0, final)

    def _skip_from(self, buf: bytearray, start: int, final: bool) -> int:
        """Skips an oversized TSP message up to its terminator."""
        n = len(buf)
        i = start
        while i < n:
            b = buf[i]
            if b == _BEL:
                self._discard = False
                return i + 1
            if b == _ESC:
                if i + 1 >= n:
                    return i
                self._discard = False
                return i + 2 if buf[i + 1] == 0x5C else i
            i += 1
        return n

    def _classify(self, buf: bytearray, i: int, final: bool) -> tuple[str, int]:
        """What the sequence at ESC `i` is, and where it ends."""
        n = len(buf)
        if i + 1 >= n:
            return ("keys", i + 1) if final else ("hold", i)
        b = buf[i + 1]
        if b == 0x5F or b == 0x5D:  # APC, OSC
            prefix = _APC_PREFIX if b == 0x5F else _OSC_PREFIX
            got = bytes(buf[i + 2 : i + 2 + len(prefix)])
            if got != prefix:
                if prefix.startswith(got):
                    return ("keys", i + 1) if final else ("hold", i)
                return "keys", i + 2
            j = i + 2 + len(prefix)
            if i == 0:
                j = max(j, self._resume)
            self._resume = 0
            bel, esc = buf.find(b"\x07", j), buf.find(b"\x1b", j)
            if bel >= 0 and (esc < 0 or bel < esc):
                return "tsp", bel + 1
            if esc >= 0:
                if esc + 1 < n:
                    return ("tsp", esc + 2) if buf[esc + 1] == 0x5C else ("drop", esc)
                j = esc
            else:
                j = n
            if n - i > MAX_MESSAGE:
                return "discard", j
            if i == 0:
                self._resume = j
            return "hold", i
        if b == 0x5B:  # CSI
            j = i + 2
            while j < n and 0x30 <= buf[j] <= 0x3F:
                j += 1
            while j < n and 0x20 <= buf[j] <= 0x2F:
                j += 1
            if j >= n:
                return ("keys", i + 1) if final else ("hold", i)
            if not 0x40 <= buf[j] <= 0x7E:
                return "keys", j
            seq = bytes(buf[i : j + 1])
            if _is_da1(seq):
                return "da1", j + 1
            if seq == _PASTE_START:
                self._paste = True
            return "keys", j + 1
        return "keys", i + 1


def _is_da1(seq: bytes) -> bool:
    if len(seq) < 4 or not seq.startswith(b"\x1b[?") or seq[-1] != 0x63:
        return False
    return all(0x30 <= b <= 0x39 or b == 0x3B for b in seq[3:-1])


def _partial_suffix(buf: bytearray, start: int, needle: bytes) -> int:
    """Length of the longest suffix of `buf` (from `start`) that begins `needle`."""
    n = len(buf)
    for k in range(max(start, n - len(needle) + 1), n):
        if needle.startswith(bytes(buf[k:])):
            return n - k
    return 0


def _decode(buf: bytearray, start: int, end: int) -> Reply | Event | None:
    """Decodes a complete TSP string, `None` for anything not delivered."""
    prefix = len(_APC_PREFIX) if buf[start + 1] == 0x5F else len(_OSC_PREFIX)
    stop = end - 1 if buf[end - 1] == _BEL else end - 2
    message = split(bytes(buf[start + 2 + prefix : stop]))
    if message is None or message.verb not in ("r", "e"):
        return None
    try:
        value = json.loads(message.body.decode("utf-8"))
    except (UnicodeDecodeError, ValueError):
        return None
    if not isinstance(value, dict):
        return None
    return decode_reply(value) if message.verb == "r" else decode_event(value)
