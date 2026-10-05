"""Typed builders for every kind except `block` (Tern's own), `list` and
`input` (in `_shadow`, since their names shadow builtins)."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any, Literal, Unpack

from ..wire import Json
from ._node import Common, CommonNoTitle, Handlers, Node, PickerCommon, make, text_props
from ._types import (
    AgentRetry,
    AgentStats,
    AgentStatus,
    AgentTool,
    Align,
    Badge,
    CaretAnchor,
    ChartCol,
    ChartKind,
    ChartPoint,
    ChecklistMode,
    CodeMark,
    Decor,
    DiffMode,
    EffortLevel,
    ElapsedFormat,
    Fx,
    Gap,
    Hunk,
    IconName,
    Justify,
    KvItem,
    KvLayout,
    LinesPreview,
    MeterMark,
    MeterPart,
    MeterStyle,
    NodeAnchor,
    OverlayPlace,
    OverlaySize,
    Phase,
    PickerAction,
    PickerColumn,
    PickerConfirm,
    PickerFocus,
    PickerGroup,
    PickerItem,
    PickerLayout,
    PickerPreview,
    PickerScope,
    PickerSize,
    PickerState,
    PickerStrip,
    PickerTab,
    PrefsEditing,
    PrefsPage,
    PrefsSection,
    ShimmerMode,
    ShimmerPalette,
    Size3,
    Span,
    Spans,
    SpinnerStyle,
    Status,
    Tab,
    TableCol,
    TableRow,
    TailPreview,
    TargetKind,
    Text,
    Thresholds,
    ToolButton,
    ToolFrame,
    TreeNode,
    Truncate,
    Wrap,
)


def span(t: str, s: str | None = None, *, fx: Fx | None = None, href: str | None = None) -> Span:
    """A styled span: text `t` with space-separated style tokens `s`."""
    out: Span = {"t": t}
    if s is not None:
        out["s"] = s
    if fx is not None:
        out["fx"] = fx
    if href is not None:
        out["href"] = href
    return out


def node(
    kind: str,
    props: Mapping[str, Any] | None = None,
    children: Sequence[Node | None] = (),
    **handlers: Unpack[Handlers],
) -> Node:
    """Any kind with untyped props: the escape hatch for kinds and props
    newer than the SDK. Handlers attach as on every builder."""
    return make(kind, props or {}, children, handlers)


# Layout


def col(
    *children: Node | None,
    gap: Gap | None = None,
    align: Align | None = None,
    justify: Justify | None = None,
    wrap: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """A vertical stack of its children."""
    return make("col", {"gap": gap, "align": align, "justify": justify, "wrap": wrap}, children, common)


def row(
    *children: Node | None,
    gap: Gap | None = None,
    align: Align | None = None,
    justify: Justify | None = None,
    wrap: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """A horizontal flex row of its children."""
    return make("row", {"gap": gap, "align": align, "justify": justify, "wrap": wrap}, children, common)


def card(
    *children: Node | None,
    head: Text | None = None,
    status: Status | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    preview: Literal["auto"] | LinesPreview | None = None,
    selected: bool | None = None,
    inset: bool | None = None,
    variant: Literal["bare"] | None = None,
    **common: Unpack[Common],
) -> Node:
    """A framed box: a head row (icon, head, status chip, chevron) over a body."""
    own = {
        "head": head, "status": status, "collapsible": collapsible, "collapsed": collapsed,
        "preview": preview, "selected": selected, "inset": inset, "variant": variant,
    }  # fmt: skip
    return make("card", own, children, common)


def section(
    *children: Node | None,
    head: Text | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """An unframed disclosure group: chevron and head over an indented body."""
    return make("section", {"head": head, "collapsible": collapsible, "collapsed": collapsed}, children, common)


def rule(label: Text | None = None, **common: Unpack[Common]) -> Node:
    """A hairline divider, optionally labeled."""
    return make("rule", {"label": label}, (), common)


def spacer(size: Gap | None = None, **common: Unpack[Common]) -> Node:
    """A fixed block of vertical space."""
    return make("spacer", {"size": size}, (), common)


# Text and code


def text(
    text: Text,
    *,
    wrap: Wrap | None = None,
    truncate: Truncate | None = None,
    lines: int | None = None,
    measure: Literal["prose"] | None = None,
    **common: Unpack[Common],
) -> Node:
    """Styled text that wraps, clamps or truncates (a string or spans)."""
    own = {**text_props(text), "wrap": wrap, "truncate": truncate, "lines": lines, "measure": measure}
    return make("text", own, (), common)


def md(text: str, *, stream: bool | None = None, marks: Spans | None = None, **common: Unpack[Common]) -> Node:
    """Markdown; `stream` while the source is still arriving."""
    return make("md", {"text": text, "stream": stream, "marks": marks}, (), common)


def code(
    text: str,
    lang: str | None = None,
    *,
    path: str | None = None,
    numbers: bool | None = None,
    start: int | None = None,
    marks: Sequence[CodeMark] | None = None,
    wrap: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """A highlighted code block."""
    own = {"text": text, "lang": lang, "path": path, "numbers": numbers, "start": start, "marks": marks, "wrap": wrap}
    return make("code", own, (), common)


def diff(
    text: str | None = None,
    *,
    hunks: Sequence[Hunk] | None = None,
    path: str | None = None,
    lang: str | None = None,
    mode: DiffMode | None = None,
    **common: Unpack[Common],
) -> Node:
    """A unified or side-by-side diff, from unified diff text or hunks."""
    return make("diff", {"text": text, "hunks": hunks, "path": path, "lang": lang, "mode": mode}, (), common)


def ansi(
    text: str,
    *,
    cols: int | None = None,
    preview: LinesPreview | None = None,
    follow: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """Program output with escape sequences, in a mini terminal."""
    return make("ansi", {"text": text, "cols": cols, "preview": preview, "follow": follow}, (), common)


def rows(lines: Sequence[str], *, cols: int | None = None, **common: Unpack[Common]) -> Node:
    """Pre-rendered ANSI rows at a fixed width (the migration fallback)."""
    return make("rows", {"lines": lines, "cols": cols}, (), common)


def math(text: str, *, display: bool | None = None, **common: Unpack[Common]) -> Node:
    """One typeset TeX formula."""
    return make("math", {"text": text, "display": display}, (), common)


# Data


def kv(
    items: Sequence[KvItem | tuple[Text, Text]], *, layout: KvLayout | None = None, **common: Unpack[Common]
) -> Node:
    """An aligned key/value list; pairs as `{"k", "v"}` or `(k, v)`."""
    pairs = [{"k": i[0], "v": i[1]} if isinstance(i, tuple) else i for i in items]
    return make("kv", {"items": pairs, "layout": layout}, (), common)


def table(cols: Sequence[TableCol], rows: Sequence[TableRow], **common: Unpack[Common]) -> Node:
    """A table with priority-hidden columns and text or meter cells."""
    return make("table", {"cols": cols, "rows": rows}, (), common)


def tree(nodes: Sequence[TreeNode], **common: Unpack[Common]) -> Node:
    """A disclosure tree."""
    return make("tree", {"nodes": nodes}, (), common)


def badge(text: str, **common: Unpack[Common]) -> Node:
    """A pill chip (its color is the `tone` prop)."""
    return make("badge", {"text": text}, (), common)


def kbd(*keys: str, **common: Unpack[Common]) -> Node:
    """Keycaps for key names (`"cmd"`, `"k"`)."""
    return make("kbd", {"keys": keys}, (), common)


def icon(name: IconName, **common: Unpack[Common]) -> Node:
    """A named icon from Tern's set."""
    return make("icon", {"name": name}, (), common)


