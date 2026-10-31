"""One-call helpers: `show` (static output kept in the scrollback), `ask`
(one question, its answer returned) and `plain` (the plain-text fallback)."""

from __future__ import annotations

import json
import re
import sys
import unicodedata
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from typing import Final

from .keys import Key
from .reconcile import REGIONS, AnyView, Region, as_view
from .session import Session, connect
from .ui import Node
from .wire import ActionEvent, Font, Json


class Unsupported(RuntimeError):
    """`ask` can't ask: there is no TSP here, so the program asks its own way."""


@dataclass(frozen=True)
class Answer:
    """The `action` that submitted an `ask` form: node `id`, `act` and the form's `values`."""

    id: str | None
    act: str
    values: dict[str, Json] = field(default_factory=dict)
    event: ActionEvent | None = field(default=None, repr=False, compare=False)


def show(
    view: AnyView,
    *,
    css: str | None = None,
    fallback: str | None = None,
    font: Font | None = None,
    session: Session | None = None,
) -> None:
    """Shows a static view that stays in the scrollback like command output:
    a `flow` surface opened with `listen: false`, its sheet, one frame and a
    close with `keep`. Without TSP it prints `plain(view)`, or `fallback`.
    `font` is the surface's face (`"sans"` for app-like output)."""
    own = session is None
    if session is None:
        session = connect(paste=False, kitty=False)
    if session is None:
        text = fallback if fallback is not None else plain(view)
        sys.stdout.write(text if text.endswith("\n") or not text else text + "\n")
        sys.stdout.flush()
        return
    try:
        surface = session.open(mode="flow", listen=False, font=font)
        if css is not None:
            surface.stylesheet("main", css)
        surface.render(as_view(view))
        surface.close(keep=True)
    finally:
        if own:
            session.close()


def _dismisses(key: Key) -> bool:
    return key.name == "escape" or (key.ctrl and not key.alt and not key.meta and key.name in ("c", "d"))


def ask(
    view: AnyView,
    *,
    css: str | None = None,
    submit: str = "submit",
    font: Font | None = None,
    session: Session | None = None,
) -> Answer | None:
    """Shows a form in a `flow` surface and returns the first `action` whose
    `act` is `submit`, or `None` when the user presses Escape, Ctrl+C or
    Ctrl+D. The form stays in the scrollback; handlers on its nodes run while
    it waits. `font` is the surface's face. Raises `Unsupported` without TSP."""
    own = session is None
    if session is None:
        session = connect()
    if session is None:
        raise Unsupported("no Tern Surface Protocol on this terminal")
    try:
        with session.open(mode="flow", font=font) as surface:
            if css is not None:
                surface.stylesheet("main", css)
            surface.render(as_view(view))
            surface._submit = submit
            for item in session.input():
                if isinstance(item, Key):
                    if _dismisses(item):
                        return None
                elif isinstance(item, ActionEvent) and item.sf == surface.id and item.act == submit:
                    return Answer(item.id, submit, dict(item.values or {}), item)
            return None
    finally:
        if own:
            session.close()


# ---------------------------------------------------------------------------
# Plain text

_ANSI: Final = re.compile(r"\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[PX^_][^\x1b]*\x1b\\|[@-Z\\-_])")
_BLOCK_TAGS: Final = frozenset(
    "div p section header footer nav aside main article figure blockquote ul ol li dl dt dd "
    "h1 h2 h3 h4 pre hr table thead tbody tr form".split()
)


def plain(view: AnyView, cols: int = 80) -> str:
    """Renders a view as readable plain text without escapes (fallback output,
    not a copy of Tern's layout)."""
    regions = as_view(view)
    lines: list[str] = []
    if regions is not None:
        for name in REGIONS:
            region = regions.get(name)
            if region is not None:
                lines += _render(_wire(region), max(cols, 8))
    return "\n".join(line.rstrip() for line in lines)


def _wire(region: Region) -> Mapping[str, Json]:
    if isinstance(region, Node):
        return region.to_json()
    if isinstance(region, Mapping):
        return region
    return {"k": "col", "c": [n.to_json() for n in region]}


