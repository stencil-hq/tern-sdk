"""Typed shapes of node props: enumerations, spans and structured values.

TypedDict keys are the wire's own names (camelCase where the wire uses it),
since these values travel as they are."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Literal, Required, TypeAlias, TypedDict

Tone: TypeAlias = Literal["neutral", "accent", "info", "success", "warning", "error", "pending", "muted", "user"]
"""The semantic color of a node's chrome."""
Mark: TypeAlias = Literal["pick", "drop"]
"""A transient selection drawn over a node."""
Status: TypeAlias = Literal["pending", "running", "done", "error", "cancelled"]
"""The state of a `card` or `tool`."""
AgentStatus: TypeAlias = Literal["pending", "running", "done", "failed", "aborted", "idle", "parked"]
"""The state of an `agent`."""
Gap: TypeAlias = Literal["none", "xs", "sm", "md", "lg"]
"""Space between a `col`'s or `row`'s children (also a `spacer`'s height)."""
Align: TypeAlias = Literal["start", "center", "end", "baseline", "stretch"]
"""Cross-axis alignment of a `col` or `row`."""
Justify: TypeAlias = Literal["between", "end"]
"""Main-axis distribution of a `col` or `row`."""
Fx: TypeAlias = Literal["shimmer", "pulse", "none"]
"""A span effect."""
Wrap: TypeAlias = Literal["word", "char", "none"]
"""How a `text` wraps."""
Truncate: TypeAlias = Literal["end", "start", "middle"]
"""Where overflowing text is cut."""
DiffMode: TypeAlias = Literal["unified", "split", "auto"]
"""How a `diff` lays out."""
KvLayout: TypeAlias = Literal["grid", "inline"]
"""How a `kv` lays out."""
TextAlign: TypeAlias = Literal["start", "center", "end"]
"""A table column's alignment."""
SpinnerStyle: TypeAlias = Literal["braille", "dots", "starburst", "orbit"]
"""A spinner's indicator."""
ShimmerMode: TypeAlias = Literal["classic", "kitt"]
"""A shimmer's sweep."""
ElapsedFormat: TypeAlias = Literal["short", "clock"]
"""How an `elapsed` timer reads."""
MeterStyle: TypeAlias = Literal["bar", "ring", "blocks"]
"""A meter's shape."""
Size3: TypeAlias = Literal["sm", "md", "lg"]
"""Small, medium or large."""
ChartKind: TypeAlias = Literal["heatmap", "bars", "spark"]
"""Which chart a `chart` draws."""
EffortLevel: TypeAlias = Literal["off", "minimal", "low", "medium", "high", "xhigh", "max"]
"""A thinking-effort rung."""
OverlaySize: TypeAlias = Literal["sm", "md", "lg", "full"]
"""An overlay's width, or a full sheet."""
OverlayPlace: TypeAlias = Literal["center", "top", "bottom"]
"""An overlay placed in the layer."""
TargetKind: TypeAlias = Literal["command", "path", "pattern", "query", "text"]
"""How a tool's string `target` is drawn."""
ToolFrame: TypeAlias = Literal["card", "inline"]
"""Whether a tool has a ring."""
ChecklistMode: TypeAlias = Literal["full", "hud", "reminder"]
"""A checklist's presentation."""
ItemStatus: TypeAlias = Literal["pending", "active", "done", "dropped", "blocked"]
"""A checklist item's state."""
PickerSize: TypeAlias = Literal["md", "lg", "screen"]
"""A picker sheet's size."""
PickerLayout: TypeAlias = Literal["rows", "cards", "timeline", "tree"]
"""A picker's row template."""
PickerPreview: TypeAlias = Literal["side", "below", "none"]
"""Where a picker's preview sits."""
PickerFocus: TypeAlias = Literal["list", "scopes", "tabs", "strip", "preview"]
"""Which part of a picker the keys drive."""
PickerState: TypeAlias = Literal["ready", "loading", "error"]
"""A picker's loading state."""
ColumnFormat: TypeAlias = Literal["text", "num", "price", "bar", "time", "elapsed", "dim"]
"""A picker fact column's format."""
InputType: TypeAlias = Literal["checkbox", "radio"]
"""An `el` `input`'s control type."""
Tag: TypeAlias = Literal[
    "div", "span", "p", "section", "header", "footer", "nav", "aside", "main", "article",
    "figure", "blockquote", "ul", "ol", "li", "dl", "dt", "dd", "h1", "h2", "h3", "h4", "pre",
    "code", "kbd", "strong", "b", "em", "i", "del", "mark", "hr", "table", "thead", "tbody",
    "tr", "th", "td", "label", "button", "form", "input",
]  # fmt: skip
"""An allowed `el` tag."""
IconName: TypeAlias = Literal[
    "logo", "tern", "pi-mark",
    "arrow-up", "arrow-down", "arrow-left", "arrow-right", "back", "forward", "chev", "chev-r",
    "chev-up", "corner-down-right", "expand", "shrink", "minimize", "fit", "zoom-in",
    "zoom-out", "more", "grip", "compass",
    "check", "x", "x-circle", "warn", "info", "help", "bell", "shield", "shield-alert",
    "slash", "slash-circle", "lock", "key", "key-round", "plus", "minus", "clock", "timer",
    "doc", "file", "file-code", "file-image", "file-pdf", "file-plus", "files", "folder",
    "folder-go", "folder-minus", "folder-open", "folder-plus", "markdown", "note",
    "clipboard", "save", "inbox", "image", "newspaper", "trash",
    "branch", "commit", "merge", "rebase", "pull", "push", "fetch", "stash", "cherry", "diff",
    "unified", "split", "split-down", "revert", "history",
    "play", "pause", "stop", "rewind", "fast-forward", "frame-next", "frame-prev", "repeat",
    "redo", "undo", "run", "reel", "mic", "wave", "vibrate", "power", "send", "share", "open",
    "download", "copy", "scissors",
    "bold", "italic", "underline", "type", "align-left", "align-center", "align-right",
    "align-justify", "pen", "eraser", "wand", "cursor", "selection", "path-insert", "prompt",
    "vector", "blur", "ink", "palette",
    "columns", "grid", "sidebar", "tabs-h", "tabs-v", "dock-up", "dock-down", "dock-left",
    "dock-right", "pip", "pip-tl", "pip-tr", "pip-bl", "pip-exit", "layers", "stack",
    "kanban", "canvas", "diagram", "flow", "peek", "tree", "box", "eye", "eye-off",
    "terminal", "code", "braces", "binary", "bug", "cpu", "database", "server", "laptop",
    "monitor", "gauge", "activity", "chart", "plug", "puzzle", "wrench", "gear", "sliders",
    "stethoscope", "hash", "tag", "link", "globe", "broadcast", "search", "funnel",
    "funnel-x", "sort-x", "swap", "list", "list-checks", "pin", "keyboard",
    "user", "users", "message", "brain", "sparkle", "lightbulb", "bolt", "flame", "rocket",
    "moon", "sun", "cart", "footprints", "scale", "log-in", "log-out",
]  # fmt: skip
"""A name from Tern's icon set. The app icons (`app-vim`, `app-python`, …) follow Tern's
catalog of programs and aren't listed: pass the name untyped."""


