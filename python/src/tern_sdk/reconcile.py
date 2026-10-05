"""Views diffed into frame ops with derived ids, matching tern-sdk's
reconciler used by Tern's plugin worker and pinned by the shared corpus."""

from __future__ import annotations

from bisect import bisect_left
from collections.abc import Mapping, Sequence
from copy import deepcopy
from dataclasses import dataclass, field
from typing import Final, TypeAlias

from .ui import Handler, Node
from .wire import TEXT_KINDS, Json, Op

REGIONS: Final = ("main", "dock", "layer")
"""The regions of a surface, in the order they are added."""

Region: TypeAlias = Node | Sequence[Node] | Mapping[str, Json]
"""A region's root: a node, a list of nodes (wrapped in a `col`) or a wire node."""
ViewLike: TypeAlias = Mapping[str, Region | None] | None
"""A view `{main?, dock?, layer?}`; `None` is the empty view."""


AnyView: TypeAlias = ViewLike | Node | Sequence[Node]
"""A view, or a node or list of nodes shown as `main`'s children."""


def as_view(view: AnyView) -> ViewLike:
    """`view` as `{main?, dock?, layer?}`: a bare node or a list of nodes
    becomes `main`'s children (a region root's own kind and props are not
    drawn, so a bare node is never made the root)."""
    if isinstance(view, Node):
        return {"main": [view]}
    if isinstance(view, Sequence) and not isinstance(view, str):
        return {"main": list(view)}
    return view


class ReconcileError(ValueError):
    """A view that can't be reconciled: two siblings with one id, or a malformed node."""


def json_equal(a: Json, b: Json) -> bool:
    """Deep JSON equality as serde_json has it: `1`, `1.0` and `True` differ."""
    if isinstance(a, dict):
        if not isinstance(b, dict) or a.keys() != b.keys():
            return False
        return all(json_equal(v, b[k]) for k, v in a.items())
    if isinstance(a, (list, tuple)):
        if not isinstance(b, (list, tuple)) or len(a) != len(b):
            return False
        return all(json_equal(x, y) for x, y in zip(a, b, strict=True))
    if type(a) is not type(b):
        return False
    return bool(a == b)


@dataclass(slots=True)
class Tree:
    """One node of a view with its id assigned."""

    id: str
    kind: str
    props: dict[str, Json]
    children: list[Tree]
    handlers: dict[str, Handler] = field(default_factory=dict)

    @classmethod
    def build(cls, id: str, node: Node | Mapping[str, Json]) -> Tree:
        """The tree of `node` under id `id`; raises `ReconcileError` on a
        malformed node or two siblings with one id."""
        if isinstance(node, Node):
            kind: Json = node.kind
            props: Json = node.props
            nodes: Sequence[Node | Mapping[str, Json]] = node.children
            handlers = node.handlers
        elif isinstance(node, Mapping):
            kind = node.get("k")
            props = node.get("p")
            raw = node.get("c")
            if raw is None or (isinstance(raw, Mapping) and not raw):
                raw = []
            if not isinstance(raw, list):
                raise ReconcileError(f"{id}: `c` must be a list, got {raw!r}")
            nodes = raw
            handlers = {}
        else:
            raise ReconcileError(f"{id}: a node must be a table, got {node!r}")
        if not isinstance(kind, str):
            raise ReconcileError(f"{id}: a node needs a string `k`")
        if props is None:
            props = {}
        if not isinstance(props, Mapping):
            raise ReconcileError(f"{id}: `p` must be a table, got {props!r}")
        seen: set[str] = set()
        children: list[Tree] = []
        for i, child in enumerate(nodes):
            child_props = (
                child.props if isinstance(child, Node) else (child.get("p") if isinstance(child, Mapping) else None)
            )
            key = child_props.get("key") if isinstance(child_props, Mapping) else None
            if isinstance(key, str):
                name = key
            elif isinstance(key, (int, float)) and not isinstance(key, bool):
                name = str(key)
            else:
                name = str(i)
            child_id = f"{id}.{name}"
            if child_id in seen:
                raise ReconcileError(f"{id}: duplicate child key {name!r}")
            seen.add(child_id)
            children.append(cls.build(child_id, child))
        return cls(id, kind, deepcopy(dict(props)), children, dict(handlers))

    def wire(self) -> dict[str, Json]:
        """The node as a wire subtree with every id."""
        out: dict[str, Json] = {"id": self.id, "k": self.kind}
        if self.props:
            out["p"] = self.props
        if self.children:
            out["c"] = [c.wire() for c in self.children]
        return out

    def walk(self) -> list[Tree]:
        """This node and every descendant, in document order."""
        out = [self]
        for child in self.children:
            out.extend(child.walk())
        return out


def _region(name: str, value: Region) -> Tree:
    if isinstance(value, Node) or isinstance(value, Mapping):
        return Tree.build(name, value)
    return Tree.build(name, Node("col", {}, list(value)))


