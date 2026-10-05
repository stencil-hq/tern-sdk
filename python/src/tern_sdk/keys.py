"""Key decoder: turns the key bytes the input parser passes through into keys
named as in Tern's plugin API."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Final

from .wire import Json


@dataclass(frozen=True, slots=True)
class Key:
    """One key press: its name, the text it types and the modifiers held.

    `name` is a lowercase character (`"a"`, `"é"`), `"space"`, a named key
    (`"enter"`, `"up"`, `"f5"`, …) or `"paste"` with the pasted `text`.
    """

    name: str
    text: str | None = None
    ctrl: bool = False
    alt: bool = False
    shift: bool = False
    meta: bool = False

    def to_json(self) -> dict[str, Json]:
        """The key as JSON: `text` only when present, modifiers only when held."""
        out: dict[str, Json] = {"name": self.name}
        if self.text is not None:
            out["text"] = self.text
        for mod in ("ctrl", "alt", "shift", "meta"):
            if getattr(self, mod):
                out[mod] = True
        return out


_PASTE_START: Final = b"\x1b[200~"
_PASTE_END: Final = b"\x1b[201~"

_CSI_LETTER: Final = {
    "A": "up", "B": "down", "C": "right", "D": "left", "E": "begin", "H": "home", "F": "end",
    "P": "f1", "Q": "f2", "R": "f3", "S": "f4",
}  # fmt: skip
_TILDE: Final = {
    1: "home", 2: "insert", 3: "delete", 4: "end", 5: "page_up", 6: "page_down", 7: "home",
    8: "end", 11: "f1", 12: "f2", 13: "f3", 14: "f4", 15: "f5", 17: "f6", 18: "f7", 19: "f8",
    20: "f9", 21: "f10", 23: "f11", 24: "f12", 25: "f13", 26: "f14", 28: "f15", 29: "menu",
    31: "f17", 32: "f18", 33: "f19", 34: "f20",
}  # fmt: skip
_KITTY: Final = {
    9: "tab", 13: "enter", 27: "escape", 127: "backspace", 8: "backspace",
    57358: "caps_lock", 57359: "scroll_lock", 57360: "num_lock", 57361: "print_screen",
    57362: "pause", 57363: "menu", 57414: "enter", 57417: "left", 57418: "right", 57419: "up",
    57420: "down", 57421: "page_up", 57422: "page_down", 57423: "home", 57424: "end",
    57425: "insert", 57426: "delete", 57427: "begin",
}  # fmt: skip
_KEYPAD_TEXT: Final = {
    **{57399 + d: str(d) for d in range(10)},
    57409: ".", 57410: "/", 57411: "*", 57412: "-", 57413: "+", 57415: "=", 57416: ",",
}  # fmt: skip


def _mods(param: str) -> tuple[bool, bool, bool, bool] | None:
    """`(shift, alt, ctrl, meta)` of an xterm/kitty modifier parameter."""
    if not param:
        return False, False, False, False
    head = param.split(":", 1)[0]
    if not head.isdigit():
        return None
    m = max(int(head) - 1, 0)
    return bool(m & 1), bool(m & 2), bool(m & 4), bool(m & 8 or m & 32)


def _char_key(ch: str) -> Key:
    """The key typing the printable character `ch`."""
    if ch == " ":
        return Key("space", " ")
    lower = ch.lower()
    return Key(lower if len(lower) == 1 else ch, ch, shift=ch != lower)


def _control_key(c: int) -> Key:
    """The key for C0 control byte `c` (or DEL)."""
    if c in (0x0D, 0x0A):
        return Key("enter")
    if c == 0x09:
        return Key("tab")
    if c in (0x7F, 0x08):
        return Key("backspace", ctrl=c == 0x08)
    if c == 0x1B:
        return Key("escape")
    if c == 0x00:
        return Key("space", ctrl=True)
    if c <= 0x1A:
        return Key(chr(c + 0x60), ctrl=True)
    return Key(chr(c + 0x40), ctrl=True)


def _with_alt(key: Key) -> Key:
    return Key(key.name, None, key.ctrl, True, key.shift, key.meta)


def _csi(seq: str) -> Key | None:
    """Decodes the body of a CSI sequence (between `ESC [` and its end)."""
    final, body = seq[-1], seq[:-1]
    if body[:1] in ("?", "<", ">", "="):
        return None
    params = body.split(";") if body else []
    if final in _CSI_LETTER or final == "Z":
        mods = _mods(params[1]) if len(params) > 1 else _mods("")
        if mods is None:
            return None
        shift, alt, ctrl, meta = mods
        if final == "Z":
            return Key("tab", shift=True, alt=alt, ctrl=ctrl, meta=meta)
        return Key(_CSI_LETTER[final], ctrl=ctrl, alt=alt, shift=shift, meta=meta)
    if final == "~":
        if not params or not params[0].isdigit():
            return None
        name = _TILDE.get(int(params[0]))
        mods = _mods(params[1]) if len(params) > 1 else _mods("")
        if name is None or mods is None:
            return None
        shift, alt, ctrl, meta = mods
        return Key(name, ctrl=ctrl, alt=alt, shift=shift, meta=meta)
    if final == "u":
        return _kitty(params)
    return None


def _kitty(params: list[str]) -> Key | None:
    """Decodes a kitty `CSI code[:shifted];mods[:event] u` key."""
    if not params:
        return None
    codes = params[0].split(":")
    if not codes[0].isdigit():
        return None
    code = int(codes[0])
    shifted = int(codes[1]) if len(codes) > 1 and codes[1].isdigit() else None
    mod_param = params[1] if len(params) > 1 else ""
    if ":" in mod_param and mod_param.split(":", 1)[1] == "3":
        return None
    mods = _mods(mod_param)
    if mods is None:
        return None
    shift, alt, ctrl, meta = mods
    name = _KITTY.get(code)
    if name is not None:
        return Key(name, ctrl=ctrl, alt=alt, shift=shift, meta=meta)
    if 57376 <= code <= 57398:
        return Key(f"f{code - 57376 + 13}", ctrl=ctrl, alt=alt, shift=shift, meta=meta)
    text = _KEYPAD_TEXT.get(code)
    if text is None:
        if 57344 <= code <= 63743 or code < 32 or code > 0x10FFFF:
            return None
        ch = chr(code)
        text = chr(shifted) if shift and shifted else (ch.upper() if shift else ch)
        name = ch.lower()
    else:
        name = text
    if name == " ":
        name = "space"
    if ctrl or alt or meta:
        return Key(name, ctrl=ctrl, alt=alt, shift=shift, meta=meta)
    return Key(name, text, shift=shift)


class KeyDecoder:
    """Turns key bytes into keys: legacy xterm input, kitty `CSI u` keys and
    bracketed paste. Incomplete sequences are held until more bytes arrive or
    `flush` drops them (a lone ESC is then Escape); unknown sequences are
    dropped, and a paste always waits for its end."""

    def __init__(self) -> None:
        self._buf = bytearray()
        self._paste: bytearray | None = None

    @property
    def pending(self) -> bool:
        """Whether bytes are held waiting for more input."""
        return bool(self._buf) or self._paste is not None

    def feed(self, data: bytes) -> list[Key]:
        """Decodes `data`, returning the keys it completes."""
        self._buf += data
        return self._run(final=False)

    def flush(self) -> list[Key]:
        """Releases held bytes: a lone ESC is Escape."""
        return self._run(final=True)

    def _run(self, *, final: bool) -> list[Key]:
        buf = self._buf
        out: list[Key] = []
        i = 0
        n = len(buf)
        while i < n:
            if self._paste is not None:
                j = buf.find(_PASTE_END, i)
                if j < 0:
                    keep = _partial_suffix(buf, i, _PASTE_END)
                    self._paste += buf[i : n - keep]
                    i = n - keep
                    break
                self._paste += buf[i:j]
                out.append(Key("paste", self._paste.decode("utf-8", "replace")))
                self._paste = None
                i = j + len(_PASTE_END)
                continue
            key, used = self._one(buf, i, final)
            if used == 0:
                break
            if key is not None:
                out.append(key)
            i += used
        del buf[:i]
        return out

    def _one(self, buf: bytearray, i: int, final: bool) -> tuple[Key | None, int]:
        """The key at `i` and the bytes it takes; 0 bytes means wait for more."""
        n = len(buf)
        c = buf[i]
        if c != 0x1B:
            return _plain(buf, i, final)
        if i + 1 >= n:
            return (Key("escape"), 1) if final else (None, 0)
        nxt = buf[i + 1]
        if nxt == 0x5B:  # CSI
            j = i + 2
            while j < n and 0x30 <= buf[j] <= 0x3F:
                j += 1
            while j < n and 0x20 <= buf[j] <= 0x2F:
                j += 1
            if j >= n:
                return (None, n - i) if final else (None, 0)
            if not 0x40 <= buf[j] <= 0x7E:
                return None, j - i
            seq = bytes(buf[i : j + 1])
            if seq == _PASTE_START:
                self._paste = bytearray()
                return None, j + 1 - i
            return _csi(seq[2:].decode("ascii")), j + 1 - i
        if nxt == 0x4F:  # SS3
            if i + 2 >= n:
                return (None, n - i) if final else (None, 0)
            name = _CSI_LETTER.get(chr(buf[i + 2]))
            return (Key(name) if name else None), 3
        if nxt == 0x1B and i + 2 >= n and not final:
            return None, 0
        key, used = self._one(buf, i + 1, final) if nxt == 0x1B else _plain(buf, i + 1, final)
        if used == 0:
            return None, 0
        return (_with_alt(key) if key is not None else None), used + 1


def _plain(buf: bytearray, i: int, final: bool) -> tuple[Key | None, int]:
    """A control byte or one UTF-8 character at `i`."""
    c = buf[i]
    if c < 0x20 or c == 0x7F:
        return _control_key(c), 1
    if c < 0x80:
        return _char_key(chr(c)), 1
    size = 2 if c >> 5 == 0b110 else 3 if c >> 4 == 0b1110 else 4 if c >> 3 == 0b11110 else 0
    if size == 0:
        return None, 1
    if i + size > len(buf):
        return (None, len(buf) - i) if final else (None, 0)
    try:
        ch = bytes(buf[i : i + size]).decode("utf-8")
    except UnicodeDecodeError:
        return None, 1
    return _char_key(ch), size


def _partial_suffix(buf: bytearray, start: int, needle: bytes) -> int:
    """Length of the longest suffix of `buf` (from `start`) that begins `needle`."""
    n = len(buf)
    for k in range(max(start, n - len(needle) + 1), n):
        if needle.startswith(bytes(buf[k:])):
            return n - k
    return 0