class Span(TypedDict, total=False):
    """A styled run of text: `t` the text, `s` space-separated style tokens."""

    t: Required[str]
    s: str
    fx: Fx
    href: str


Spans: TypeAlias = Sequence[Span | str]
"""A list of spans (plain strings are unstyled spans)."""
Text: TypeAlias = str | Spans
"""Text given as a plain string or as styled spans."""
Extent: TypeAlias = str | float
"""A size: `"<n>ch"`, `"<n>lines"` or a fraction of the parent."""


class Bounds(TypedDict, total=False):
    """`min` / `max` size bounds."""

    w: Extent
    h: Extent


class Actions(TypedDict, total=False):
    """What a pointer does on a node: an action name per gesture."""

    click: str
    dblclick: str
    menu: Sequence[str]


class LinesPreview(TypedDict):
    """A preview clamp of the first `lines` lines."""

    lines: int


class TailPreview(TypedDict):
    """A preview clamp of the last `tail` lines."""

    tail: int


class KvItem(TypedDict):
    """One key/value pair of a `kv`."""

    k: Text
    v: Text


class Thresholds(TypedDict, total=False):
    """Levels at which a meter turns `warn` or `bad`."""

    warn: float
    bad: float


class MeterPart(TypedDict, total=False):
    """One stacked fill of a meter."""

    value: Required[float]
    token: str
    label: str
    hatch: bool


