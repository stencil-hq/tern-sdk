"""One-call helpers: `show` (static output kept in the scrollback), `ask`
(one question, its answer returned) and `plain` (the plain-text fallback)."""

from __future__ import annotations

import re
import sys
import unicodedata
from collections.abc import Mapping, Sequence
from dataclasses import dataclass, field
from typing import Final

from .keys import Key
from .reconcile import REGIONS, AnyView, Region, as_view
from .session import Session, available, connect
from .ui import Node
from .wire import ActionEvent, Json


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
    session: Session | None = None,
) -> None:
    """Shows a static view that stays in the scrollback like command output:
    a `flow` surface opened with `listen: false`, its sheet, one frame and a
    close with `keep`. Without TSP it prints `plain(view)`, or `fallback`."""
    own = session is None
    if session is None:
        session = connect(paste=False, kitty=False) if available() else None
    if session is None:
        text = fallback if fallback is not None else plain(view)
        sys.stdout.write(text if text.endswith("\n") or not text else text + "\n")
        sys.stdout.flush()
        return
    try:
        surface = session.open(mode="flow", listen=False)
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
    session: Session | None = None,
) -> Answer | None:
    """Shows a form in a `flow` surface and returns the first `action` whose
    `act` is `submit`, or `None` when the user presses Escape, Ctrl+C or
    Ctrl+D. The form stays in the scrollback; handlers on its nodes run while
    it waits. Raises `Unsupported` without TSP."""
    own = session is None
    if session is None:
        session = connect() if available() else None
    if session is None:
        raise Unsupported("no Tern Surface Protocol on this terminal")
    try:
        with session.open(mode="flow") as surface:
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
        case "row":
            parts = [" ".join(line for line in _render(c, cols) if line) for c in kids]
            return [" ".join(part for part in parts if part)]
        case "card" | "section" | "overlay":
            head = _text(p.get("head"))
            status = p.get("status")
            if isinstance(status, str):
                head = f"{head} ({status})" if head else f"({status})"
            body = _indent(_block([], cols, kids))
            return ([head] if head else []) + body
        case "rule":
            label = _text(p.get("label"))
            if not label:
                return ["-" * cols]
            side = max((cols - _width(label) - 2) // 2, 2)
            return [f"{'-' * side} {label} {'-' * side}"]
        case "spacer":
            return [""]
        case "text" | "shimmer" | "seg":
            seg = _node_text(p)
            if k == "seg" and kids:
                seg = " ".join([*(" ".join(_render(c, cols)) for c in kids), seg]).strip()
            return seg.split("\n")
        case "md" | "code" | "math" | "editor" | "input":
            return _text(p.get("text")).split("\n")
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
        case "badge":
            return [f"[{_text(p.get('text'))}]"]
        case "kbd":
            return ["+".join(s for s in p.get("keys") or [] if isinstance(s, str))]
        case "icon":
            return []
        case "image":
            return [f"[image: {_text(p.get('alt')) or _text(p.get('title')) or 'image'}]"]
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
        case "picker":
            title = _text(p.get("title"))
            items = [i for i in p.get("items") or [] if isinstance(i, Mapping)]
            return ([title] if title else []) + [f"- {_text(i.get('label')) or i.get('id')}" for i in items]
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
            return [summary] if summary else []
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
    for r in rows:
        cells = r.get("cells") if isinstance(r.get("cells"), Mapping) else {}
        line = []
        for c in columns:
            cell = cells.get(c["id"]) if isinstance(cells, Mapping) else None
            if isinstance(cell, Mapping):
                meter = cell.get("meter")
                value = meter.get("value") if isinstance(meter, Mapping) else None
                parts = meter.get("parts") if isinstance(meter, Mapping) else None
                if value is None and isinstance(parts, list):
                    value = sum(part.get("value", 0) for part in parts if isinstance(part, Mapping))
                line.append(_bar(value))
            else:
                line.append(_text(cell))
        out.append(line)
    return _table(out, [c.get("align") == "end" for c in columns])


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
