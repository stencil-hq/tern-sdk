"""Typed shapes of node props: enumerations, spans and structured values.

TypedDict keys are the wire's own names (camelCase where the wire uses it),
since these values travel as they are."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import TYPE_CHECKING, Literal, Required, TypeAlias, TypedDict

if TYPE_CHECKING:
    from ._node import Node

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
ButtonVariant: TypeAlias = Literal["default", "primary", "quiet", "danger"]
"""A button's chrome."""
ButtonSize: TypeAlias = Literal["sm", "md"]
"""A button's height: md 28px for bars and heads, sm 22px inline."""
TabsVariant: TypeAlias = Literal["underline", "segmented"]
"""A tab strip's look."""
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
ChartKind: TypeAlias = Literal["heatmap", "bars", "spark", "line", "area", "donut"]
"""Which chart a `chart` draws."""
EffortLevel: TypeAlias = Literal["off", "minimal", "low", "medium", "high", "xhigh", "max"]
"""A thinking-effort rung."""
OverlaySize: TypeAlias = Literal["sm", "md", "lg", "full"]
"""An overlay's width, or a full sheet."""
OverlayPlace: TypeAlias = Literal["center", "top", "bottom"]
"""An overlay placed in the layer."""
OverlaySide: TypeAlias = Literal["right", "left", "bottom"]
"""The layer edge an overlay docks to: a full-height sheet or a bottom drawer."""
OverlayVariant: TypeAlias = Literal["edge", "inset"]
"""A docked overlay flush with its edge, or floating off it."""
TargetKind: TypeAlias = Literal["command", "path", "pattern", "query", "text"]
"""How a tool's string `target` is drawn."""
ToolFrame: TypeAlias = Literal["card", "inline"]
"""Whether a tool has a ring."""
ToolStatus: TypeAlias = Literal["pending", "running", "done", "error", "cancelled", "ask"]
"""The state of a `tool`; `ask` waits for approval."""
ToolConfirm: TypeAlias = Literal["card", "strip", "scopes"]
"""A tool's approval look while its status is `ask`."""
Approval: TypeAlias = Literal["once", "session", "always", "denied"]
"""A tool's resolved approval."""
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
Size4: TypeAlias = Literal["xs", "sm", "md", "lg"]
"""An avatar's size: 16, 20, 28 or 36px square."""
InputVariant: TypeAlias = Literal["well", "field"]
"""An `input`'s or `editor`'s look: a plain well or a framed field."""
SelectVariant: TypeAlias = Literal["field", "quiet"]
"""A select's trigger."""
SwitchVariant: TypeAlias = Literal["default", "compact", "segment"]
"""A switch's look."""
SliderVariant: TypeAlias = Literal["default", "accent"]
"""A slider's look."""
SliderFormat: TypeAlias = Literal["number", "percent", "duration"]
"""How a slider's value reads."""
FieldVariant: TypeAlias = Literal["stacked", "row", "grid"]
"""A form field's layout."""
OtpVariant: TypeAlias = Literal["cells", "joined", "underline"]
"""An otp's look."""
ButtonsVariant: TypeAlias = Literal["joined", "split", "toolbar"]
"""A button group's look."""
AvatarVariant: TypeAlias = Literal["circle", "square"]
"""An avatar's shape."""
BreadcrumbVariant: TypeAlias = Literal["chevron", "path"]
"""A breadcrumb's look."""
PagerVariant: TypeAlias = Literal["range", "pages", "more"]
"""A pager's look."""
EmptyVariant: TypeAlias = Literal["center", "inline", "drop"]
"""An empty state's look."""
SkeletonVariant: TypeAlias = Literal["shimmer", "pulse", "wire"]
"""A skeleton's look."""
SkeletonShape: TypeAlias = Literal["text", "avatar", "rows", "card", "block"]
"""What a skeleton stands for."""
CalendarVariant: TypeAlias = Literal["month", "week"]
"""A calendar's look: a month grid, or presets beside a week strip."""
WeekStart: TypeAlias = Literal["mon", "sun"]
"""A month grid's first column."""
MenuTrigger: TypeAlias = Literal["default", "quiet", "primary"]
"""A menu trigger's look (the `button` variants)."""
MenuVariant: TypeAlias = Literal["default", "dense"]
"""A menu's look."""
MenubarVariant: TypeAlias = Literal["strip", "pill"]
"""A menubar's look."""
NavVariant: TypeAlias = Literal["pills", "links"]
"""A nav's look."""
SplitVariant: TypeAlias = Literal["hairline", "gutter", "tiles"]
"""A split's look."""
ScrollVariant: TypeAlias = Literal["bar", "fade"]
"""A scroll's look."""
CarouselVariant: TypeAlias = Literal["peek", "arrows", "strip"]
"""A carousel's look."""
MessageVariant: TypeAlias = Literal["auto", "bubble", "flat"]
"""A message's look."""
AttachmentVariant: TypeAlias = Literal["chip", "tile", "pill"]
"""An attachment's look."""
AttachmentState: TypeAlias = Literal["idle", "uploading", "processing", "error", "done"]
"""An attachment's upload state."""
TraceVariant: TypeAlias = Literal["rail", "summary", "card"]
"""A trace's look."""
TraceStatus: TypeAlias = Literal["pending", "running", "done", "error"]
"""The state of a trace or one of its lines."""
StacktraceVariant: TypeAlias = Literal["frames", "split", "mono"]
"""A stacktrace's look."""
SuggestionsVariant: TypeAlias = Literal["chips", "rows", "cards"]
"""A suggestions list's look."""
QuestionsVariant: TypeAlias = Literal["steps", "inline", "list"]
"""A questionnaire's look."""
RuleVariant: TypeAlias = Literal["line", "note", "pin"]
"""A rule's look: a hairline, a note row or a pinned transcript marker."""
CiteLook: TypeAlias = Literal["chip", "pill", "note"]
"""How a text's cited spans look."""
TableVariant: TypeAlias = Literal["check", "bar"]
"""How a table's picks and sort look."""
SortDir: TypeAlias = Literal["asc", "desc"]
"""A sort direction."""
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


