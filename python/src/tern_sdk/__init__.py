"""Talk to Tern over the Tern Surface Protocol (TSP).

Six layers, each usable without the ones above it: `wire` (messages,
framing, chunking), `input` and `keys` (the input parser and key decoder),
`ui` (typed node builders and handlers), `reconcile` (views diffed into
frame ops), `session` (handshake, surfaces, flow control, events) and
`helpers` (`show`, `ask`, `plain`)."""

from . import input, keys, reconcile, session, ui, wire
from .helpers import Answer, Unsupported, ask, plain, show
from .input import DA1, Da1, InputParser
from .keys import Key, KeyDecoder
from .reconcile import AnyView, ReconcileError, View
from .session import Capabilities, InputItem, Session, Surface, available, connect, start
from .ui import Node, span
from .wire import (
    AckEvent,
    ActionEvent,
    ActivateEvent,
    BlobsReply,
    Cell,
    ChangeEvent,
    EditEvent,
    ErrorEvent,
    Event,
    FocusEvent,
    GoneEvent,
    HelloReply,
    MotionEvent,
    Reply,
    ResizeEvent,
    SelectEvent,
    SendEvent,
    ThemeEvent,
    ToggleEvent,
    UndoEvent,
    UnknownEvent,
    UnknownReply,
    VisibleEvent,
)

__all__ = [
    "DA1", "AckEvent", "ActionEvent", "ActivateEvent", "Answer", "AnyView", "BlobsReply",
    "Capabilities", "Cell", "ChangeEvent", "Da1", "EditEvent", "ErrorEvent", "Event",
    "FocusEvent", "GoneEvent", "HelloReply", "InputItem", "InputParser", "Key", "KeyDecoder",
    "MotionEvent", "Node", "ReconcileError", "Reply", "ResizeEvent", "SelectEvent", "SendEvent",
    "Session", "Surface", "ThemeEvent", "ToggleEvent", "UndoEvent", "UnknownEvent",
    "UnknownReply", "Unsupported", "View", "VisibleEvent",
    "ask", "available", "connect", "input", "keys", "plain", "reconcile", "session",
    "show", "span", "start", "ui", "wire",
]  # fmt: skip
