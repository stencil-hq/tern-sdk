"""TSP wire layer: constants, typed messages in both directions, the encoder
(framing, chunking, blobs) and the decoder of replies and events."""

from __future__ import annotations

import base64
import hashlib
import json
from collections.abc import Callable, Iterator, Mapping, Sequence
from dataclasses import dataclass, field
from typing import Any, Final, Literal, TypeAlias

VERSION: Final = 1
"""The protocol version this SDK speaks."""
APC_LIMIT: Final = 65536
"""The largest body sent in one message unless the hello reply says otherwise."""
CREDITS: Final = 2
"""Unacknowledged frames allowed unless the hello reply says otherwise."""
MAX_BLOB: Final = 16 << 20
"""The largest blob, decoded, that Tern accepts."""

KINDS: Final = (
    "col", "row", "card", "section", "rule", "spacer", "text", "md", "code", "diff", "ansi",
    "math", "image", "kv", "table", "tree", "badge", "kbd", "icon", "spinner", "shimmer",
    "elapsed", "progress", "rate", "list", "item", "tabs", "editor", "input", "status", "seg",
    "overlay", "toast", "rows", "picker", "prefs", "tool", "checklist", "agent", "chart",
    "meter", "block", "effort", "el",
)  # fmt: skip
"""Every node kind of protocol version 1."""
TEXT_KINDS: Final = frozenset(("text", "md", "code", "ansi", "math", "editor", "input", "shimmer", "el"))
"""Kinds whose `text` prop is primary text, updated with the `text` op."""
PROGRAM_FEATURES: Final = ("edit", "undo", "send")
"""Features a program may announce in its `hello`."""
TERMINAL_FEATURES: Final = (
    "blobs", "settle", "adopt", "dock", "program-palette", "reduce-motion", "aside",
    "scroll", "styles", "flow",
)  # fmt: skip
"""Features a terminal may list in its `hello` reply."""

Kind: TypeAlias = Literal[
    "col", "row", "card", "section", "rule", "spacer", "text", "md", "code", "diff", "ansi",
    "math", "image", "kv", "table", "tree", "badge", "kbd", "icon", "spinner", "shimmer",
    "elapsed", "progress", "rate", "list", "item", "tabs", "editor", "input", "status", "seg",
    "overlay", "toast", "rows", "picker", "prefs", "tool", "checklist", "agent", "chart",
    "meter", "block", "effort", "el",
]  # fmt: skip
"""A node kind."""
ProgramFeature: TypeAlias = Literal["edit", "undo", "send"]
"""A program feature of the `hello` query."""
Mode: TypeAlias = Literal["inline", "screen", "flow"]
"""A surface mode."""

Json: TypeAlias = Any
"""A JSON value as `json.loads` returns it."""
Op: TypeAlias = list[Json]
"""One frame op, e.g. `["del", "main.0"]`."""

ESC: Final = 0x1B
BEL: Final = 0x07
ST: Final = b"\x1b\\"


def dumps(value: Json) -> str:
    """Compact JSON, as every TSP body is written."""
    return json.dumps(value, separators=(",", ":"), ensure_ascii=False, allow_nan=False)


# ---------------------------------------------------------------------------
# Framing


def _key_byte(b: int) -> bool:
    return 0x30 <= b <= 0x39 or 0x41 <= b <= 0x5A or 0x61 <= b <= 0x7A or b in (0x5F, 0x2D)


def _value_byte(b: int) -> bool:
    return 0x21 <= b <= 0x7E and b != 0x3B


def _parameter(segment: bytes) -> tuple[str, str] | None:
    eq = segment.find(b"=")
    if eq <= 0:
        return None
    key, value = segment[:eq], segment[eq + 1 :]
    if all(_key_byte(b) for b in key) and all(_value_byte(b) for b in value):
        return key.decode("ascii"), value.decode("ascii")
    return None


@dataclass(frozen=True, slots=True)
class Message:
    """One TSP message as framed: verb, `key=value` parameters and raw body."""

    verb: str
    params: tuple[tuple[str, str], ...]
    body: bytes


def split(inner: bytes) -> Message | None:
    """Splits the data after `tsp;` into a message, `None` when it has no verb."""
    semi = inner.find(b";")
    if semi < 0:
        return Message(inner.decode("utf-8", "replace"), (), b"") if inner else None
    if semi == 0:
        return None
    verb = inner[:semi].decode("utf-8", "replace")
    params: list[tuple[str, str]] = []
    pos = semi + 1
    while True:
        end = inner.find(b";", pos)
        if end < 0:
            break
        param = _parameter(inner[pos:end])
        if param is None:
            break
        params.append(param)
        pos = end + 1
    return Message(verb, tuple(params), inner[pos:])


