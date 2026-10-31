"""Typed builders for every kind except `block` (Tern's own), `list` and
`input` (in `_shadow`, since their names shadow builtins)."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any, Literal, Unpack

from ..wire import Json
from ._node import (
    Common,
    CommonNoActions,
    CommonNoBounds,
    CommonNoMax,
    CommonNoMin,
    CommonNoTitle,
    Handlers,
    Node,
    PickerCommon,
    make,
    text_props,
)  # fmt: skip
from ._types import (
    AddonButton,
    AgentRetry,
    AgentStats,
    AgentStatus,
    AgentTool,
    Align,
    Approval,
    AttachmentState,
    AttachmentVariant,
    AvatarItem,
    AvatarVariant,
    Badge,
    BarAction,
    BreadcrumbVariant,
    ButtonSize,
    ButtonsVariant,
    ButtonVariant,
    CalendarMark,
    CalendarPreset,
    CalendarVariant,
    CaretAnchor,
    CarouselVariant,
    ChartCol,
    ChartKind,
    ChartMark,
    ChartPoint,
    ChartSeries,
    ChecklistMode,
    Cite,
    CiteLook,
    CodeFold,
    CodeMark,
    Crumb,
    Decor,
    DiffMode,
    EffortLevel,
    ElapsedFormat,
    EmptyVariant,
    Extent,
    FieldVariant,
    Fx,
    Gap,
    Hunk,
    IconAction,
    IconName,
    InputVariant,
    Justify,
    KvItem,
    KvLayout,
    LinesPreview,
    MenubarMenu,
    MenubarVariant,
    MenuEntry,
    MenuTrigger,
    MenuVariant,
    MessageVariant,
    MeterMark,
    MeterPart,
    MeterStyle,
    NavItem,
    NavVariant,
    NodeAnchor,
    OtpVariant,
    OverlayPlace,
    OverlaySide,
    OverlaySize,
    OverlayVariant,
    PagerVariant,
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
    Question,
    QuestionsVariant,
    Reaction,
    RuleVariant,
    ScrollVariant,
    SelectOption,
    SelectVariant,
    ShimmerMode,
    ShimmerPalette,
    Size3,
    Size4,
    SkeletonShape,
    SkeletonVariant,
    SliderFormat,
    SliderVariant,
    Span,
    Spans,
    SpinnerStyle,
    SplitVariant,
    StackError,
    StackFrame,
    StacktraceVariant,
    StatItem,
    Status,
    Suggestion,
    SuggestionsVariant,
    SwitchVariant,
    Tab,
    TableCol,
    TableRow,
    TableSort,
    TableVariant,
    TabsVariant,
    TailPreview,
    TargetKind,
    Text,
    Thresholds,
    Tone,
    ToolAsk,
    ToolButton,
    ToolConfirm,
    ToolFrame,
    ToolStatus,
    TraceLine,
    TraceSource,
    TraceStatus,
    TraceVariant,
    TreeNode,
    Truncate,
    WeekStart,
    Wrap,
)


def span(
    t: str,
    s: str | None = None,
    *,
    fx: Fx | None = None,
    href: str | None = None,
    cite: Cite | Sequence[Cite] | None = None,
) -> Span:
    """A styled span: text `t` with space-separated style tokens `s`; `cite`
    the sources it cites (in a `text`)."""
    out: Span = {"t": t}
    if s is not None:
        out["s"] = s
    if fx is not None:
        out["fx"] = fx
    if href is not None:
        out["href"] = href
    if cite is not None:
        out["cite"] = cite
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


def _slotted(
    kind: str,
    own: dict[str, Any],
    children: Sequence[Node | None],
    common: Mapping[str, Any],
    names: Sequence[str] = ("head", "end", "foot"),
) -> Node:
    """A node whose slot props (`head`/`end`/`foot` unless `names` says
    otherwise) given as nodes become children keyed by the prop's name; the
    prop is sent as that child's id."""
    extra: list[Node] = []
    slots: dict[str, str] = {}
    for prop in names:
        value = own.get(prop)
        if isinstance(value, Node):
            extra.append(Node(value.kind, {**value.props, "key": prop}, value.children, value.handlers, value.slots))
            slots[prop] = prop
            own[prop] = None
    out = make(kind, own, [*children, *extra], common)
    out.slots = slots
    return out


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
    head: Text | Node | None = None,
    icon: IconName | None = None,
    meta: Text | None = None,
    end: Node | str | None = None,
    status: Status | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    preview: Literal["auto"] | LinesPreview | None = None,
    selected: bool | None = None,
    inset: bool | None = None,
    variant: Literal["bare"] | None = None,
    **common: Unpack[Common],
) -> Node:
    """A framed box: a head row (icon, head, `meta`, `end` controls, status
    chip, chevron) over a body. `head` or `end` given as a node becomes a
    child keyed `head`/`end`, drawn in the head row."""
    own: dict[str, Any] = {
        "head": head, "icon": icon, "meta": meta, "end": end, "status": status,
        "collapsible": collapsible, "collapsed": collapsed, "preview": preview,
        "selected": selected, "inset": inset, "variant": variant,
    }  # fmt: skip
    return _slotted("card", own, children, common)