class Cite(TypedDict, total=False):
    """A source a span cites; one without `href` is ignored."""

    href: Required[str]
    title: str
    n: int
    """The number shown (default: the next free one in order of appearance)."""
    site: str
    """The pill's name (default: the `href`'s host)."""
    snippet: str
    """A quote shown in the hover card."""


class Span(TypedDict, total=False):
    """A styled run of text: `t` the text, `s` space-separated style tokens,
    `cite` the sources it cites (in a `text`)."""

    t: Required[str]
    s: str
    fx: Fx
    href: str
    cite: Cite | Sequence[Cite]


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
    menu: Sequence[MenuEntry]


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


class TextCell(TypedDict, total=False):
    """A table cell of text led by a status dot (`dot`) or an icon colored by `tone`."""

    text: Required[Text]
    icon: IconName
    dot: Tone
    tone: Tone
    title: str


class BadgeCellProps(TypedDict, total=False):
    """The badge inside a table cell."""

    text: Required[str]
    tone: Tone
    title: str


class BadgeCell(TypedDict):
    """A table cell drawn as a badge."""

    badge: BadgeCellProps


class ButtonCellProps(TypedDict, total=False):
    """The small button inside a table cell; a click sends action `act` with
    the row id as `item` and does not select the row."""

    text: Required[str]
    act: Required[str]
    icon: IconName
    keys: Sequence[str]
    variant: ButtonVariant
    disabled: bool | str


class ButtonCell(TypedDict):
    """A table cell drawn as a small button."""

    button: ButtonCellProps


class SparkCellProps(TypedDict, total=False):
    """The sparkline inside a table cell."""

    values: Required[Sequence[float]]
    tone: Tone
    min: float
    max: float
    title: str


class SparkCell(TypedDict):
    """A table cell drawn as a sparkline."""

    spark: SparkCellProps


TableCell: TypeAlias = Text | MeterCell | TextCell | BadgeCell | ButtonCell | SparkCell
"""One table cell: text, or a meter, text-with-dot, badge, button or sparkline cell."""


class TableCol(TypedDict, total=False):
    """A table column."""

    id: Required[str]
    head: Text
    align: TextAlign
    truncate: Truncate
    priority: float
    grow: float
    mono: bool
    sortable: bool
    """Its header takes clicks and sends `action` `sort`."""