@dataclass(slots=True)
class View:
    """A surface's view: the tree of each region, as last sent."""

    main: Tree | None = None
    dock: Tree | None = None
    layer: Tree | None = None

    @classmethod
    def build(cls, view: ViewLike) -> View:
        """The view `{main?, dock?, layer?}`; raises `ReconcileError` when it
        has an unknown region, a malformed node or siblings sharing an id."""
        if view is None:
            return cls()
        extra = [k for k in view if k not in REGIONS]
        if extra:
            raise ReconcileError(f"view: unknown region {extra[0]!r} (main, dock or layer)")
        trees = {name: _region(name, v) for name in REGIONS if (v := view.get(name)) is not None}
        return cls(**trees)

    def regions(self) -> tuple[Tree | None, Tree | None, Tree | None]:
        """`main`, `dock` and `layer`, in that order."""
        return self.main, self.dock, self.layer

    def nodes(self) -> dict[str, Tree]:
        """Every node of the view by id."""
        return {t.id: t for region in self.regions() if region is not None for t in region.walk()}

    def ops(self, next: View, surface: str) -> list[Op]:
        """The ops turning this view into `next` in surface `surface`."""
        ops: list[Op] = []
        for old, new in zip(self.regions(), next.regions(), strict=True):
            if old is None and new is None:
                continue
            if new is None:
                assert old is not None
                ops.append(["del", old.id])
            elif old is None:
                ops.append(["add", new.id, surface, None, new.wire()])
            elif old.kind != new.kind:
                ops.append(["del", old.id])
                ops.append(["add", new.id, surface, None, new.wire()])
            else:
                _diff(old, new, ops)
        return ops


def reconcile(before: ViewLike, after: ViewLike, surface: str) -> list[Op]:
    """The ops turning view `before` (as already sent) into `after`."""
    return View.build(before).ops(View.build(after), surface)


def _diff(old: Tree, new: Tree, ops: list[Op]) -> None:
    """Ops turning `old` into `new`, which share id and kind."""
    text = new.kind in TEXT_KINDS
    changed: dict[str, Json] = {}
    for key, value in new.props.items():
        if text and key == "text":
            continue
        if key not in old.props or not json_equal(old.props[key], value):
            changed[key] = value
    for key in old.props:
        if not (text and key == "text") and key not in new.props:
            changed[key] = None
    if changed:
        ops.append(["set", new.id, changed])
    if text:
        before, after = old.props.get("text"), new.props.get("text")
        absent = "text" not in old.props, "text" not in new.props
        if absent[0] != absent[1] or not json_equal(before, after):
            if isinstance(after, str):
                if isinstance(before, str) and before and after.startswith(before):
                    ops.append(["text", new.id, "append", after[len(before) :]])
                else:
                    ops.append(["text", new.id, "replace", after])
            else:
                ops.append(["set", new.id, {"text": None}])
    _children(old, new, ops)


def _children(old: Tree, new: Tree, ops: list[Op]) -> None:
    """Deletions first, then from the last child back each new, re-kinded or
    out-of-order child is placed before its next sibling; kept children diff
    right after they are placed."""
    before = {c.id: (i, c) for i, c in enumerate(old.children)}
    kept = {c.id for c in new.children}
    for child in old.children:
        if child.id not in kept:
            ops.append(["del", child.id])
    same = [
        (at, before[c.id][0]) for at, c in enumerate(new.children) if c.id in before and before[c.id][1].kind == c.kind
    ]
    stay = {same[k][0] for k in _rising([i for _, i in same])}
    nxt: str | None = None
    for at in range(len(new.children) - 1, -1, -1):
        child = new.children[at]
        prior = before.get(child.id)
        if prior is None:
            ops.append(["add", child.id, new.id, nxt, child.wire()])
        elif prior[1].kind != child.kind:
            ops.append(["del", child.id])
            ops.append(["add", child.id, new.id, nxt, child.wire()])
        else:
            if at not in stay:
                ops.append(["move", child.id, new.id, nxt])
            _diff(prior[1], child, ops)
        nxt = child.id


def _rising(seq: list[int]) -> list[int]:
    """Indices into `seq` of one longest strictly rising subsequence."""
    tails: list[int] = []
    tail_values: list[int] = []
    prev: list[int | None] = [None] * len(seq)
    for i, v in enumerate(seq):
        at = bisect_left(tail_values, v)
        prev[i] = tails[at - 1] if at > 0 else None
        if at == len(tails):
            tails.append(i)
            tail_values.append(v)
        else:
            tails[at] = i
            tail_values[at] = v
    out: list[int] = []
    cur = tails[-1] if tails else None
    while cur is not None:
        out.append(cur)
        cur = prev[cur]
    out.reverse()
    return out
