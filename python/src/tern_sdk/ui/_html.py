"""`el` nodes: plain HTML elements of an allowed tag, one builder per tag."""

from __future__ import annotations

from collections.abc import Mapping
from typing import Unpack

from ._node import Common, Node, make
from ._types import InputType, Tag

AttrValue = str | int | float | bool
"""An `el` attribute value."""


def el(
    tag: Tag,
    *children: Node | str | None,
    class_: str | None = None,
    attrs: Mapping[str, AttrValue] | None = None,
    text: str | None = None,
    type: InputType | None = None,  # noqa: A002
    name: str | None = None,
    value: str | None = None,
    checked: bool | None = None,
    disabled: bool | None = None,
    **common: Unpack[Common],
) -> Node:
    """An `el` node of `tag`. A leading string child is its `text` (drawn
    before the children); later strings become `span` elements."""
    nodes: list[Node | None] = []
    for i, child in enumerate(children):
        if isinstance(child, str):
            if i == 0 and text is None:
                text = child
            else:
                nodes.append(make("el", {"tag": "span", "text": child}))
        else:
            nodes.append(child)
    own = {
        "tag": tag, "class": class_, "attrs": attrs, "text": text, "type": type, "name": name,
        "value": value, "checked": checked, "disabled": disabled,
    }  # fmt: skip
    return make("el", own, nodes, common)


class ElementBuilder:
    """Builds `el` nodes of one tag: `ui.html.div(child, class_="row")`."""

    def __init__(self, tag: Tag) -> None:
        self.tag: Tag = tag

    def __call__(
        self,
        *children: Node | str | None,
        class_: str | None = None,
        attrs: Mapping[str, AttrValue] | None = None,
        text: str | None = None,
        **common: Unpack[Common],
    ) -> Node:
        """An `el` of this tag; a leading string is its text."""
        return el(self.tag, *children, class_=class_, attrs=attrs, text=text, **common)


class InputBuilder:
    """Builds `el` checkbox and radio controls: `ui.html.input(type="radio", name="size", value="s")`."""

    def __call__(
        self,
        *,
        type: InputType,  # noqa: A002
        name: str | None = None,
        value: str | None = None,
        checked: bool | None = None,
        disabled: bool | None = None,
        class_: str | None = None,
        attrs: Mapping[str, AttrValue] | None = None,
        **common: Unpack[Common],
    ) -> Node:
        """A checkbox or radio `el` input."""
        return el(
            "input", type=type, name=name, value=value, checked=checked, disabled=disabled,
            class_=class_, attrs=attrs, **common,
        )  # fmt: skip


class Html:
    """One `el` builder per allowed tag (`del` is `del_`)."""

    div = ElementBuilder("div")
    span = ElementBuilder("span")
    p = ElementBuilder("p")
    section = ElementBuilder("section")
    header = ElementBuilder("header")
    footer = ElementBuilder("footer")
    nav = ElementBuilder("nav")
    aside = ElementBuilder("aside")
    main = ElementBuilder("main")
    article = ElementBuilder("article")
    figure = ElementBuilder("figure")
    blockquote = ElementBuilder("blockquote")
    ul = ElementBuilder("ul")
    ol = ElementBuilder("ol")
    li = ElementBuilder("li")
    dl = ElementBuilder("dl")
    dt = ElementBuilder("dt")
    dd = ElementBuilder("dd")
    h1 = ElementBuilder("h1")
    h2 = ElementBuilder("h2")
    h3 = ElementBuilder("h3")
    h4 = ElementBuilder("h4")
    pre = ElementBuilder("pre")
    code = ElementBuilder("code")
    kbd = ElementBuilder("kbd")
    strong = ElementBuilder("strong")
    b = ElementBuilder("b")
    em = ElementBuilder("em")
    i = ElementBuilder("i")
    del_ = ElementBuilder("del")
    mark = ElementBuilder("mark")
    hr = ElementBuilder("hr")
    table = ElementBuilder("table")
    thead = ElementBuilder("thead")
    tbody = ElementBuilder("tbody")
    tr = ElementBuilder("tr")
    th = ElementBuilder("th")
    td = ElementBuilder("td")
    label = ElementBuilder("label")
    button = ElementBuilder("button")
    form = ElementBuilder("form")
    input = InputBuilder()


html = Html()
"""The `el` builders by tag: `ui.html.button("Create", on_click=submit)`."""