def _width(s: str) -> int:
    return sum(2 if unicodedata.east_asian_width(c) in "WF" else 0 if unicodedata.combining(c) else 1 for c in s)


def _pad(s: str, width: int, end: bool = False) -> str:
    gap = " " * max(width - _width(s), 0)
    return gap + s if end else s + gap


def _text(value: Json) -> str:
    """Plain text of a string or spans."""
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        out = []
        for s in value:
            if isinstance(s, str):
                out.append(s)
            elif isinstance(s, Mapping) and isinstance(s.get("t"), str) and "icon" not in str(s.get("s", "")).split():
                out.append(s["t"])
        return "".join(out)
    return ""


def _node_text(p: Mapping[str, Json]) -> str:
    return _text(p["spans"]) if isinstance(p.get("spans"), list) else _text(p.get("text"))


def _indent(lines: list[str], by: str = "  ") -> list[str]:
    return [by + line if line else line for line in lines]


def _bar(value: Json) -> str:
    width = 10
    if isinstance(value, (int, float)) and not isinstance(value, bool):
        v = min(max(float(value), 0.0), 1.0)
        filled = int(v * width + 0.5)
        return f"[{'#' * filled}{'-' * (width - filled)}] {int(v * 100 + 0.5)}%"
    return f"[{'-' * width}]"


def _table(rows: list[list[str]], ends: Sequence[bool] = ()) -> list[str]:
    if not rows:
        return []
    n = max(len(r) for r in rows)
    widths = [max((_width(r[i]) for r in rows if i < len(r)), default=0) for i in range(n)]
    out = []
    for r in rows:
        cells = [_pad(c, widths[i], i < len(ends) and ends[i]) for i, c in enumerate(r)]
        out.append("  ".join(cells))
    return out


def _children(node: Mapping[str, Json]) -> list[Mapping[str, Json]]:
    c = node.get("c")
    return [x for x in c if isinstance(x, Mapping)] if isinstance(c, list) else []


def _props(node: Mapping[str, Json]) -> Mapping[str, Json]:
    p = node.get("p")
    return p if isinstance(p, Mapping) else {}


def _block(lines: list[str], cols: int, children: list[Mapping[str, Json]]) -> list[str]:
    for child in children:
        lines += _render(child, cols)
    return lines