def _leads_with_parameter(body: bytes, at: int) -> bool:
    n = len(body)
    i = at
    while i < n and _key_byte(body[i]):
        i += 1
    if i == at or i >= n or body[i] != 0x3D:
        return False
    i += 1
    while i < n and _value_byte(body[i]):
        i += 1
    return i < n and body[i] == 0x3B


def _chunk_end(body: bytes, start: int, limit: int) -> int:
    target = start + max(limit, 1)
    n = len(body)
    if target >= n:
        return n

    def safe(at: int) -> bool:
        return body[at] & 0xC0 != 0x80 and not _leads_with_parameter(body, at)

    for at in range(target, start, -1):
        if safe(at):
            return at
    for at in range(target + 1, n):
        if safe(at):
            return at
    return n


def chunks(body: bytes, limit: int) -> Iterator[bytes]:
    """Splits a body into chunk bodies exactly as Tern's `tsp_chunks` does."""
    start = 0
    while start < len(body):
        end = _chunk_end(body, start, limit)
        yield body[start:end]
        start = end


def encode(
    verb: str,
    body: bytes,
    params: Sequence[tuple[str, str]] = (),
    *,
    limit: int = APC_LIMIT,
    chunk: str | Callable[[], str] = "c",
) -> bytes:
    """Frames one message as APC strings, chunked when `body` is over `limit`.

    `chunk` is the chunk id, or a function giving one; it is used only when
    the body has to be chunked.
    """
    head = "".join(f"{k}={v};" for k, v in params).encode("ascii")
    prefix = b"\x1b_tsp;" + verb.encode("ascii") + b";"
    if len(body) <= limit:
        return prefix + head + body + ST
    cid = (chunk() if callable(chunk) else chunk).encode("ascii")
    pieces = list(chunks(body, limit))
    out = bytearray()
    for i, piece in enumerate(pieces):
        out += prefix
        if i == 0:
            out += head
        out += b"c=" + cid + b";"
        if i < len(pieces) - 1:
            out += b"m=1;"
        out += piece + ST
    return bytes(out)


class ChunkIds:
    """A per-session base-36 counter of chunk ids."""

    def __init__(self) -> None:
        self._n = 0

    def __call__(self) -> str:
        self._n += 1
        n, digits = self._n, ""
        while n:
            n, r = divmod(n, 36)
            digits = "0123456789abcdefghijklmnopqrstuvwxyz"[r] + digits
        return digits


# ---------------------------------------------------------------------------
# Program -> terminal messages


@dataclass(frozen=True, slots=True)
class Outgoing:
    """A program -> terminal message: verb, parameters and JSON (or base64) payload."""

    verb: str
    payload: Json
    params: tuple[tuple[str, str], ...] = ()

    def body(self) -> bytes:
        """The body bytes: the base64 text of a blob, else compact JSON."""
        if isinstance(self.payload, str) and self.verb == "b":
            return self.payload.encode("ascii")
        return dumps(self.payload).encode("utf-8")

    def encode(self, *, limit: int = APC_LIMIT, chunk: str | Callable[[], str] = "c") -> bytes:
        """The message framed as one or more APC strings."""
        return encode(self.verb, self.body(), self.params, limit=limit, chunk=chunk)


def _drop_none(pairs: Mapping[str, Json]) -> dict[str, Json]:
    return {k: v for k, v in pairs.items() if v is not None}


def hello(
    app: str | None = None,
    *,
    ver: str | None = None,
    features: Sequence[str] = (),
    versions: Sequence[int] = (VERSION,),
) -> Outgoing:
    """The `hello` query (`q`)."""
    body = _drop_none({"q": "hello", "v": list(versions), "app": app, "ver": ver, "features": list(features) or None})
    return Outgoing("q", body)


def blobs_query(ids: Sequence[str]) -> Outgoing:
    """The `blobs` query (`q`): which of these blobs does Tern hold."""
    return Outgoing("q", {"q": "blobs", "ids": list(ids)})


def open_surface(
    id: str,
    *,
    mode: Mode | None = None,
    title: str | None = None,
    role: str | None = None,
    listen: bool | None = None,
    adopt: bool | None = None,
) -> Outgoing:
    """Opens a surface (`o`)."""
    body = _drop_none({"id": id, "mode": mode, "title": title, "role": role, "listen": listen, "adopt": adopt})
    return Outgoing("o", body)