def image(
    blob: str | None = None,
    *,
    builtin: Literal["omp"] | None = None,
    alt: str | None = None,
    w: float | None = None,
    h: float | None = None,
    path: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """An image from a blob id (see `Session.blob`)."""
    return make("image", {"blob": blob, "builtin": builtin, "alt": alt, "w": w, "h": h, "path": path}, (), common)


# Lists, tabs and pickers


def item(
    label: Text | None = None,
    *,
    detail: Text | None = None,
    value: Text | None = None,
    icon: IconName | None = None,
    hint: Sequence[str] | None = None,
    disabled: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """One `list` row: icon, label, detail, value, key hint."""
    own = {"label": label, "detail": detail, "value": value, "icon": icon, "hint": hint, "disabled": disabled}
    return make("item", own, (), common)


def tabs(items: Sequence[Tab], *, active: str | None = None, **common: Unpack[Common]) -> Node:
    """A tab strip; `active` is the active tab's id."""
    return make("tabs", {"items": items, "active": active}, (), common)


def picker(
    *children: Node | None,
    size: PickerSize | None = None,
    layout: PickerLayout | None = None,
    preview: PickerPreview | None = None,
    title: Text | None = None,
    subtitle: Text | None = None,
    icon: IconName | None = None,
    query: str | None = None,
    cursor: int | None = None,
    placeholder: str | None = None,
    noun: str | None = None,
    focus: PickerFocus | None = None,
    state: PickerState | None = None,
    message: Text | None = None,
    empty: Text | None = None,
    total: int | None = None,
    items: Sequence[PickerItem] | None = None,
    items_add: Sequence[PickerItem] | None = None,
    items_del: Sequence[str] | None = None,
    order: Sequence[str | PickerGroup] | None = None,
    selected: str | None = None,
    current: Sequence[str] | None = None,
    hits: Mapping[str, Sequence[Sequence[int]]] | None = None,
    columns: Sequence[PickerColumn] | None = None,
    confirm: PickerConfirm | None = None,
    scopes: Sequence[PickerScope] | None = None,
    scope: str | None = None,
    tabs: Sequence[PickerTab] | None = None,
    tab: str | None = None,
    strip: PickerStrip | None = None,
    actions: Sequence[PickerAction] | None = None,
    **common: Unpack[PickerCommon],
) -> Node:
    """A searchable picker sheet; children are its preview pane's content."""
    own: dict[str, Json] = {
        "size": size, "layout": layout, "preview": preview, "title": title, "subtitle": subtitle,
        "icon": icon, "query": query, "cursor": cursor, "placeholder": placeholder, "noun": noun,
        "focus": focus, "state": state, "message": message, "empty": empty, "total": total,
        "items": items, "itemsAdd": items_add, "itemsDel": items_del, "order": order,
        "selected": selected, "current": current, "hits": hits, "columns": columns,
        "confirm": confirm, "scopes": scopes, "scope": scope, "tabs": tabs, "tab": tab,
        "strip": strip, "actions": actions,
    }  # fmt: skip
    return make("picker", own, children, common)


# Progress and motion


def spinner(label: Text | None = None, *, style: SpinnerStyle | None = None, **common: Unpack[Common]) -> Node:
    """An activity indicator, optionally labeled."""
    return make("spinner", {"style": style, "label": label}, (), common)


def shimmer(
    text: Text,
    *,
    mode: ShimmerMode | None = None,
    palette: ShimmerPalette | None = None,
    **common: Unpack[Common],
) -> Node:
    """Text with a light sweeping across it."""
    return make("shimmer", {**text_props(text), "mode": mode, "palette": palette}, (), common)


def elapsed(
    age: float = 0,
    *,
    stopped: float | None = None,
    format: ElapsedFormat | None = None,
    **common: Unpack[Common],
) -> Node:
    """A live timer: `age` ms already elapsed (negative counts down)."""
    return make("elapsed", {"age": age, "stopped": stopped, "format": format}, (), common)


def rate(value: float, *, unit: str | None = None, **common: Unpack[Common]) -> Node:
    """A number that eases to each new value."""
    return make("rate", {"value": value, "unit": unit}, (), common)


def progress(value: float | None = None, label: Text | None = None, **common: Unpack[Common]) -> Node:
    """A progress bar: `value` 0-1, or `None` for indeterminate."""
    return make("progress", {"value": value, "label": label}, (), common)


def meter(
    value: float | None = None,
    label: Text | None = None,
    *,
    parts: Sequence[MeterPart] | None = None,
    thresholds: Thresholds | None = None,
    style: MeterStyle | None = None,
    size: Size3 | None = None,
    steps: int | None = None,
    marks: Sequence[MeterMark] | None = None,
    total: Text | None = None,
    **common: Unpack[Common],
) -> Node:
    """A value or stacked parts as a bar, ring or block grid."""
    own = {
        "value": value, "parts": parts, "thresholds": thresholds, "style": style, "size": size,
        "steps": steps, "marks": marks, "label": label, "total": total,
    }  # fmt: skip
    return make("meter", own, (), common)


def chart(
    kind: ChartKind | None = None,
    *,
    size: Size3 | None = None,
    token: str | None = None,
    summary: Text | None = None,
    series: Sequence[ChartPoint] | None = None,
    cells: Sequence[Sequence[float | None]] | None = None,
    tips: Sequence[Sequence[str | None]] | None = None,
    rows: Sequence[str] | None = None,
    cols: Sequence[ChartCol] | None = None,
    **common: Unpack[Common],
) -> Node:
    """A heatmap, a bar chart or a sparkline."""
    own = {
        "kind": kind, "size": size, "token": token, "summary": summary, "series": series,
        "cells": cells, "tips": tips, "rows": rows, "cols": cols,
    }  # fmt: skip
    return make("chart", own, (), common)


def effort(level: EffortLevel, **common: Unpack[Common]) -> Node:
    """A ring that fills by thinking-effort level."""
    return make("effort", {"level": level}, (), common)


# Inputs


def editor(
    text: str = "",
    *,
    cursor: int | None = None,
    anchor: int | None = None,
    decor: Sequence[Decor] | None = None,
    ghost: str | None = None,
    placeholder: str | None = None,
    prompt: Text | None = None,
    mode: str | None = None,
    lang: str | None = None,
    readonly: bool | None = None,
    sendable: bool | None = None,
    max_lines: int | None = None,
    **common: Unpack[Common],
) -> Node:
    """A multi-line text field; the program owns its text and caret."""
    own = {
        "text": text, "cursor": cursor, "anchor": anchor, "decor": decor, "ghost": ghost,
        "placeholder": placeholder, "prompt": prompt, "mode": mode, "lang": lang,
        "readonly": readonly, "sendable": sendable, "maxLines": max_lines,
    }  # fmt: skip
    return make("editor", own, (), common)


# Bars, toasts and overlays


def status(*segs: Node | None, transparent: bool | None = None, **common: Unpack[Common]) -> Node:
    """A status strip of `seg`s (`side="right"` ones on the right)."""
    return make("status", {"transparent": transparent}, segs, common)


def seg(
    text: Text = "",
    *children: Node | None,
    icon: IconName | None = None,
    side: Literal["right"] | None = None,
    priority: float | None = None,
    **common: Unpack[Common],
) -> Node:
    """One status segment: icon, text (a string or spans), optional children."""
    own = {**text_props(text), "icon": icon, "side": side, "priority": priority}
    return make("seg", own, children, common)


def toast(text: str, *, sub: str | None = None, ttl: float | None = None, **common: Unpack[Common]) -> Node:
    """A transient notice (`tone="error"` for an error toast)."""
    return make("toast", {"text": text, "sub": sub, "ttl": ttl}, (), common)


def overlay(
    *children: Node | None,
    anchor: OverlayPlace | NodeAnchor | CaretAnchor | None = None,
    size: OverlaySize | None = None,
    modal: bool | None = None,
    head: Text | None = None,
    **common: Unpack[Common],
) -> Node:
    """A floating panel or sheet, for the `layer` region."""
    return make("overlay", {"anchor": anchor, "size": size, "modal": modal, "head": head}, children, common)


# Tool, agent and task kinds


def tool(
    *children: Node | None,
    name: str | None = None,
    title: Text | None = None,
    target: Text | None = None,
    target_kind: TargetKind | None = None,
    meta: Sequence[Text] | None = None,
    badges: Sequence[Badge] | None = None,
    note: Text | None = None,
    exit: int | None = None,
    status: Status | None = None,
    age: float | None = None,
    took: float | None = None,
    intent: str | None = None,
    frame: ToolFrame | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    preview: LinesPreview | TailPreview | None = None,
    tools: Sequence[ToolButton] | None = None,
    **common: Unpack[CommonNoTitle],
) -> Node:
    """One step of work: a data head (verb, target, status, timer) over a body."""
    own = {
        "name": name, "title": title, "target": target, "targetKind": target_kind, "meta": meta,
        "badges": badges, "note": note, "exit": exit, "status": status, "age": age, "took": took,
        "intent": intent, "frame": frame, "collapsible": collapsible, "collapsed": collapsed,
        "preview": preview, "tools": tools,
    }  # fmt: skip
    return make("tool", own, children, common)


def agent(
    *children: Node | None,
    name: str | None = None,
    agent: str | None = None,
    badges: Sequence[Badge] | None = None,
    task: Text | None = None,
    status: AgentStatus | None = None,
    model: str | None = None,
    thinking: str | None = None,
    stats: AgentStats | None = None,
    tool: AgentTool | None = None,
    retry: AgentRetry | None = None,
    depth: int | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """One worker row with live status and stats; children are its content."""
    own = {
        "name": name, "agent": agent, "badges": badges, "task": task, "status": status,
        "model": model, "thinking": thinking, "stats": stats, "tool": tool, "retry": retry,
        "depth": depth, "collapsible": collapsible, "collapsed": collapsed,
    }  # fmt: skip
    return make("agent", own, children, common)


def checklist(
    phases: Sequence[Phase],
    *,
    mode: ChecklistMode | None = None,
    note: Text | None = None,
    **common: Unpack[Common],
) -> Node:
    """A phased todo list, a dock pill or a reminder."""
    return make("checklist", {"phases": phases, "mode": mode, "note": note}, (), common)


def prefs(
    *children: Node | None,
    title: str | None = None,
    pages: Sequence[PrefsPage] | None = None,
    page: str | None = None,
    lead: str | None = None,
    sections: Sequence[PrefsSection] | None = None,
    query: str | None = None,
    cursor: int | None = None,
    focus: str | None = None,
    editing: PrefsEditing | None = None,
    **common: Unpack[CommonNoTitle],
) -> Node:
    """A settings page or sheet driven by the program's data and keys."""
    own = {
        "title": title, "pages": pages, "page": page, "lead": lead, "sections": sections,
        "query": query, "cursor": cursor, "focus": focus, "editing": editing,
    }  # fmt: skip
    return make("prefs", own, children, common)