def _render(node: Mapping[str, Json], cols: int) -> list[str]:
    """A node as plain lines."""
    k = node.get("k")
    p = _props(node)
    kids = _children(node)
    if p.get("hidden") is True:
        return []
    match k:
        case "col" | "status":
            if k == "status":
                return ["  ".join(" ".join(_render(c, cols)) for c in kids)]
            return _block([], cols, kids)
        case "row" | "buttons":
            parts = [" ".join(line for line in _render(c, cols) if line) for c in kids]
            return [" ".join(part for part in parts if part)]
        case "split" | "scroll":
            return _block([], cols, kids)
        case "carousel":
            index = p.get("index")
            at = index if isinstance(index, int) and not isinstance(index, bool) else 0
            return _render(kids[min(max(at, 0), len(kids) - 1)], cols) if kids else []
        case "empty":
            return _block([x for x in (_text(p.get("title")), _text(p.get("text"))) if x], cols, kids)
        case "card" | "section" | "overlay":
            slots = {name: _slot(p, kids, name) for name in ("head", "end", "foot")}
            body_kids = [c for c in kids if all(c is not s for s in slots.values())]
            head_node = slots["head"]
            head = " ".join(_render(head_node, cols)) if head_node is not None else _text(p.get("head"))
            note = _text(p.get("meta"))
            end_node = slots["end"]
            end = " ".join(line for line in _render(end_node, cols) if line) if end_node is not None else ""
            head = "  ".join(x for x in (head, note, end) if x)
            status = p.get("status")
            if isinstance(status, str):
                head = f"{head} ({status})" if head else f"({status})"
            body = _indent(_block([], cols, body_kids))
            foot_node = slots["foot"]
            foot = _indent(_render(foot_node, cols)) if foot_node is not None else []
            return ([head] if head else []) + body + foot
        case "button":
            return [f"[{_text(p.get('text')) or _text(p.get('title'))}]"]
        case "stats":
            items = [i for i in p.get("items") or [] if isinstance(i, Mapping)]
            return ["  ·  ".join(_stat(i) for i in items)]
        case "rule":
            label = _text(p.get("label"))
            time = _text(p.get("time"))
            if time:
                label = f"{label} · {time}" if label else time
            if not label:
                return ["-" * cols]
            side = max((cols - _width(label) - 2) // 2, 2)
            return [f"{'-' * side} {label} {'-' * side}"]
        case "spacer":
            return [""]
        case "text" | "shimmer" | "seg":
            spans = p.get("spans")
            seg = _cited(spans) if k == "text" and isinstance(spans, list) else _node_text(p)
            if k == "seg" and kids:
                seg = " ".join([*(" ".join(_render(c, cols)) for c in kids), seg]).strip()
            return seg.split("\n")
        case "md" | "code" | "math" | "editor" | "input":
            text = _text(p.get("text"))
            if k == "input" and p.get("secret") is True:
                return ["•" * len(text)]
            return text.split("\n")
        case "ansi":
            return _ANSI.sub("", _text(p.get("text"))).split("\n")
        case "rows":
            lines = p.get("lines")
            return [_ANSI.sub("", s) for s in lines if isinstance(s, str)] if isinstance(lines, list) else []
        case "diff":
            hunks = p.get("hunks")
            if isinstance(hunks, list):
                return [
                    s for h in hunks if isinstance(h, Mapping) for s in (h.get("lines") or []) if isinstance(s, str)
                ]
            return _text(p.get("text")).split("\n")
        case "kv":
            items = [i for i in p.get("items") or [] if isinstance(i, Mapping)]
            pairs = [[_text(i.get("k")), _text(i.get("v"))] for i in items]
            if p.get("layout") == "inline":
                return [" · ".join(f"{a} {b}" for a, b in pairs)]
            return _table(pairs)
        case "table":
            return _plain_table(p, cols)
        case "tree":
            return _tree([n for n in p.get("nodes") or [] if isinstance(n, Mapping)])
        case "json":
            return json.dumps(p.get("value"), indent=2, ensure_ascii=False).split("\n")
        case "badge":
            return [f"[{_text(p.get('text'))}]"]
        case "kbd":
            return ["+".join(s for s in p.get("keys") or [] if isinstance(s, str))]
        case "icon":
            return []
        case "avatar":
            name = _text(p.get("text"))
            return [name] if name else []
        case "avatars":
            names = ", ".join(x for x in (_text(i.get("text")) for i in _items(p, "items")) if x)
            return [" ".join(x for x in (names, _text(p.get("label"))) if x)]
        case "attachment":
            return [f"[attachment: {_text(p.get('name'))}]"]
        case "image":
            return [f"[image: {_text(p.get('alt')) or _text(p.get('title')) or 'image'}]"]
        case "media":
            rev = _text(p.get("rev"))
            path = _text(p.get("path")) or "media"
            return [f"[media: {path} @ {rev}]" if rev else f"[media: {path}]"]
        case "list":
            out = _block([], cols, kids)
            if not out and p.get("empty") is not None:
                return [_text(p.get("empty"))]
            return out
        case "item":
            parts = [_text(p.get(f)) for f in ("label", "detail", "value")]
            return ["- " + "  ".join(x for x in parts if x)]
        case "tabs":
            active = p.get("active")
            tabs = [t for t in p.get("items") or [] if isinstance(t, Mapping)]
            return [
                " | ".join(
                    f"[{_text(t.get('label'))}]" if t.get("id") == active else _text(t.get("label")) for t in tabs
                )
            ]
        case "nav":
            active = p.get("active")
            navs = _items(p, "items")
            return [
                " | ".join(
                    f"[{_text(n.get('label'))}]"
                    if n.get("id") == active or any(s.get("id") == active for s in _items(n, "items"))
                    else _text(n.get("label"))
                    for n in navs
                )
            ]
        case "breadcrumb":
            sep = " / " if p.get("variant") == "path" else " › "
            return [sep.join(_text(c.get("label")) or str(c.get("id", "")) for c in _items(p, "items"))]
        case "pager":
            return [_pager(p)]
        case "menu":
            return [f"[{_text(p.get('text')) or _text(p.get('title'))}]"]
        case "menubar":
            return ["  ".join(_text(m.get("label")) or _text(m.get("id")) for m in _items(p, "menus"))]
        case "suggestions":
            return [f"- {_text(s.get('text'))}" for s in _items(p, "items") if _text(s.get("text"))]
        case "picker":
            title = _text(p.get("title"))
            items = [i for i in p.get("items") or [] if isinstance(i, Mapping)]
            return ([title] if title else []) + [f"- {_text(i.get('label')) or i.get('id')}" for i in items]
        case "select":
            label = _text(p.get("label"))
            shown = _select_shown(p)
            return [f"{label}: {shown}" if label else shown]
        case "switch":
            mark = "[x]" if p.get("on") is True else "[ ]"
            return [f"{mark} {_text(p.get('label'))}".rstrip()]
        case "slider":
            value = p.get("value")
            if isinstance(value, list):
                shown = "–".join(_number(v) for v in value[:2])
            else:
                shown = _number(value)
            symbol = _text(p.get("unit"))
            line = _with_unit(shown, symbol, symbol in _ATTACHED)
            label = _text(p.get("label"))
            return [f"{label}: {line}" if label and line else label or line]
        case "field":
            label = _text(p.get("label"))
            if label and p.get("required") is True:
                label += " *"
            out = _block([label] if label else [], cols, kids)
            note = _text(p.get("error")) or _text(p.get("warning")) or _text(p.get("hint"))
            return [*out, note] if note else out
        case "otp":
            return [_otp(p)]
        case "calendar" | "date":
            shown = _day_value(p.get("value"))
            if not shown and k == "date":
                shown = _text(p.get("placeholder"))
            return [shown] if shown else []
        case "skeleton":
            return []
        case "spinner":
            return [_text(p.get("label"))]
        case "elapsed":
            ms = p.get("stopped", p.get("age", 0))
            return [_elapsed(ms if isinstance(ms, (int, float)) else 0)]
        case "rate":
            unit = p.get("unit")
            return [f"{p.get('value', 0)}{' ' + unit if isinstance(unit, str) else ''}"]
        case "progress" | "meter":
            value = p.get("value")
            fills = p.get("parts")
            if value is None and isinstance(fills, list):
                value = sum(x.get("value", 0) for x in fills if isinstance(x, Mapping))
            line = _bar(value)
            extra = " ".join(x for x in (_text(p.get("label")), _text(p.get("total"))) if x)
            return [f"{line} {extra}".rstrip()]
        case "chart":
            summary = _text(p.get("summary"))
            return [summary] if summary else _chart(p)
        case "effort":
            return [f"effort: {p.get('level')}"] if isinstance(p.get("level"), str) else []
        case "toast":
            return [" ".join(x for x in (_text(p.get("text")), _text(p.get("sub"))) if x)]
        case "tool":
            head = " ".join(x for x in (_text(p.get("title")) or _text(p.get("name")), _text(p.get("target"))) if x)
            meta = [_text(m) for m in p.get("meta") or []]
            status = p.get("status")
            line = " ".join([head, *meta] + ([f"({status})"] if isinstance(status, str) else []))
            return [line] + _indent(_block([], cols, kids))
        case "trace":
            return _trace(p, kids, cols)
        case "message":
            return _message(p, kids, cols)
        case "stacktrace":
            return _stacktrace(p)
        case "questions":
            return _questions(p)
        case "agent":
            status = p.get("status")
            line = " ".join(
                x
                for x in (_text(p.get("name")), _text(p.get("task")), f"({status})" if isinstance(status, str) else "")
                if x
            )
            return [line] + _indent(_block([], cols, kids))
        case "checklist":
            return _checklist(p)
        case "prefs":
            return _prefs(p)
        case "el":
            return _el(node, cols)
        case _:
            return _block([], cols, kids)


def _slot(p: Mapping[str, Json], kids: list[Mapping[str, Json]], name: str) -> Mapping[str, Json] | None:
    """The child a `head`/`end`/`foot` prop names: by key (id-less nodes)
    or by id (wire nodes)."""
    ref = p.get(name)
    if not isinstance(ref, str):
        return None
    for child in kids:
        cid = child.get("id")
        key = _props(child).get("key")
        if cid == ref or (isinstance(key, str) and (ref == key or ref.endswith("." + key))):
            return child
    return None


def _items(p: Mapping[str, Json], key: str) -> list[Mapping[str, Json]]:
    """The mapping entries of an array prop."""
    value = p.get(key)
    return [x for x in value if isinstance(x, Mapping)] if isinstance(value, list) else []


def _number(value: Json) -> str:
    """A JSON number as written: integral floats without `.0`."""
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        return ""
    return str(int(value)) if isinstance(value, float) and value.is_integer() else str(value)


_ATTACHED: Final = ("%", "‰", "°", "×", "x", "′", "″", "'", '"')
# Currency symbols a donut writes before its numbers (`$51.4`).
_CURRENCY: Final = ("$", "€", "£", "¥")
"""Units a slider writes against its number (`72%`)."""


def _with_unit(value: str, unit: str, attached: bool) -> str:
    """A value and its unit, attached (`72%`) or after a space (`182 ms`)."""
    if not value or not unit:
        return value
    return f"{value}{unit}" if attached else f"{value} {unit}"


def _chart(p: Mapping[str, Json]) -> list[str]:
    """A `line`/`area` chart as a spark per series (`label spark` with
    several), a `donut` as `label value` segments."""
    kind = p.get("kind")
    series = p.get("series")
    entries = series if isinstance(series, list) else []
    if kind == "donut":
        unit = _text(p.get("unit"))
        attached = unit.startswith(("%", "×", "°"))
        segs = []
        for e in entries:
            seg = e if isinstance(e, Mapping) else {"value": e}
            number = _number(seg.get("value"))
            if unit in _CURRENCY and number:
                # A currency symbol leads the number, after any minus sign.
                sign, digits = ("−", number[1:]) if number[0] in "-−" else ("", number)
                value = f"{sign}{unit}{digits}"
            else:
                value = _with_unit(number, unit, attached)
            if value:
                segs.append(" ".join(x for x in (_text(seg.get("label")), value) if x))
        return ["  ".join(segs)] if segs else []
    if kind not in ("line", "area"):
        return []
    lines = [e for e in entries if isinstance(e, Mapping) and isinstance(e.get("values"), list)]
    if lines:
        out = []
        for s in lines:
            spark = _spark(_values(s["values"]))
            out.append(" ".join(x for x in (_text(s.get("label")), spark) if x))
        return out
    spark = _spark(_values(entries))
    return [spark] if spark else []


def _values(points: list[Json]) -> list[float]:
    """The numbers of chart points (numbers or `{value}`), gaps skipped."""
    out: list[float] = []
    for v in points:
        value = v.get("value") if isinstance(v, Mapping) else v
        if isinstance(value, (int, float)) and not isinstance(value, bool):
            out.append(value)
    return out


def _cited(spans: list[Json]) -> str:
    """Spans' text with `[n]` after each run citing sources: numbered in order
    of first `href` (a cite's own `n` wins); cites without `href` are ignored."""
    runs: list[tuple[str, list[str]]] = []
    numbers: dict[str, int] = {}
    order: list[str] = []
    for span in spans:
        cite = span.get("cite") if isinstance(span, Mapping) else None
        hrefs: list[str] = []
        for c in cite if isinstance(cite, list) else [cite]:
            if not isinstance(c, Mapping) or not isinstance(c.get("href"), str):
                continue
            href = c["href"]
            hrefs.append(href)
            if href not in order:
                order.append(href)
            n = c.get("n")
            if href not in numbers and isinstance(n, int) and not isinstance(n, bool) and n >= 1:
                numbers[href] = n
        runs.append((_text([span]), hrefs))
    used = set(numbers.values())
    free = 1
    for href in order:
        if href not in numbers:
            while free in used:
                free += 1
            numbers[href] = free
            used.add(free)
    return "".join(text + "".join(f"[{numbers[h]}]" for h in hrefs) for text, hrefs in runs)


def _select_shown(p: Mapping[str, Json]) -> str:
    """A select's shown value: the picked option's label, else the value,
    else the placeholder."""
    value = p.get("value")
    for option in p.get("options") or []:
        if isinstance(option, str) and option == value:
            return option
        if isinstance(option, Mapping) and option.get("value") == value:
            return _text(option.get("label")) or _text(value)
    return _text(value) or _text(p.get("placeholder"))


def _otp(p: Mapping[str, Json]) -> str:
    length = p.get("length")
    cells = length if isinstance(length, int) and not isinstance(length, bool) else 6
    text = _text(p.get("text"))[:cells]
    mask = p.get("mask")
    if mask is True:
        text = "•" * len(text)
    elif isinstance(mask, str) and mask:
        text = mask[0] * len(text)
    return text + "_" * (cells - len(text))


def _day_value(value: Json) -> str:
    """A calendar value: a day, or a range as `from – to`."""
    if isinstance(value, list):
        days = [d for d in value[:2] if isinstance(d, str)]
        return " – ".join(days)
    return _text(value)


def _pager(p: Mapping[str, Json]) -> str:
    page = p.get("page")
    current = page if isinstance(page, int) and not isinstance(page, bool) else 1
    pages = _count(p.get("pages"))
    total, size = _count(p.get("total")), _count(p.get("size"))
    if pages is None and total is not None and size is not None and size > 0:
        pages = -(-total // size)
    return f"Page {current} of {pages}" if pages else f"Page {current}"


def _count(value: Json) -> int | None:
    return value if isinstance(value, int) and not isinstance(value, bool) else None


def _trace(p: Mapping[str, Json], kids: list[Mapping[str, Json]], cols: int) -> list[str]:
    """A trace: its head (with its status), then a `- title target (status)`
    row per line with the line's content child (keyed by its id, else its
    index) indented under it, then the other children."""
    out: list[str] = []
    status = p.get("status")
    head = _text(p.get("title")) or _text(p.get("summary"))
    if isinstance(status, str):
        head = f"{head} ({status})" if head else f"({status})"
    if head:
        out.append(head)
    placed: list[Mapping[str, Json]] = []
    for index, line in enumerate(_items(p, "lines")):
        state = line.get("status")
        parts = [_text(line.get("title")), _text(line.get("target"))]
        row = " ".join([x for x in parts if x] + ([f"({state})"] if isinstance(state, str) else []))
        out.append(f"- {row}")
        lid = line.get("id")
        child = _slot({"line": lid if isinstance(lid, str) else str(index)}, kids, "line")
        if child is not None:
            placed.append(child)
            out += _indent(_render(child, cols), "    ")
    return _block(out, cols, [c for c in kids if all(c is not x for x in placed)])


def _message(p: Mapping[str, Json], kids: list[Mapping[str, Json]], cols: int) -> list[str]:
    """A message: `author time`, then its body, reactions and status indented."""
    avatar = _slot(p, kids, "avatar")
    head = " ".join(x for x in (_text(p.get("author")), _text(p.get("time"))) if x)
    body = _block([], cols, [c for c in kids if c is not avatar])
    chips = []
    for r in _items(p, "reactions"):
        n = r.get("count")
        count = _number(n) if isinstance(n, (int, float)) and n > 0 else ""
        chips.append(" ".join(x for x in (_text(r.get("text")), count) if x))
    if any(chips):
        body.append("  ".join(x for x in chips if x))
    status = _text(p.get("status"))
    if status:
        body.append(status)
    return ([head] if head else []) + _indent(body)


def _stacktrace(p: Mapping[str, Json]) -> list[str]:
    """The error as `type: message`, then `  at fn (file:line:col)` per frame."""
    error = p.get("error")
    err = error if isinstance(error, Mapping) else {}
    head = ": ".join(x for x in (_text(err.get("type")), _text(err.get("message"))) if x)
    out = [head] if head else []
    for frame in _items(p, "frames"):
        fn = _text(frame.get("fn")) or "<anonymous>"
        file = _text(frame.get("file"))
        where = ":".join(x for x in (file, _number(frame.get("line")), _number(frame.get("col"))) if x)
        out.append(f"  at {fn} ({where})" if file else f"  at {fn}")
    return out


def _questions(p: Mapping[str, Json]) -> list[str]:
    """Each question's prompt, then a `  - label` row per choice."""
    out: list[str] = []
    for q in _items(p, "items"):
        if not isinstance(q.get("id"), str):
            continue
        out.append(_text(q.get("prompt")))
        for choice in q.get("choices") or []:
            if isinstance(choice, str):
                out.append(f"  - {choice}")
            elif isinstance(choice, Mapping):
                out.append(f"  - {_text(choice.get('label')) or _text(choice.get('value'))}")
    return out


def _stat(item: Mapping[str, Json]) -> str:
    sub = _text(item.get("sub"))
    line = f"{_text(item.get('label'))}: {_text(item.get('value'))}"
    return f"{line} ({sub})" if sub else line


def _cell(cell: Json) -> str:
    """Plain text of one table cell."""
    if not isinstance(cell, Mapping):
        return _text(cell)
    if isinstance(cell.get("meter"), Mapping):
        meter = cell["meter"]
        value = meter.get("value")
        parts = meter.get("parts")
        if value is None and isinstance(parts, list):
            value = sum(part.get("value", 0) for part in parts if isinstance(part, Mapping))
        return _bar(value)
    if isinstance(cell.get("badge"), Mapping):
        return f"[{_text(cell['badge'].get('text'))}]"
    if isinstance(cell.get("button"), Mapping):
        return f"[{_text(cell['button'].get('text'))}]"
    if isinstance(cell.get("spark"), Mapping):
        values = [v for v in cell["spark"].get("values") or [] if isinstance(v, (int, float))]
        return _spark(values)
    text = _text(cell.get("text"))
    return f"● {text}" if cell.get("dot") is not None else text


_SPARK: Final = "▁▂▃▄▅▆▇█"


def _spark(values: Sequence[float]) -> str:
    if not values:
        return ""
    lo, hi = min(values), max(values)
    span = hi - lo or 1
    return "".join(_SPARK[int((v - lo) / span * (len(_SPARK) - 1) + 0.5)] for v in values)


def _elapsed(ms: float) -> str:
    s = abs(ms) / 1000
    if s < 60:
        return f"{int(s * 10) / 10:.1f}s"
    if s < 3600:
        return f"{int(s // 60)}m {int(s % 60):02d}s"
    return f"{int(s // 3600)}h {int(s % 3600 // 60):02d}m"


def _plain_table(p: Mapping[str, Json], cols: int) -> list[str]:
    columns = [c for c in p.get("cols") or [] if isinstance(c, Mapping) and isinstance(c.get("id"), str)]
    rows = [r for r in p.get("rows") or [] if isinstance(r, Mapping)]
    out: list[list[str]] = []
    if any(_text(c.get("head")) for c in columns):
        out.append([_text(c.get("head")) for c in columns])
    if not rows and p.get("empty") is not None:
        return [*_table(out), _text(p.get("empty"))]
    selected = p.get("selected")
    marks = selected is not None
    if marks and out:
        out[0] = ["", *out[0]]
    for r in rows:
        cells = r.get("cells") if isinstance(r.get("cells"), Mapping) else {}
        line = [_cell(cells.get(c["id"]) if isinstance(cells, Mapping) else None) for c in columns]
        out.append([">" if r.get("id") == selected else "", *line] if marks else line)
    ends = [c.get("align") == "end" for c in columns]
    return _table(out, [False, *ends] if marks else ends)


def _tree(nodes: list[Mapping[str, Json]], depth: int = 0) -> list[str]:
    out = []
    for n in nodes:
        out.append("  " * depth + "- " + _text(n.get("label")))
        kids = n.get("children")
        if isinstance(kids, list):
            out += _tree([c for c in kids if isinstance(c, Mapping)], depth + 1)
    return out


_CHECK: Final = {"done": "[x]", "dropped": "[-]", "active": "[>]", "blocked": "[!]"}


def _checklist(p: Mapping[str, Json]) -> list[str]:
    out = []
    for phase in p.get("phases") or []:
        if not isinstance(phase, Mapping):
            continue
        title = _text(phase.get("title"))
        if title:
            out.append(title)
        for item in phase.get("items") or []:
            if isinstance(item, Mapping):
                mark = _CHECK.get(str(item.get("status")), "[ ]")
                out.append(("  " if title else "") + f"{mark} {_text(item.get('text'))}")
    return out


def _prefs(p: Mapping[str, Json]) -> list[str]:
    out = [_text(p.get("title")) or "Settings"]
    for section in p.get("sections") or []:
        if not isinstance(section, Mapping):
            continue
        if _text(section.get("title")):
            out.append(_text(section.get("title")))
        for row in section.get("rows") or []:
            if not isinstance(row, Mapping):
                continue
            control = row.get("control") if isinstance(row.get("control"), Mapping) else {}
            value = control.get("value", control.get("on", "")) if isinstance(control, Mapping) else ""
            out.append(f"  {_text(row.get('label')) or row.get('id')}: {value}")
    return out


def _el_inline(node: Mapping[str, Json]) -> str:
    p = _props(node)
    if node.get("k") != "el":
        return " ".join(_render(node, 80))
    tag = p.get("tag", "div")
    if tag == "input":
        on = p.get("checked") is True
        return ("(*)" if on else "( )") if p.get("type") == "radio" else ("[x]" if on else "[ ]")
    text = _text(p.get("text"))
    control = False
    for child in _children(node):
        piece = _el_inline(child)
        if control and piece and text:
            text += " "
        text += piece
        control = child.get("k") == "el" and _props(child).get("tag") == "input"
    if tag == "button":
        return f"[ {text.strip()} ]"
    return text


def _el(node: Mapping[str, Json], cols: int) -> list[str]:
    p = _props(node)
    tag = p.get("tag", "div")
    if tag not in _BLOCK_TAGS:
        return [_el_inline(node)]
    if tag == "hr":
        return ["-" * cols]
    if tag in ("table", "thead", "tbody"):
        rows: list[list[str]] = []
        _collect_rows(node, rows)
        return _table(rows)
    if tag == "tr":
        return ["  ".join(_el_inline(c) for c in _children(node))]
    lines: list[str] = []
    cur = _text(p.get("text"))
    ordered = 0
    for child in _children(node):
        cp = _props(child)
        if child.get("k") == "el" and cp.get("tag", "div") not in _BLOCK_TAGS:
            piece = _el_inline(child)
            cur = f"{cur} {piece}" if cur and piece and not cur.endswith(" ") else cur + piece
            continue
        if cur:
            lines.append(cur)
            cur = ""
        sub = _render(child, cols)
        if child.get("k") == "el" and cp.get("tag") == "li":
            ordered += 1
            bullet = f"{ordered}. " if tag == "ol" else "- "
            sub = [bullet + sub[0], *_indent(sub[1:])] if sub else [bullet.rstrip()]
        lines += sub
    if cur:
        lines.append(cur)
    return _indent(lines) if tag in ("blockquote", "dd", "figure") else lines


def _collect_rows(node: Mapping[str, Json], rows: list[list[str]]) -> None:
    for child in _children(node):
        tag = _props(child).get("tag")
        if tag == "tr":
            rows.append([_el_inline(c) for c in _children(child)])
        elif tag in ("thead", "tbody"):
            _collect_rows(child, rows)
