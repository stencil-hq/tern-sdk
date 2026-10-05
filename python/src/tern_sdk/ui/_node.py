"""The node type, the props every node takes, and handlers attached to nodes."""

from __future__ import annotations

from collections.abc import Callable, Iterable, Mapping
from dataclasses import dataclass, field
from typing import Any, Literal, TypeAlias, TypedDict

from ..wire import (
    ActionEvent,
    ActivateEvent,
    ChangeEvent,
    EditEvent,
    Event,
    FocusEvent,
    Json,
    SelectEvent,
    SendEvent,
    ToggleEvent,
    UndoEvent,
)
from ._types import Actions, Bounds, Mark, Text, Tone

Handler: TypeAlias = Callable[[Any], object]
"""A function run on an event routed to its node."""


@dataclass
class Node:
    """A node of a view: kind, props, children and handlers. It has no id
    until it is reconciled; ids derive from its place in the view."""

    kind: str
    props: dict[str, Json] = field(default_factory=dict)
    children: list[Node] = field(default_factory=list)
    handlers: dict[str, Handler] = field(default_factory=dict)
    """Handlers by route: `action:<act>`, or an event name (`toggle`, `change`, …)."""

    def to_json(self) -> dict[str, Json]:
        """The node as a wire node without ids: `{"k", "p"?, "c"?}`."""
        out: dict[str, Json] = {"k": self.kind}
        if self.props:
            out["p"] = self.props
        if self.children:
            out["c"] = [c.to_json() for c in self.children]
        return out


class Handlers(TypedDict, total=False):
    """Event handlers any builder takes; each receives the typed event (`None` sets none)."""

    on_click: Callable[[ActionEvent], object] | None
    on_dblclick: Callable[[ActionEvent], object] | None
    on_menu: Mapping[str, Callable[[ActionEvent], object]] | None
    on_action: Mapping[str, Callable[[ActionEvent], object]] | None
    on_toggle: Callable[[ToggleEvent], object] | None
    on_select: Callable[[SelectEvent], object] | None
    on_activate: Callable[[ActivateEvent], object] | None
    on_change: Callable[[ChangeEvent], object] | None
    on_focus: Callable[[FocusEvent], object] | None
    on_edit: Callable[[EditEvent], object] | None
    on_undo: Callable[[UndoEvent], object] | None
    on_send: Callable[[SendEvent], object] | None


class BaseProps(Handlers, total=False):
    """Props every node takes, minus the ones some kinds use for their own (`None` leaves a prop out)."""

    key: str | int | None
    role: str | None
    tone: Tone | None
    hidden: bool | None
    mark: Mark | None
    aria: str | None
    href: str | None
    grow: float | None
    shrink: float | None
    basis: float | Literal["content"] | None
    min: Bounds | None
    props: Mapping[str, Json] | None
    """Extra props set untyped, merged last (an escape hatch for newer props)."""


class Common(BaseProps, total=False):
    """Every common prop: `BaseProps` plus `title`, `max` and `actions`."""

    title: str | None
    max: Bounds | None
    actions: Actions | None


class CommonNoTitle(BaseProps, total=False):
    """Common props for kinds whose `title` is their own."""

    max: Bounds | None
    actions: Actions | None


class CommonNoMax(BaseProps, total=False):
    """Common props for kinds whose `max` is their own."""

    title: str | None
    actions: Actions | None


class PickerCommon(BaseProps, total=False):
    """Common props for `picker`, whose `title` and `actions` are its own."""

    max: Bounds | None


_EVENT_HANDLERS = (
    ("on_toggle", "toggle"),
    ("on_select", "select"),
    ("on_activate", "activate"),
    ("on_change", "change"),
    ("on_focus", "focus"),
    ("on_edit", "edit"),
    ("on_undo", "undo"),
    ("on_send", "send"),
)
_EVENT_ROUTES = frozenset(r for _, r in _EVENT_HANDLERS)
_COMMON = (
    "key", "role", "tone", "hidden", "mark", "title", "aria", "href", "grow", "shrink",
    "basis", "min", "max", "actions",
)  # fmt: skip


def text_props(text: Text | None) -> dict[str, Json]:
    """`{"text": s}` for a string, `{"spans": [...]}` for spans, `{}` for None."""
    if text is None:
        return {}
    if isinstance(text, str):
        return {"text": text}
    return {"spans": text}


def jsonify(value: Any) -> Json:
    """A prop value as plain JSON data: mappings to dicts, sequences to lists."""
    if isinstance(value, (str, int, float, bool)) or value is None:
        return value
    if isinstance(value, Mapping):
        return {str(k): jsonify(v) for k, v in value.items()}
    if isinstance(value, (list, tuple)):
        return [jsonify(v) for v in value]
    raise TypeError(f"not a JSON value: {value!r}")


def make(
    kind: str,
    own: Mapping[str, Json],
    children: Iterable[Node | None] = (),
    common: Mapping[str, Any] | None = None,
) -> Node:
    """A node of `kind` from its own props (`None` values dropped), its
    children (`None` skipped) and the common props and handlers."""
    props: dict[str, Json] = {k: jsonify(v) for k, v in own.items() if v is not None}
    handlers: dict[str, Handler] = {}
    common = common or {}
    for name in _COMMON:
        value = common.get(name)
        if value is not None:
            props[name] = jsonify(value)
    actions: dict[str, Json] | None = None

    def ensure_actions() -> dict[str, Json]:
        nonlocal actions
        if actions is None:
            existing = props.get("actions")
            actions = dict(existing) if isinstance(existing, dict) else {}
            props["actions"] = actions
        return actions

    for gesture in ("click", "dblclick"):
        fn = common.get(f"on_{gesture}")
        if fn is not None:
            acts = ensure_actions()
            name = acts.setdefault(gesture, gesture)
            handlers[f"action:{name}"] = fn
    menu = common.get("on_menu")
    if menu:
        acts = ensure_actions()
        names = list(acts.get("menu") or [])
        for name, fn in menu.items():
            if name not in names:
                names.append(name)
            handlers[f"action:{name}"] = fn
        acts["menu"] = names
    for name, fn in (common.get("on_action") or {}).items():
        handlers[f"action:{name}"] = fn
    for py, route in _EVENT_HANDLERS:
        fn = common.get(py)
        if fn is not None:
            handlers[route] = fn
    extra = common.get("props")
    if extra:
        props.update(jsonify(extra))
    return Node(kind, props, [c for c in children if c is not None], handlers)


def route(handlers: Mapping[str, Handler], event: Event) -> Handler | None:
    """The handler among a node's `handlers` that `event` runs, if any."""
    if isinstance(event, ActionEvent):
        if event.act is None:
            return None
        if event.value is not None:
            fn = handlers.get(f"action:{event.act}={event.value}")
            if fn is not None:
                return fn
        return handlers.get(f"action:{event.act}")
    if event.ev is not None and event.ev in _EVENT_ROUTES:
        return handlers.get(event.ev)
    return None