class TableSort(TypedDict, total=False):
    """The column a table's rows are sorted by."""

    col: Required[str]
    dir: SortDir


class BarAction(TypedDict, total=False):
    """A small button in a table's selection bar; a click sends `act` with
    the picked ids as `value`."""

    act: str
    text: str
    icon: IconName
    keys: Sequence[str]
    variant: ButtonVariant
    disabled: bool | str
    title: str


class TableRow(TypedDict, total=False):
    """A table row: one cell per column id. `disabled` (true or a reason)
    dims it and blocks select/activate; `menu` names right-click actions."""

    id: Required[str]
    cells: Required[Mapping[str, TableCell]]
    disabled: bool | str
    menu: Sequence[str]
    title: str


class StatItem(TypedDict, total=False):
    """One tile of a `stats` well."""

    label: Required[Text]
    value: Required[Text]
    sub: Text
    tone: Tone
    title: str


class TreeNode(TypedDict, total=False):
    """One item of a `tree`."""

    id: Required[str]
    label: Text
    icon: IconName
    open: bool
    children: Sequence[TreeNode]


CodeMark = TypedDict(
    "CodeMark",
    {"line": Required[int], "tone": Tone, "heat": float, "ranges": Sequence[Sequence[int]], "wash": Sequence[float]},
    total=False,
)
"""A marked line of a `code` block, with byte ranges drawn stronger. `wash`
is scores 0–1 washing the row on the heat ramp: one flat, several a gradient
across it, left to right; in place of `heat`'s wash, which still drives the
overview and fold sparkline."""


CodeFold = TypedDict(
    "CodeFold",
    {
        "from": Required[int],
        "to": Required[int],
        "label": str,
        "note": "Text",
        "tone": Tone,
        "open": bool,
        "hovercard": "Node | Mapping[str, object]",
    },
    total=False,
)
"""A run of lines a `code` block draws as one row until it is opened.
`from`/`to` are the first and last folded line in the gutter's numbering;
`label` the folded row's text in the code's face (default `… [lines {from}-{to}]`,
`… [line {from}]` for one line); `note`
what the fold hides, after a sparkline of its lines' heat; `tone` `error`
tints the row (any tone colors the note); `open` draws it open (a click on
the folded row or an open fold's head toggles it in place); `hovercard` a
card shown while the pointer rests on the folded row."""


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
    """One bar of a `bars` or `spark` chart, one point of a single-series
    `line` or `area` chart (`label` its x label), or one `donut` segment."""

    value: Required[float]
    label: str
    title: str
    tone: Tone
    """This bar's fill instead of the chart's `token`."""
    token: str
    """A donut segment's color."""


class ChartValue(TypedDict, total=False):
    """One point of a `line` or `area` chart series."""

    value: Required[float]
    title: str


class ChartSeries(TypedDict, total=False):
    """One series of a multi-series `line` or `area` chart."""

    label: str
    token: str
    values: Required[Sequence[float | ChartValue | None]]


class ChartMark(TypedDict, total=False):
    """A dashed reference line (a target, a limit) across a `bars` or `spark`
    chart, in value units; outside the domain it is clamped to its edge."""

    value: Required[float]
    tone: Tone
    """The line's color (default muted foreground)."""
    title: str
    """The line's tooltip."""


class ChartCol(TypedDict):
    """A heatmap column label."""

    at: int
    label: str


Decor = TypedDict("Decor", {"from": Required[int], "to": Required[int], "s": Required[str], "fx": Fx}, total=False)
"""A styled UTF-16 range of an editor's or input's text."""


class AddonButton(TypedDict, total=False):
    """A button attached to an `input` or `editor`; a click sends `act`
    (`"copy"` copies the text)."""

    act: Required[str]
    text: str
    icon: IconName
    variant: ButtonVariant
    keys: Sequence[str]
    title: str
    disabled: bool | str
    busy: bool


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


class ToolAskScope(TypedDict, total=False):
    """One radio of a tool's `scopes` approval look."""

    id: Required[str]
    label: str


class ToolAsk(TypedDict, total=False):
    """A tool's approval panel: the question, the button labels (`always`
    `False` hides that button) and the scope radios."""

    text: str
    allow: str
    deny: str
    always: str | Literal[False]
    scopes: Sequence[ToolAskScope]
    scope: str