def section(
    *children: Node | None,
    head: Text | Node | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """An unframed disclosure group: chevron and head over an indented body."""
    own: dict[str, Any] = {"head": head, "collapsible": collapsible, "collapsed": collapsed}
    return _slotted("section", own, children, common)


def rule(
    label: Text | None = None,
    *,
    variant: RuleVariant | None = None,
    icon: IconName | None = None,
    time: str | None = None,
    end: Node | str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A hairline divider, optionally labeled; `note` and `pin` draw a note
    row or a pinned transcript marker (a checkpoint, a join). `time` follows
    the label; `end` given as a node (a Restore button) becomes a child keyed
    `end`, drawn at the end."""
    own: dict[str, Any] = {"label": label, "variant": variant, "icon": icon, "time": time, "end": end}
    return _slotted("rule", own, (), common, ("end",))


def spacer(size: Gap | None = None, **common: Unpack[Common]) -> Node:
    """A fixed block of vertical space."""
    return make("spacer", {"size": size}, (), common)


def split(
    *children: Node | None,
    variant: SplitVariant | None = None,
    dir: Literal["row", "col"] | None = None,
    sizes: Sequence[float] | None = None,
    min: Sequence[Extent | None] | Extent | None = None,
    h: Extent | None = None,
    active: int | None = None,
    **common: Unpack[CommonNoMin],
) -> Node:
    """Panes, one per child, the user resizes by dragging the handles between
    them; a drag sends `change` with every pane's share. `sizes` are the
    shares, `min` each pane's minimum, `active` the handle the keys move."""
    own = {"variant": variant, "dir": dir, "sizes": sizes, "min": min, "h": h, "active": active}
    return make("split", own, children, common)


def scroll(
    *children: Node | None,
    variant: ScrollVariant | None = None,
    stick: Literal["end"] | None = None,
    pinned: bool | None = None,
    bar: Literal["auto", "always"] | None = None,
    gap: Gap | None = None,
    noun: str | tuple[str, str] | None = None,
    **common: Unpack[Common],
) -> Node:
    """A box that scrolls its children inside its bound; `stick="end"` keeps
    it at the end while content grows (logs, chats) until the user scrolls up."""
    own = {"variant": variant, "stick": stick, "pinned": pinned, "bar": bar, "gap": gap, "noun": noun}
    return make("scroll", own, children, common)


def carousel(
    *children: Node | None,
    index: int | None = None,
    loop: bool | None = None,
    variant: CarouselVariant | None = None,
    h: Extent | None = None,
    **common: Unpack[Common],
) -> Node:
    """Slides (the children) shown one at a time; moving sends `change` with
    the index shown."""
    return make("carousel", {"index": index, "loop": loop, "variant": variant, "h": h}, children, common)


def empty(
    *children: Node | None,
    title: Text | None = None,
    text: Text | None = None,
    variant: EmptyVariant | None = None,
    icon: IconName | Literal[""] | None = None,
    frame: bool | None = None,
    over: bool | None = None,
    **common: Unpack[CommonNoTitle],
) -> Node:
    """An empty state: icon, title, a muted line and the actions that fill it
    (the children, usually one `button`). `tone="error"` for a failed load."""
    own = {"title": title, "text": text, "variant": variant, "icon": icon, "frame": frame, "over": over}
    return make("empty", own, children, common)


# Text and code


def text(
    text: Text,
    *,
    wrap: Wrap | None = None,
    truncate: Truncate | None = None,
    lines: int | None = None,
    measure: Literal["prose"] | None = None,
    cite: CiteLook | None = None,
    sources: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """Styled text that wraps, clamps or truncates (a string or spans).
    Spans with `cite` draw citation markers in the `cite` look; `sources`
    draws the numbered sources list under the text."""
    own = {
        **text_props(text), "wrap": wrap, "truncate": truncate, "lines": lines, "measure": measure,
        "cite": cite, "sources": sources,
    }  # fmt: skip
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
    nums: Sequence[int] | None = None,
    marks: Sequence[CodeMark] | None = None,
    wrap: bool | None = None,
    folds: Sequence[CodeFold] | None = None,
    overview: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """A highlighted code block. `folds` are runs of lines drawn as one row until opened; `overview` shows a strip
    of the lines' heat above them that scrolls the block on click or drag.
    `nums` is each line's number, strictly ascending, in place of counting
    from `start`; a `…` row is drawn where the numbers jump."""
    own = {
        "text": text, "lang": lang, "path": path, "numbers": numbers, "start": start, "nums": nums, "marks": marks,
        "wrap": wrap, "folds": folds, "overview": overview,
    }  # fmt: skip
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


def table(
    cols: Sequence[TableCol],
    rows: Sequence[TableRow],
    *,
    selectable: bool | None = None,
    selected: str | None = None,
    empty: Text | None = None,
    sort: TableSort | None = None,
    multi: bool | None = None,
    picked: Sequence[str] | None = None,
    variant: TableVariant | None = None,
    bar: Sequence[BarAction] | None = None,
    hint: Text | None = None,
    **common: Unpack[Common],
) -> Node:
    """A table with priority-hidden columns and rich cells. `selectable` rows
    send `select` on click and `activate` on double-click (the row id as
    `item`); `selected` draws a row selected; `empty` shows when there are no rows.
    `sort` draws the sorted column's arrow (a click on a `sortable` column's
    header sends `action` `sort` with `{col, dir}`); `multi` rows are picked
    several at a time (`change` with the picked ids), `picked` holds them and
    `bar` are the selection bar's buttons, `hint` muted text after its count."""
    own = {
        "cols": cols, "rows": rows, "selectable": selectable, "selected": selected, "empty": empty,
        "sort": sort, "multi": multi, "picked": picked, "variant": variant, "bar": bar, "hint": hint,
    }  # fmt: skip
    return make("table", own, (), common)


def stats(items: Sequence[StatItem], **common: Unpack[Common]) -> Node:
    """A well of stat tiles: label over a large value over an optional sub line."""
    return make("stats", {"items": items}, (), common)


def tree(nodes: Sequence[TreeNode], **common: Unpack[Common]) -> Node:
    """A disclosure tree."""
    return make("tree", {"nodes": nodes}, (), common)


def json(value: Json, *, open: int | None = None, **common: Unpack[Common]) -> Node:  # noqa: A002
    """A JSON value Tern draws natively as a foldable tree; `open` levels of
    containers start expanded. Check the hello reply's `kinds` for `json` and
    fall back to `tree` otherwise."""
    return make("json", {"value": value, "open": open}, (), common)


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


def media(path: str, *, rev: str | None = None, link: str | None = None, **common: Unpack[Common]) -> Node:
    """A file Tern loads by path (or from git at `rev`) and shows in its media viewer."""
    return make("media", {"path": path, "rev": rev, "link": link}, (), common)


def avatar(
    text: str = "",
    *,
    variant: AvatarVariant | None = None,
    seed: str | None = None,
    icon: str | None = None,
    blob: str | None = None,
    size: Size4 | None = None,
    status: Tone | None = None,
    **common: Unpack[Common],
) -> Node:
    """A person's, bot's or org's mark: the initials of `text` on a gradient
    picked by `seed`, an icon (`provider:<id>` for a provider's logo) or a
    picture blob; `status` is a presence dot."""
    own = {"text": text, "variant": variant, "seed": seed, "icon": icon, "blob": blob, "size": size, "status": status}
    return make("avatar", own, (), common)


def avatars(
    items: Sequence[AvatarItem],
    *,
    max: int | None = None,
    size: Size4 | None = None,
    label: str | None = None,
    **common: Unpack[CommonNoMax],
) -> Node:
    """Avatars stacked, front first: `max` shown before a `+N` chip, `label`
    muted after the stack."""
    return make("avatars", {"items": items, "max": max, "size": size, "label": label}, (), common)


def attachment(
    name: str = "",
    *,
    variant: AttachmentVariant | None = None,
    ext: str | None = None,
    mime: str | None = None,
    size: float | None = None,
    meta: str | None = None,
    blob: str | None = None,
    path: str | None = None,
    state: AttachmentState | None = None,
    progress: float | None = None,
    error: str | None = None,
    removable: bool | None = None,
    actions: Sequence[IconAction] | None = None,
    **common: Unpack[CommonNoActions],
) -> Node:
    """A file attached to a message or a composer: a chip, an image tile or a
    pill, with its upload `state` and `progress` (0-1). `removable` adds a ×
    that sends `action` `remove`; `actions` are icon buttons before it. A
    click opens `href`, else `path`."""
    own = {
        "name": name, "variant": variant, "ext": ext, "mime": mime, "size": size, "meta": meta,
        "blob": blob, "path": path, "state": state, "progress": progress, "error": error,
        "removable": removable, "actions": actions,
    }  # fmt: skip
    return make("attachment", own, (), common)


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


def tabs(
    items: Sequence[Tab],
    *,
    active: str | None = None,
    variant: TabsVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """A tab strip; `active` is the active tab's id; `variant="segmented"`
    draws an inline segmented control."""
    return make("tabs", {"items": items, "active": active, "variant": variant}, (), common)


def breadcrumb(
    items: Sequence[Crumb],
    *,
    variant: BreadcrumbVariant | None = None,
    max: int | None = None,
    open: bool | None = None,  # noqa: A002
    active: str | None = None,
    **common: Unpack[CommonNoMax],
) -> Node:
    """A trail of places, root first, the last one current; a click on
    another crumb sends `select` with its id as `item`. Past `max` crumbs
    the middle folds into a menu (`open` shows it)."""
    own = {"items": items, "variant": variant, "max": max, "open": open, "active": active}
    return make("breadcrumb", own, (), common)


def pager(
    page: int | None = None,
    *,
    variant: PagerVariant | None = None,
    pages: int | None = None,
    total: int | None = None,
    size: int | None = None,
    loading: bool | None = None,
    label: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """Pagination: a range with arrows, page numbers or a show-more row. A
    click sends `change` with the new page (1-based), the more row `action`
    `more`."""
    own = {
        "page": page, "variant": variant, "pages": pages, "total": total, "size": size,
        "loading": loading, "label": label,
    }  # fmt: skip
    return make("pager", own, (), common)


def nav(
    items: Sequence[NavItem],
    *,
    active: str | None = None,
    open: str | None = None,  # noqa: A002
    variant: NavVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """Navigation links; an item with `items` opens a menu of them (`open` is
    that item's id). A click on a link sends `select` with its id as `item`."""
    return make("nav", {"items": items, "active": active, "open": open, "variant": variant}, (), common)


def menu(
    items: Sequence[MenuEntry],
    *,
    text: str | None = None,
    icon: IconName | None = None,
    trigger: MenuTrigger | None = None,
    caret: bool | None = None,
    size: ButtonSize | None = None,
    variant: MenuVariant | None = None,
    open: bool | None = None,  # noqa: A002
    active: str | None = None,
    align: Literal["start", "end"] | None = None,
    disabled: bool | str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A button that opens a menu of commands. Choosing a row sends `action`
    with its id as `act` (and `checked`, the new state, for check and radio
    rows); `on_action={id: fn}` handles it. The pointer opening or closing it
    sends `toggle` with `open`."""
    own = {
        "items": items, "text": text, "icon": icon, "trigger": trigger, "caret": caret, "size": size,
        "variant": variant, "open": open, "active": active, "align": align, "disabled": disabled,
    }  # fmt: skip
    return make("menu", own, (), common)


def menubar(
    menus: Sequence[MenubarMenu],
    *,
    variant: MenubarVariant | None = None,
    icon: IconName | None = None,
    open: str | None = None,  # noqa: A002
    active: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A strip of menus. Opening or closing one sends `toggle` with its id as
    `item` and `open`; choosing a row sends `action` with the row's id as
    `act` and the menu's as `item`."""
    own = {"menus": menus, "variant": variant, "icon": icon, "open": open, "active": active}
    return make("menubar", own, (), common)


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


def skeleton(
    shape: SkeletonShape | None = None,
    *,
    variant: SkeletonVariant | None = None,
    lines: int | None = None,
    rows: int | None = None,
    tiles: int | None = None,
    **common: Unpack[Common],
) -> Node:
    """Placeholder shapes while content loads: `lines` of text, an avatar,
    `rows`, a card of stat `tiles` or a block."""
    own = {"shape": shape, "variant": variant, "lines": lines, "rows": rows, "tiles": tiles}
    return make("skeleton", own, (), common)


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
    series: Sequence[ChartPoint | ChartSeries | float | None] | None = None,
    min: float | None = None,
    max: float | None = None,
    marks: Sequence[ChartMark] | None = None,
    cells: Sequence[Sequence[float | None]] | None = None,
    tips: Sequence[Sequence[str | None]] | None = None,
    rows: Sequence[str] | None = None,
    cols: Sequence[ChartCol] | None = None,
    labels: Sequence[str] | None = None,
    unit: str | None = None,
    last: bool | Text | None = None,
    total: Text | None = None,
    caption: Text | None = None,
    **common: Unpack[CommonNoBounds],
) -> Node:
    """A heatmap, a bar chart, a sparkline, a line or area chart or a donut.
    For `bars`/`spark`/`line`/`area`, `min`/`max` fix the value domain and
    `marks` draw dashed reference lines. `line`/`area` take one series of
    points (`None` a gap) or several `ChartSeries`, x `labels` and a `last`
    value label; a `donut` takes segments, its middle `total` and a
    `caption`. `unit` follows every value printed."""
    own = {
        "kind": kind, "size": size, "token": token, "summary": summary, "series": series,
        "min": min, "max": max, "marks": marks, "cells": cells, "tips": tips, "rows": rows,
        "cols": cols, "labels": labels, "unit": unit, "last": last, "total": total,
        "caption": caption,
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
    placeholder: Text | None = None,
    prompt: Text | None = None,
    mode: str | None = None,
    lang: str | None = None,
    readonly: bool | None = None,
    sendable: bool | None = None,
    typeahead: bool | None = None,
    max_lines: int | None = None,
    variant: InputVariant | None = None,
    buttons: Sequence[AddonButton] | None = None,
    **common: Unpack[Common],
) -> Node:
    """A multi-line text field; the program owns its text and caret.
    `buttons` are attached buttons (each sends `action` with its `act`)."""
    own = {
        "text": text, "cursor": cursor, "anchor": anchor, "decor": decor, "ghost": ghost,
        "placeholder": placeholder, "prompt": prompt, "mode": mode, "lang": lang,
        "readonly": readonly, "sendable": sendable, "typeahead": typeahead,
        "maxLines": max_lines, "variant": variant, "buttons": buttons,
    }  # fmt: skip
    return make("editor", own, (), common)


def button(
    text: str = "",
    *,
    icon: IconName | None = None,
    keys: Sequence[str] | None = None,
    variant: ButtonVariant | None = None,
    size: ButtonSize | None = None,
    on: bool | None = None,
    disabled: bool | str | None = None,
    busy: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """A push button. A click sends an `action` event (`actions.click`, else
    `"click"`); `on_click` handles it. `keys` show the shortcut (display
    only), `disabled` as a string is the reason tooltip. Icon-only buttons
    need a `title`."""
    own = {
        "text": text, "icon": icon, "keys": keys, "variant": variant, "size": size, "on": on,
        "disabled": disabled, "busy": busy,
    }  # fmt: skip
    return make("button", own, (), common)


def buttons(
    *children: Node | None,
    variant: ButtonsVariant | None = None,
    size: ButtonSize | None = None,
    label: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """`button` children drawn as one control: joined, a split button (an
    action and its menu) or a toolbar well. `size` wins over the children's."""
    return make("buttons", {"variant": variant, "size": size, "label": label}, children, common)


def select(
    options: Sequence[str | SelectOption],
    *,
    value: str | None = None,
    variant: SelectVariant | None = None,
    label: str | None = None,
    placeholder: str | None = None,
    size: ButtonSize | None = None,
    disabled: bool | str | None = None,
    open: bool | None = None,  # noqa: A002
    active: str | None = None,
    search: bool | None = None,
    query: str | None = None,
    cursor: int | None = None,
    total: int | None = None,
    empty: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A value picked from a menu of `options`: a field or a quiet trigger,
    with `search` a combobox whose `query` the program filters. A pick sends
    `change` with the option's value; the pointer opening or closing the menu
    sends `toggle` with `open`."""
    own = {
        "options": options, "value": value, "variant": variant, "label": label,
        "placeholder": placeholder, "size": size, "disabled": disabled, "open": open,
        "active": active, "search": search, "query": query, "cursor": cursor, "total": total,
        "empty": empty,
    }  # fmt: skip
    return make("select", own, (), common)


def switch(
    label: str | None = None,
    *,
    on: bool | None = None,
    variant: SwitchVariant | None = None,
    hint: str | None = None,
    labels: tuple[str, str] | None = None,
    disabled: bool | str | None = None,
    **common: Unpack[Common],
) -> Node:
    """An on/off switch; a flip sends `change` with the new state as `value`.
    `labels` are a `segment` switch's two sides, off first."""
    own = {"label": label, "on": on, "variant": variant, "hint": hint, "labels": labels, "disabled": disabled}
    return make("switch", own, (), common)


def slider(
    value: float | tuple[float, float] | None = None,
    *,
    min: float | None = None,
    max: float | None = None,
    step: float | None = None,
    label: str | None = None,
    unit: str | None = None,
    format: SliderFormat | None = None,
    ticks: bool | float | Sequence[float] | None = None,
    variant: SliderVariant | None = None,
    disabled: bool | str | None = None,
    **common: Unpack[CommonNoBounds],
) -> Node:
    """A value, or a `(lo, hi)` range, dragged along a track from `min` to
    `max` in `step`s; a move sends `change` with the new value or pair."""
    own = {
        "value": value, "min": min, "max": max, "step": step, "label": label, "unit": unit,
        "format": format, "ticks": ticks, "variant": variant, "disabled": disabled,
    }  # fmt: skip
    return make("slider", own, (), common)


def field(
    *children: Node | None,
    label: Text | None = None,
    hint: Text | None = None,
    error: Text | bool | None = None,
    warning: Text | bool | None = None,
    required: bool | None = None,
    label_width: Extent | None = None,
    variant: FieldVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """A labelled form row around its control children: label, help line,
    and an `error` or `warning` note in place of the help."""
    own = {
        "label": label, "hint": hint, "error": error, "warning": warning, "required": required,
        "labelWidth": label_width, "variant": variant,
    }  # fmt: skip
    return make("field", own, children, common)


def otp(
    text: str = "",
    *,
    length: int | None = None,
    group: int | None = None,
    cursor: int | Literal[False] | None = None,
    variant: OtpVariant | None = None,
    mask: bool | str | None = None,
    state: Literal["error", "success"] | None = None,
    hint: str | None = None,
    disabled: bool | str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A one-time code, one cell per character; the program owns its `text`
    and `cursor` and edits them from keys. A click on a cell sends `action`
    `focus` with the cell's index as `value`."""
    own = {
        "text": text, "length": length, "group": group, "cursor": cursor, "variant": variant,
        "mask": mask, "state": state, "hint": hint, "disabled": disabled,
    }  # fmt: skip
    return make("otp", own, (), common)


def calendar(
    value: str | tuple[str, str | None] | None = None,
    *,
    variant: CalendarVariant | None = None,
    range: bool | None = None,  # noqa: A002
    month: str | None = None,
    today: str | None = None,
    min: str | None = None,
    max: str | None = None,
    week_start: WeekStart | None = None,
    marks: Sequence[str | CalendarMark] | None = None,
    cursor: str | None = None,
    presets: Sequence[CalendarPreset] | None = None,
    preset: str | None = None,
    note: str | None = None,
    **common: Unpack[CommonNoBounds],
) -> Node:
    """A month grid, or presets beside a week strip, to pick a `"YYYY-MM-DD"`
    day or a `range`. A pick sends `change` (a day, or `[from, to]`), a
    preset `select`, stepping months `action` `month`."""
    own = {
        "value": value, "variant": variant, "range": range, "month": month, "today": today,
        "min": min, "max": max, "weekStart": week_start, "marks": marks, "cursor": cursor,
        "presets": presets, "preset": preset, "note": note,
    }  # fmt: skip
    return make("calendar", own, (), common)


def date(
    value: str | tuple[str, str | None] | None = None,
    *,
    range: bool | None = None,  # noqa: A002
    open: bool | None = None,  # noqa: A002
    placeholder: str | None = None,
    month: str | None = None,
    today: str | None = None,
    min: str | None = None,
    max: str | None = None,
    week_start: WeekStart | None = None,
    marks: Sequence[str | CalendarMark] | None = None,
    cursor: str | None = None,
    note: str | None = None,
    **common: Unpack[CommonNoBounds],
) -> Node:
    """A date field that opens a month calendar under it; picks send `change`
    as `calendar`'s, opening and closing `toggle` with `collapsed`."""
    own = {
        "value": value, "range": range, "open": open, "placeholder": placeholder, "month": month,
        "today": today, "min": min, "max": max, "weekStart": week_start, "marks": marks,
        "cursor": cursor, "note": note,
    }  # fmt: skip
    return make("date", own, (), common)


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
    size: OverlaySize | float | None = None,
    modal: bool | None = None,
    head: Text | None = None,
    icon: IconName | None = None,
    meta: Text | None = None,
    end: Node | str | None = None,
    foot: Node | str | None = None,
    side: OverlaySide | None = None,
    variant: OverlayVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """A floating panel or sheet, for the `layer` region. `end` (head-end
    controls) and `foot` (a sticky footer bar) given as nodes become children
    keyed `end`/`foot`. `side` docks it to that edge of the layer: a
    full-height sheet or a bottom drawer, `size` (or a fraction) its width or
    height, `variant="inset"` floating off the edge."""
    own: dict[str, Any] = {
        "anchor": anchor, "size": size, "modal": modal, "head": head, "icon": icon, "meta": meta,
        "end": end, "foot": foot, "side": side, "variant": variant,
    }  # fmt: skip
    return _slotted("overlay", own, children, common)


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
    status: ToolStatus | None = None,
    age: float | None = None,
    took: float | None = None,
    intent: str | None = None,
    frame: ToolFrame | None = None,
    collapsible: bool | None = None,
    collapsed: bool | None = None,
    preview: LinesPreview | TailPreview | None = None,
    tools: Sequence[ToolButton] | None = None,
    confirm: ToolConfirm | None = None,
    ask: ToolAsk | None = None,
    approved: Approval | None = None,
    **common: Unpack[CommonNoTitle],
) -> Node:
    """One step of work: a data head (verb, target, status, timer) over a body.
    `status="ask"` waits for approval in the `confirm` look with the `ask`
    panel; its buttons send `action` `allow`, `deny` or `always` (the scope
    as `value`). `approved` shows the answer as a head chip."""
    own = {
        "name": name, "title": title, "target": target, "targetKind": target_kind, "meta": meta,
        "badges": badges, "note": note, "exit": exit, "status": status, "age": age, "took": took,
        "intent": intent, "frame": frame, "collapsible": collapsible, "collapsed": collapsed,
        "preview": preview, "tools": tools, "confirm": confirm, "ask": ask, "approved": approved,
    }  # fmt: skip
    return make("tool", own, children, common)


def trace(
    lines: Sequence[TraceLine],
    *children: Node | None,
    variant: TraceVariant | None = None,
    status: TraceStatus | None = None,
    title: Text | None = None,
    summary: Text | None = None,
    icon: IconName | None = None,
    age: float | None = None,
    took: float | None = None,
    open: bool | None = None,  # noqa: A002
    sources: Sequence[TraceSource] | None = None,
    **common: Unpack[CommonNoTitle],
) -> Node:
    """An agent's steps: thoughts, commands and sources, each opening in
    place. A child keyed by a line's `id` is that line's content. Flipping a
    line sends `toggle` with its id as `item` and `open`; the head, without
    `item`."""
    own = {
        "lines": lines, "variant": variant, "status": status, "title": title, "summary": summary,
        "icon": icon, "age": age, "took": took, "open": open, "sources": sources,
    }  # fmt: skip
    return make("trace", own, children, common)


def message(
    *children: Node | None,
    variant: MessageVariant | None = None,
    side: Literal["start", "end"] | None = None,
    author: str | None = None,
    time: str | None = None,
    avatar: Node | str | None = None,
    continued: bool | None = None,
    reactions: Sequence[Reaction] | None = None,
    actions: Sequence[IconAction] | None = None,
    reveal: Literal["hover", "always"] | None = None,
    status: str | None = None,
    pending: bool | None = None,
    **common: Unpack[CommonNoActions],
) -> Node:
    """One chat message: author, time, the body (the children), reaction
    chips and hover actions. `side="end"` is the local user. `avatar` given
    as a node becomes a child keyed `avatar`, drawn in the avatar slot.
    Flipping a reaction sends `change` with its text as `value` and `on`."""
    own: dict[str, Any] = {
        "variant": variant, "side": side, "author": author, "time": time, "avatar": avatar,
        "continued": continued, "reactions": reactions, "actions": actions, "reveal": reveal,
        "status": status, "pending": pending,
    }  # fmt: skip
    return _slotted("message", own, children, common, ("avatar",))


def stacktrace(
    frames: Sequence[StackFrame],
    *,
    error: StackError | None = None,
    selected: int | None = None,
    lang: str | None = None,
    variant: StacktraceVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """An error and its stack frames, innermost first. A click on a frame
    sends `select`, a double click `activate`, with the frame's index as
    `item`."""
    own = {"frames": frames, "error": error, "selected": selected, "lang": lang, "variant": variant}
    return make("stacktrace", own, (), common)


def suggestions(
    items: Sequence[Suggestion],
    *,
    variant: SuggestionsVariant | None = None,
    **common: Unpack[Common],
) -> Node:
    """Suggested prompts; a click sends `activate` with the item's `id` (else
    its `text`) as `item`."""
    return make("suggestions", {"items": items, "variant": variant}, (), common)


def questions(
    items: Sequence[Question],
    *children: Node | None,
    step: int | None = None,
    answers: Mapping[str, str | Sequence[str]] | None = None,
    variant: QuestionsVariant | None = None,
    hint: str | None = None,
    submit: str | None = None,
    **common: Unpack[Common],
) -> Node:
    """A questionnaire: questions with choices, answered in steps. A pick
    sends `change` (the question as `item`), Next/Back/Skip `action` with the
    question left as `item`, Submit `action` `submit` with every answer as
    `value`. An `other` question's free text is the `input` child keyed
    `<id>.other`."""
    own = {"items": items, "step": step, "answers": answers, "variant": variant, "hint": hint, "submit": submit}
    return make("questions", own, children, common)


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
