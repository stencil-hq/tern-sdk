"""Builders whose names shadow Python builtins (`list`, `input`), kept apart so
no other module's annotations see the shadowing names."""

from __future__ import annotations

from collections.abc import Sequence
from typing import Unpack

from ._node import Common, CommonNoMax, Node, make
from ._types import Decor, ListMax, Text


def list(  # noqa: A001
    *items: Node | None,
    selected: str | None = None,
    filter: str | None = None,  # noqa: A002
    empty: Text | None = None,
    max: ListMax | float | None = None,  # noqa: A002
    virtual: bool | None = None,
    **common: Unpack[CommonNoMax],
) -> Node:
    """A selectable list of `item` rows; `selected` is an item's node id."""
    own = {"selected": selected, "filter": filter, "empty": empty, "max": max, "virtual": virtual}
    return make("list", own, items, common)


def input(  # noqa: A001
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
    **common: Unpack[Common],
) -> Node:
    """A single-line text field; the program owns its text and caret."""
    own = {
        "text": text, "cursor": cursor, "anchor": anchor, "decor": decor, "ghost": ghost,
        "placeholder": placeholder, "prompt": prompt, "mode": mode, "lang": lang,
        "readonly": readonly, "sendable": sendable,
    }  # fmt: skip
    return make("input", own, (), common)