class MeterMark(TypedDict, total=False):
    """A tick on a meter's track."""

    at: Required[float]
    tone: Tone
    title: str
    icon: IconName


class MeterCellProps(TypedDict, total=False):
    """The meter inside a table cell."""

    value: float
    parts: Sequence[MeterPart]
    thresholds: Thresholds
    tone: Tone
    title: str


class MeterCell(TypedDict):
    """A table cell drawn as a meter, with its required wire wrapper."""

    meter: MeterCellProps


class TableCol(TypedDict, total=False):
    """A table column."""

    id: Required[str]
    head: Text
    align: TextAlign
    truncate: Truncate
    priority: float
    grow: float


class TableRow(TypedDict):
    """A table row: one cell per column id."""

    id: str
    cells: Mapping[str, Text | MeterCell]


class TreeNode(TypedDict, total=False):
    """One item of a `tree`."""

    id: Required[str]
    label: Text
    icon: IconName
    open: bool
    children: Sequence[TreeNode]


CodeMark = TypedDict("CodeMark", {"line": Required[int], "tone": Tone, "ranges": Sequence[Sequence[int]]}, total=False)
"""A marked line of a `code` block, with byte ranges drawn stronger."""


class Hunk(TypedDict, total=False):
    """A diff hunk given directly."""

    oldStart: int
    newStart: int
    lines: Required[Sequence[str]]


class Tab(TypedDict):
    """One tab of a `tabs` strip."""

    id: str
    label: Text


class Badge(TypedDict, total=False):
    """A badge chip in a tool's, agent's or picker row's head."""

    text: Required[str]
    tone: Tone
    title: str


class ToolButton(TypedDict, total=False):
    """A hover action button of a `tool`."""

    id: Required[str]
    label: str
    keys: Sequence[str]


class AgentStats(TypedDict, total=False):
    """An agent's stats."""

    tools: int
    requests: int
    done: float
    context: float
    contextLabel: str
    tokens: int
    cost: float
    age: float
    took: float


class AgentTool(TypedDict, total=False):
    """The tool an agent runs now."""

    name: Required[str]
    intent: str
    age: float


class AgentRetry(TypedDict, total=False):
    """An agent's retry countdown."""

    attempt: int
    max: int
    delay: float
    age: float
    error: str


class ChecklistItem(TypedDict, total=False):
    """One checklist item."""

    id: Required[str]
    text: Text
    status: ItemStatus
    note: Text


class Phase(TypedDict, total=False):
    """One checklist phase."""

    id: str
    title: Text
    items: Sequence[ChecklistItem]
    collapsed: bool


class ChartPoint(TypedDict, total=False):
    """One bar of a `bars` or `spark` chart."""

    value: Required[float]
    label: str
    title: str


class ChartCol(TypedDict):
    """A heatmap column label."""

    at: int
    label: str


Decor = TypedDict("Decor", {"from": Required[int], "to": Required[int], "s": Required[str], "fx": Fx}, total=False)
"""A styled UTF-16 range of an editor's or input's text."""


class ShimmerPalette(TypedDict, total=False):
    """The three sweep colors of a shimmer, as span tokens."""

    low: str
    mid: str
    high: str


class NodeAnchor(TypedDict, total=False):
    """An overlay placed beside node `node`."""

    node: Required[str]
    side: Literal["below", "above"]


class CaretAnchor(TypedDict):
    """An overlay placed below the caret of field `caret`."""

    caret: str


class PickerMark(TypedDict, total=False):
    """A picker row's initials avatar."""

    text: Required[str]
    seed: str


class PickerChip(TypedDict, total=False):
    """A role chip of a picker row."""

    text: Required[str]
    on: bool
    auto: bool
    dot: str