def frame(sf: str, s: int, ops: Sequence[Op]) -> Outgoing:
    """A frame (`f`): an atomic batch of ops for surface `sf`, sequence `s`."""
    return Outgoing("f", {"sf": sf, "s": s, "ops": list(ops)})


def blob(data: bytes, mime: str | None = None) -> Outgoing:
    """A blob (`b`): `id` the lowercase hex SHA-256, body standard base64.

    Raises `ValueError` for more than 16 MiB.
    """
    if len(data) > MAX_BLOB:
        raise ValueError(f"blob of {len(data)} bytes is over {MAX_BLOB}")
    params = [("id", hashlib.sha256(data).hexdigest())]
    if mime is not None:
        params.append(("mime", mime))
    return Outgoing("b", base64.b64encode(data).decode("ascii"), tuple(params))


def palette(
    sf: str | None = None,
    *,
    dark: Mapping[str, str] | None = None,
    light: Mapping[str, str] | None = None,
    name: Mapping[str, str] | None = None,
) -> Outgoing:
    """The program palette (`t`)."""
    body = _drop_none(
        {
            "sf": sf,
            "dark": dict(dark) if dark is not None else None,
            "light": dict(light) if light is not None else None,
            "name": dict(name) if name is not None else None,
        }
    )
    return Outgoing("t", body)


def stylesheet(sf: str | None, name: str, css: str | None) -> Outgoing:
    """Installs, replaces or (with `css` None) removes a stylesheet (`s`)."""
    return Outgoing("s", _drop_none({"sf": sf, "name": name, "css": css}))


def close_surface(id: str, *, keep: bool = True) -> Outgoing:
    """Closes a surface (`x`), keeping its `main` in the scrollback with `keep`."""
    return Outgoing("x", {"id": id, "keep": keep})


RevealAt: TypeAlias = Literal["start", "end", "nearest"]
"""Where `reveal` puts a node."""
ScrollBy: TypeAlias = Literal["line-up", "line-down", "page-up", "page-down", "start", "end"]
"""How `scroll` moves a scroll container."""


class Ops:
    """Builders of the twelve frame ops."""

    @staticmethod
    def add(id: str, parent: str, before: str | None, node: Mapping[str, Json]) -> Op:
        """Insert `node` (with ids) under `parent`, before `before` or last."""
        return ["add", id, parent, before, dict(node)]

    @staticmethod
    def set(id: str, props: Mapping[str, Json]) -> Op:
        """Merge props shallowly; a `None` value deletes the prop."""
        return ["set", id, dict(props)]

    @staticmethod
    def text(id: str, how: Literal["append", "replace"], text: str) -> Op:
        """Append to or replace a text kind's primary text."""
        return ["text", id, how, text]

    @staticmethod
    def splice(id: str, at: int, delete: int, text: str) -> Op:
        """Replace `delete` UTF-16 units at `at` in the primary text."""
        return ["splice", id, at, delete, text]

    @staticmethod
    def move(id: str, parent: str, before: str | None) -> Op:
        """Reparent or reorder a subtree."""
        return ["move", id, parent, before]

    @staticmethod
    def delete(id: str) -> Op:
        """Remove a subtree (the `del` op)."""
        return ["del", id]

    @staticmethod
    def settle(id: str) -> Op:
        """Hint that a subtree is unlikely to change soon."""
        return ["settle", id]

    @staticmethod
    def focus(id: str | None) -> Op:
        """Give the caret to an `editor` or `input`, or to none."""
        return ["focus", id]

    @staticmethod
    def reveal(id: str, at: RevealAt = "nearest") -> Op:
        """Scroll a node into view."""
        return ["reveal", id, at]

    @staticmethod
    def scroll(id: str, by: ScrollBy) -> Op:
        """Scroll the nearest scroll container at or above a node."""
        return ["scroll", id, by]

    @staticmethod
    def suspend() -> Op:
        """Hand the pane back to the grid."""
        return ["suspend"]

    @staticmethod
    def resume() -> Op:
        """Take the pane again after `suspend`."""
        return ["resume"]


# ---------------------------------------------------------------------------
# Terminal -> program messages


def _str(raw: Mapping[str, Json], key: str) -> str | None:
    v = raw.get(key)
    return v if isinstance(v, str) else None


def _int(raw: Mapping[str, Json], key: str) -> int | None:
    v = raw.get(key)
    if isinstance(v, bool):
        return None
    if isinstance(v, int):
        return v
    if isinstance(v, float) and v.is_integer():
        return int(v)
    return None


def _num(raw: Mapping[str, Json], key: str) -> float | None:
    v = raw.get(key)
    if isinstance(v, bool) or not isinstance(v, (int, float)):
        return None
    return v


