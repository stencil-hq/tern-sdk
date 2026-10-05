"""Node builders: prop mapping, handlers and the escape hatches."""

from __future__ import annotations

import pytest

from tern_sdk import ui
from tern_sdk.reconcile import View
from tern_sdk.ui._node import route
from tern_sdk.wire import decode_event


def noop(_: object) -> None:
    return None


def test_click_handler_names_the_click_action_unless_the_node_does() -> None:
    assert ui.text("x", on_click=noop).props["actions"] == {"click": "click"}
    n = ui.text("x", actions={"click": "open", "menu": ["copy"]}, on_click=noop, on_dblclick=noop)
    assert n.props["actions"] == {"click": "open", "menu": ["copy"], "dblclick": "dblclick"}
    assert route(n.handlers, decode_event({"ev": "action", "act": "open"})) is noop
    assert route(n.handlers, decode_event({"ev": "action", "act": "click"})) is None


def test_menu_handlers_add_their_names_once() -> None:
    n = ui.card(actions={"menu": ["rerun"]}, on_menu={"rerun": noop, "sort=name": noop})
    assert n.props["actions"] == {"menu": ["rerun", "sort=name"]}
    sort = decode_event({"ev": "action", "act": "sort", "value": "name"})
    assert route(n.handlers, sort) is noop


def test_action_and_event_handlers_route_by_event() -> None:
    seen: list[str] = []
    n = ui.editor(on_action={"go": lambda e: seen.append("go")}, on_edit=lambda e: seen.append("edit"))
    assert "actions" not in n.props
    for raw in ({"ev": "action", "act": "go"}, {"ev": "edit", "id": "x"}, {"ev": "send", "id": "x"}):
        fn = route(n.handlers, decode_event(raw))
        if fn is not None:
            fn(None)
    assert seen == ["go", "edit"]


def test_snake_case_keywords_map_to_wire_names() -> None:
    assert ui.editor("a", max_lines=4).props == {"text": "a", "maxLines": 4}
    assert ui.tool(target_kind="path", title="Edit").props == {"title": "Edit", "targetKind": "path"}
    assert ui.picker(items_add=[{"id": "a"}], items_del=["b"]).props == {"itemsAdd": [{"id": "a"}], "itemsDel": ["b"]}


def test_text_as_string_or_spans() -> None:
    assert ui.text("plain").props == {"text": "plain"}
    spans = ui.text([ui.span("a", "muted"), "b", ui.span("c", fx="pulse", href="https://x")])
    assert spans.props == {"spans": [{"t": "a", "s": "muted"}, "b", {"t": "c", "fx": "pulse", "href": "https://x"}]}
    assert ui.seg([ui.span("main")], icon="branch").props == {"spans": [{"t": "main"}], "icon": "branch"}


def test_common_props_and_untyped_extras() -> None:
    n = ui.badge("v1", tone="accent", key=3, min={"w": "9ch"}, basis="content", hidden=False, props={"future": [1, 2]})
    assert n.props == {
        "text": "v1", "key": 3, "tone": "accent", "hidden": False, "basis": "content",
        "min": {"w": "9ch"}, "future": [1, 2],
    }  # fmt: skip


def test_none_props_and_children_are_dropped() -> None:
    n = ui.col(None, ui.rule(), gap=None)
    assert n.to_json() == {"k": "col", "c": [{"k": "rule"}]}


def test_el_builders() -> None:
    label = ui.html.label(ui.html.input(type="radio", name="size", value="s", checked=True), "Small", class_="opt")
    assert label.to_json() == {
        "k": "el",
        "p": {"tag": "label", "class": "opt"},
        "c": [
            {"k": "el", "p": {"tag": "input", "type": "radio", "name": "size", "value": "s", "checked": True}},
            {"k": "el", "p": {"tag": "span", "text": "Small"}},
        ],
    }
    assert ui.html.p("Which size?").props == {"tag": "p", "text": "Which size?"}
    assert ui.html.del_("old").props["tag"] == "del"
    assert ui.el("td", text="2K", attrs={"colspan": 2}).props == {"tag": "td", "attrs": {"colspan": 2}, "text": "2K"}


def test_tuples_become_lists_and_kv_pairs() -> None:
    n = ui.kv([("a", "1"), {"k": "b", "v": [ui.span("2", "num")]}], layout="inline")
    assert n.props == {"items": [{"k": "a", "v": "1"}, {"k": "b", "v": [{"t": "2", "s": "num"}]}], "layout": "inline"}
    assert ui.kbd("cmd", "k").props == {"keys": ["cmd", "k"]}


def test_untyped_node() -> None:
    n = ui.node("future-kind", {"x": 1}, [ui.text("a")], on_toggle=noop)
    assert n.to_json() == {"k": "future-kind", "p": {"x": 1}, "c": [{"k": "text", "p": {"text": "a"}}]}
    assert n.handlers == {"toggle": noop}


def test_non_json_props_are_refused() -> None:
    with pytest.raises(TypeError):
        ui.node("text", {"text": object()})


def test_handlers_follow_derived_ids() -> None:
    view = View.build({"main": [ui.text("a"), ui.html.button("b", key="go", on_click=noop)]})
    nodes = view.nodes()
    assert set(nodes) == {"main", "main.0", "main.go"}
    assert nodes["main.go"].handlers == {"action:click": noop}