class PickerItem(TypedDict, total=False):
    """One picker catalog item."""

    id: Required[str]
    label: Text
    detail: Text
    icon: IconName
    role: str
    mark: PickerMark
    dot: Tone
    mono: bool
    facts: Mapping[str, float | Text]
    badges: Sequence[Badge]
    chips: Sequence[PickerChip]
    tone: Tone
    disabled: bool | str
    hits: Sequence[Sequence[int]]
    node: str
    depth: int
    open: bool
    title: str


class PickerGroup(TypedDict, total=False):
    """A group header in a picker's `order`."""

    group: Required[str]
    label: Required[str]
    count: int


class PickerColumn(TypedDict, total=False):
    """A picker fact column."""

    id: Required[str]
    head: str
    format: ColumnFormat
    priority: float
    min: int


class PickerConfirm(TypedDict, total=False):
    """A picker's confirm strip."""

    text: Required[str]
    act: str
    label: str


class PickerScope(TypedDict, total=False):
    """A picker scope."""

    id: Required[str]
    label: Required[str]
    group: str
    mark: PickerMark
    icon: IconName
    dot: Tone
    count: int
    disabled: bool | str


class PickerTab(TypedDict, total=False):
    """A picker tab."""

    id: Required[str]
    label: Required[str]
    count: int


class PickerStripItem(TypedDict, total=False):
    """A picker strip chip."""

    id: Required[str]
    label: Required[str]
    on: bool
    dot: str


class PickerStrip(TypedDict, total=False):
    """A picker's row of switch chips."""

    label: str
    items: Required[Sequence[PickerStripItem]]
    selected: str


class PickerAction(TypedDict, total=False):
    """A picker action bar button."""

    id: Required[str]
    label: str
    keys: Sequence[str]
    primary: bool
    end: bool
    danger: bool
    on: bool
    disabled: bool | str


class PrefsPage(TypedDict, total=False):
    """A prefs nav row."""

    id: Required[str]
    label: str
    group: str
    icon: IconName
    disabled: str
    changed: int


class ChoiceOption(TypedDict, total=False):
    """An option of a prefs `choice` or `multi` control."""

    value: Required[str]
    label: str
    detail: str


class SwitchControl(TypedDict):
    """A prefs switch."""

    k: Literal["switch"]
    on: bool


class ChoiceControl(TypedDict, total=False):
    """A prefs choice."""

    k: Required[Literal["choice"]]
    value: str
    options: Required[Sequence[ChoiceOption]]
    style: Literal["auto", "segmented", "menu"]
    mono: bool


class NumberControl(TypedDict, total=False):
    """A prefs stepper."""

    k: Required[Literal["number"]]
    value: Required[float]
    min: float
    max: float
    step: float
    unit: str
    labels: Mapping[str, str]


class TextControl(TypedDict, total=False):
    """A prefs text field look."""

    k: Required[Literal["text"]]
    value: str
    placeholder: str
    secret: bool
    mono: bool


class KeysControl(TypedDict):
    """A prefs row of key chords."""

    k: Literal["keys"]
    keys: Sequence[Sequence[str]]


class MultiControl(TypedDict, total=False):
    """A prefs set of toggle chips, or an ordered list."""

    k: Required[Literal["multi"]]
    values: Sequence[str]
    options: Required[Sequence[ChoiceOption]]
    ordered: bool


class ActionControl(TypedDict, total=False):
    """A prefs action button."""

    k: Required[Literal["action"]]
    act: str
    label: str


PrefsControl: TypeAlias = (
    SwitchControl | ChoiceControl | NumberControl | TextControl | KeysControl | MultiControl | ActionControl
)
"""A prefs row's control."""


class PrefsRow(TypedDict, total=False):
    """A prefs row."""

    id: Required[str]
    label: str
    hint: str
    warning: str
    disabled: str
    changed: bool
    defaultLabel: str
    control: PrefsControl


class PrefsSection(TypedDict, total=False):
    """A prefs section."""

    id: str
    title: str
    page: str
    rows: Required[Sequence[PrefsRow]]


class PrefsEditing(TypedDict, total=False):
    """The prefs row being edited with the keyboard."""

    row: Required[str]
    option: str
    draft: str
    cursor: int


class ListMax(TypedDict):
    """A list height cap in rows."""

    lines: int