def _bool(raw: Mapping[str, Json], key: str) -> bool | None:
    v = raw.get(key)
    return v if isinstance(v, bool) else None


def _strs(raw: Mapping[str, Json], key: str) -> tuple[str, ...]:
    v = raw.get(key)
    if not isinstance(v, list):
        return ()
    return tuple(x for x in v if isinstance(x, str))


def _obj(raw: Mapping[str, Json], key: str) -> dict[str, Json] | None:
    v = raw.get(key)
    return v if isinstance(v, dict) else None


@dataclass(frozen=True, slots=True)
class Cell:
    """A cell's size in pixels."""

    w: float
    h: float


def _cell(raw: Mapping[str, Json]) -> Cell | None:
    c = _obj(raw, "cell")
    if c is None:
        return None
    w, h = _num(c, "w"), _num(c, "h")
    return Cell(w, h) if w is not None and h is not None else None


@dataclass(frozen=True, kw_only=True)
class Reply:
    """A reply (`r`) to a query; `raw` is its JSON object."""

    raw: dict[str, Json] = field(repr=False)
    r: str | None


@dataclass(frozen=True, kw_only=True)
class HelloReply(Reply):
    """The `hello` reply: version, vocabulary, features and pane state."""

    v: int | None
    term: str | None
    ver: str | None
    kinds: tuple[str, ...]
    features: tuple[str, ...]
    apc: int | None
    credits: int | None
    cols: int | None
    cell: Cell | None
    dark: bool | None
    reduce_motion: bool | None
    hour12: bool | None


@dataclass(frozen=True, kw_only=True)
class BlobsReply(Reply):
    """The `blobs` reply: the asked ids Tern holds."""

    have: tuple[str, ...]


@dataclass(frozen=True, kw_only=True)
class UnknownReply(Reply):
    """A reply this SDK has no type for, kept raw."""


def decode_reply(raw: dict[str, Json]) -> Reply:
    """Types a reply object; unknown replies and fields never fail."""
    r = _str(raw, "r")
    if r == "hello":
        return HelloReply(
            raw=raw,
            r=r,
            v=_int(raw, "v"),
            term=_str(raw, "term"),
            ver=_str(raw, "ver"),
            kinds=_strs(raw, "kinds"),
            features=_strs(raw, "features"),
            apc=_int(raw, "apc"),
            credits=_int(raw, "credits"),
            cols=_int(raw, "cols"),
            cell=_cell(raw),
            dark=_bool(raw, "dark"),
            reduce_motion=_bool(raw, "reduceMotion"),
            hour12=_bool(raw, "hour12"),
        )
    if r == "blobs":
        return BlobsReply(raw=raw, r=r, have=_strs(raw, "have"))
    return UnknownReply(raw=raw, r=r)


Mod: TypeAlias = Literal["shift", "ctrl", "alt", "meta"]
"""A modifier named in an `action` event's `mods`."""


@dataclass(frozen=True, kw_only=True)
class Event:
    """An event (`e`); `raw` is its JSON object, `sf` the surface it is about."""

    raw: dict[str, Json] = field(repr=False)
    ev: str | None
    sf: str | None


@dataclass(frozen=True, kw_only=True)
class AckEvent(Event):
    """Every frame up to `s` is applied and drawn."""

    s: int | None


@dataclass(frozen=True, kw_only=True)
class ResizeEvent(Event):
    """The pane's width or cell size changed, or the surface was first drawn."""

    cols: int | None
    cell: Cell | None
    visible: bool | None


@dataclass(frozen=True, kw_only=True)
class ThemeEvent(Event):
    """The appearance switched."""

    dark: bool | None


@dataclass(frozen=True, kw_only=True)
class MotionEvent(Event):
    """Reduce Motion was toggled."""

    reduce: bool | None


@dataclass(frozen=True, kw_only=True)
class VisibleEvent(Event):
    """The pane was hidden or shown."""

    visible: bool | None


@dataclass(frozen=True, kw_only=True)
class ToggleEvent(Event):
    """A collapsible node (or a tree item, `key`) was folded or unfolded."""

    id: str | None
    collapsed: bool | None
    key: str | None


@dataclass(frozen=True, kw_only=True)
class SelectEvent(Event):
    """A `select` action: `id` the node or list, `item` the item."""

    id: str | None
    item: str | None
    values: dict[str, Json] | None


@dataclass(frozen=True, kw_only=True)
class ActivateEvent(Event):
    """An `activate` action: `id` the node or list, `item` the item."""

    id: str | None
    item: str | None
    values: dict[str, Json] | None