class SelectOption(TypedDict, total=False):
    """One option of a `select`. `icon` is an icon name or `provider:<id>`."""

    value: Required[str]
    label: str
    detail: str
    icon: str
    group: str
    disabled: bool


class AvatarItem(TypedDict, total=False):
    """One avatar of an `avatars` stack: the `avatar` props but `size`.
    `icon` is an icon name or `provider:<id>`; `status` a presence dot."""

    variant: AvatarVariant
    text: str
    seed: str
    icon: str
    blob: str
    status: Tone
    title: str


class Crumb(TypedDict, total=False):
    """One place of a `breadcrumb`."""

    id: Required[str | int]
    label: str
    icon: IconName
    href: str


class CalendarMark(TypedDict, total=False):
    """A calendar day carrying a dot."""

    date: Required[str]
    tone: Tone


class CalendarPreset(TypedDict, total=False):
    """A preset of a `week` calendar."""

    id: Required[str]
    label: str


class MenuItem(TypedDict, total=False):
    """One command of a menu (a `menu`, a `menubar` menu or `actions.menu`):
    `checked` makes it a check row, `radio` a radio row of that group,
    `items` a submenu."""

    id: Required[str]
    label: str
    icon: IconName
    keys: str | Sequence[str]
    detail: str
    checked: bool
    radio: str
    disabled: bool | str
    danger: bool
    items: Sequence[MenuEntry]


class MenuHeader(TypedDict):
    """A dim section title in a menu."""

    header: str


MenuEntry: TypeAlias = str | MenuItem | MenuHeader
"""A menu row: `"-"` a separator, another string an action labeled from its
id, a header or an item."""


class MenubarMenu(TypedDict, total=False):
    """One menu of a `menubar`."""

    id: Required[str]
    label: str
    items: Required[Sequence[MenuEntry]]


class NavSubItem(TypedDict, total=False):
    """A sub-item in a nav item's menu or panel."""

    id: Required[str]
    label: str
    detail: str | float
    icon: IconName


class NavItem(TypedDict, total=False):
    """One item of a `nav`; with `items` it opens a menu or panel."""

    id: Required[str]
    label: str
    icon: IconName
    count: str | int
    items: Sequence[NavSubItem]


class IconAction(TypedDict, total=False):
    """An icon button of a `message` or `attachment`; a click sends `act`."""

    act: Required[str]
    icon: IconName
    title: str


class Reaction(TypedDict, total=False):
    """A reaction chip of a `message`; `text` (an emoji) is its identity."""

    text: Required[str]
    count: int
    on: bool


class TraceSource(TypedDict, total=False):
    """A source chip of a trace or trace line; a click opens `href`."""

    title: str
    href: str
    site: str


class TraceLine(TypedDict, total=False):
    """One step of a `trace`. A child keyed by its `id` is its content."""

    id: str
    icon: IconName
    title: Text
    target: Text
    detail: Text
    status: TraceStatus
    age: float
    took: float
    open: bool
    sources: Sequence[TraceSource]


class StackError(TypedDict, total=False):
    """The error a `stacktrace` shows."""

    type: str
    message: str


class StackCode(TypedDict, total=False):
    """A source excerpt of a stack frame; `line` is the one marked."""

    start: int
    lines: Required[Sequence[str]]
    line: int


class StackFrame(TypedDict, total=False):
    """One stack frame; `app` false folds a library's frame."""

    fn: str
    file: str
    line: int
    col: int
    app: bool
    code: StackCode


class Suggestion(TypedDict, total=False):
    """One suggested prompt; `activate` names its `id` (else its `text`)."""

    id: str
    text: Required[str]
    detail: str
    icon: IconName
    keys: Sequence[str]


class QuestionChoice(TypedDict, total=False):
    """A choice of a question."""

    value: Required[str]
    label: str
    detail: str


class QuestionOther(TypedDict, total=False):
    """A free-text answer, typed into the `input` child keyed `<id>.other`."""

    placeholder: str


class Question(TypedDict, total=False):
    """One question of a `questions` node."""

    id: Required[str]
    prompt: str
    description: str
    choices: Sequence[QuestionChoice | str]
    multi: bool
    other: QuestionOther | bool
    required: bool