@dataclass(frozen=True, kw_only=True)
class ActionEvent(Event):
    """A custom pointer action `act` (with `value` after `=`) on node `id`."""

    id: str | None
    act: str | None
    value: str | None
    mods: tuple[str, ...]
    values: dict[str, Json] | None


@dataclass(frozen=True, kw_only=True)
class ChangeEvent(Event):
    """An `el` checkbox or radio flipped, or a `prefs` row's value changed (`item`)."""

    id: str | None
    value: Json
    checked: bool | None
    name: str | None
    item: str | None
    values: dict[str, Json] | None


@dataclass(frozen=True, kw_only=True)
class FocusEvent(Event):
    """A click asked for the keys in field `id`."""

    id: str | None


@dataclass(frozen=True, kw_only=True)
class EditEvent(Event):
    """Native editing: replace UTF-16 `[from_, to)` of field `id` with `text`."""

    id: str | None
    from_: int | None
    to: int | None
    text: str | None
    cursor: int | None
    len: int | None


@dataclass(frozen=True, kw_only=True)
class UndoEvent(Event):
    """Undo the last change to field `id`."""

    id: str | None


@dataclass(frozen=True, kw_only=True)
class SendEvent(Event):
    """Submit `text` through composer `id`."""

    id: str | None
    text: str | None


@dataclass(frozen=True, kw_only=True)
class ErrorEvent(Event):
    """A rejected op, message, stylesheet or `el`."""

    msg: str | None
    s: int | None
    op: int | None
    sheet: str | None
    id: str | None


@dataclass(frozen=True, kw_only=True)
class GoneEvent(Event):
    """Tern dropped these nodes or surfaces."""

    ids: tuple[str, ...]


@dataclass(frozen=True, kw_only=True)
class UnknownEvent(Event):
    """An event this SDK has no type for, kept raw."""


def decode_event(raw: dict[str, Json]) -> Event:
    """Types an event object; unknown events and fields never fail."""
    ev, sf = _str(raw, "ev"), _str(raw, "sf")
    base: dict[str, Any] = {"raw": raw, "ev": ev, "sf": sf}
    match ev:
        case "ack":
            return AckEvent(**base, s=_int(raw, "s"))
        case "resize":
            return ResizeEvent(**base, cols=_int(raw, "cols"), cell=_cell(raw), visible=_bool(raw, "visible"))
        case "theme":
            return ThemeEvent(**base, dark=_bool(raw, "dark"))
        case "motion":
            return MotionEvent(**base, reduce=_bool(raw, "reduce"))
        case "visible":
            return VisibleEvent(**base, visible=_bool(raw, "visible"))
        case "toggle":
            return ToggleEvent(**base, id=_str(raw, "id"), collapsed=_bool(raw, "collapsed"), key=_str(raw, "key"))
        case "select":
            return SelectEvent(**base, id=_str(raw, "id"), item=_str(raw, "item"), values=_obj(raw, "values"))
        case "activate":
            return ActivateEvent(**base, id=_str(raw, "id"), item=_str(raw, "item"), values=_obj(raw, "values"))
        case "action":
            return ActionEvent(
                **base,
                id=_str(raw, "id"),
                act=_str(raw, "act"),
                value=_str(raw, "value"),
                mods=_strs(raw, "mods"),
                values=_obj(raw, "values"),
            )
        case "change":
            return ChangeEvent(
                **base,
                id=_str(raw, "id"),
                value=raw.get("value"),
                checked=_bool(raw, "checked"),
                name=_str(raw, "name"),
                item=_str(raw, "item"),
                values=_obj(raw, "values"),
            )
        case "focus":
            return FocusEvent(**base, id=_str(raw, "id"))
        case "edit":
            return EditEvent(
                **base,
                id=_str(raw, "id"),
                from_=_int(raw, "from"),
                to=_int(raw, "to"),
                text=_str(raw, "text"),
                cursor=_int(raw, "cursor"),
                len=_int(raw, "len"),
            )
        case "undo":
            return UndoEvent(**base, id=_str(raw, "id"))
        case "send":
            return SendEvent(**base, id=_str(raw, "id"), text=_str(raw, "text"))
        case "error":
            return ErrorEvent(
                **base,
                msg=_str(raw, "msg"),
                s=_int(raw, "s"),
                op=_int(raw, "op"),
                sheet=_str(raw, "sheet"),
                id=_str(raw, "id"),
            )
        case "gone":
            return GoneEvent(**base, ids=_strs(raw, "ids"))
        case _:
            return UnknownEvent(**base)
