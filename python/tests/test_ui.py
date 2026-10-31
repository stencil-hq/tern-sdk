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


def test_code_bars_folds_and_overview() -> None:
    n = ui.code(
        "a\nb\nc",
        nums=[1, 2, 5],
        marks=[{"line": 1, "wash": (0.9, 0.3)}],
        folds=[{"from": 2, "to": 3, "note": "cut", "tone": "error", "hovercard": ui.text("why")}],
        overview=True,
    )
    assert n.props == {
        "text": "a\nb\nc", "nums": [1, 2, 5], "marks": [{"line": 1, "wash": [0.9, 0.3]}],
        "folds": [{"from": 2, "to": 3, "note": "cut", "tone": "error", "hovercard": {"k": "text", "p": {"text": "why"}}}],
        "overview": True,
    }  # fmt: skip


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


def test_json_value_and_open() -> None:
    n = ui.json({"a": [1, None]}, open=1)
    assert n.to_json() == {"k": "json", "p": {"value": {"a": [1, None]}, "open": 1}}


def test_input_kinds_map_their_props() -> None:
    options: list[str | ui.SelectOption] = ["opus", {"value": "haiku", "label": "Haiku 4.5", "group": "Fast"}]
    sel = ui.select(options, value="opus", open=True)
    assert sel.to_json() == {"k": "select", "p": {"options": options, "value": "opus", "open": True}}
    switch = ui.switch("Wrap", on=True, labels=("Off", "On"))
    assert switch.props == {"label": "Wrap", "on": True, "labels": ["Off", "On"]}
    slider = ui.slider((200, 1200), min=0, max=2000, unit="ms")
    assert slider.props == {"value": [200, 1200], "min": 0, "max": 2000, "unit": "ms"}
    field = ui.field(ui.input("x"), label="Name", error=True, label_width="14ch", variant="grid")
    assert field.to_json() == {
        "k": "field",
        "p": {"label": "Name", "error": True, "labelWidth": "14ch", "variant": "grid"},
        "c": [{"k": "input", "p": {"text": "x"}}],
    }
    otp = ui.otp("12", length=4, cursor=False, mask="*")
    assert otp.props == {"text": "12", "length": 4, "cursor": False, "mask": "*"}
    group = ui.buttons(ui.button("Undo"), ui.button("Redo"), variant="joined", size="sm")
    assert group.props == {"variant": "joined", "size": "sm"} and len(group.children) == 2
    marks: list[str | ui.CalendarMark] = ["2026-10-02", {"date": "2026-10-03", "tone": "error"}]
    cal = ui.calendar(("2026-10-01", None), range=True, week_start="sun", marks=marks)
    assert cal.props == {"value": ["2026-10-01", None], "range": True, "weekStart": "sun", "marks": marks}
    date = ui.date("2026-10-30", open=True, min="2026-01-01")
    assert date.props == {"value": "2026-10-30", "open": True, "min": "2026-01-01"}


def test_input_and_editor_addons() -> None:
    buttons: list[ui.AddonButton] = [{"act": "copy", "icon": "copy"}]
    n = ui.input("hunter2", secret=True, reveal=False, prefix="https://", keys="/", buttons=buttons)
    assert n.props == {
        "text": "hunter2", "secret": True, "reveal": False, "prefix": "https://", "keys": "/", "buttons": buttons,
    }  # fmt: skip
    send: list[ui.AddonButton] = [{"act": "send", "text": "Send", "variant": "primary"}]
    assert ui.editor("a", variant="field", buttons=send).props == {"text": "a", "variant": "field", "buttons": send}


def test_layout_and_data_kinds() -> None:
    split = ui.split(ui.text("a"), ui.text("b"), dir="col", sizes=[0.3, 0.7], min=["20ch", None], key="panes")
    assert split.props == {"dir": "col", "sizes": [0.3, 0.7], "min": ["20ch", None], "key": "panes"}
    scroll = ui.scroll(ui.text("log"), stick="end", noun=("entry", "entries"))
    assert scroll.props == {"stick": "end", "noun": ["entry", "entries"]}
    carousel = ui.carousel(ui.image("b1"), index=1, loop=True, h="10lines")
    assert carousel.props == {"index": 1, "loop": True, "h": "10lines"}
    empty = ui.empty(ui.button("Retry"), title="No runs", text=[ui.span("Start one")], icon="", tone="error")
    assert empty.props == {"title": "No runs", "text": [{"t": "Start one"}], "icon": "", "tone": "error"}
    assert ui.skeleton("rows", rows=6, variant="pulse").props == {"shape": "rows", "rows": 6, "variant": "pulse"}
    avatar = ui.avatar("Ann Lee", seed="ann@x.io", size="sm", status="success")
    assert avatar.props == {"text": "Ann Lee", "seed": "ann@x.io", "size": "sm", "status": "success"}
    people: list[ui.AvatarItem] = [{"text": "Ann"}, {"icon": "provider:anthropic"}]
    assert ui.avatars(people, max=3, label="reviewers").props == {"items": people, "max": 3, "label": "reviewers"}
    att = ui.attachment("plan.pdf", state="uploading", progress=0.5, removable=True, actions=[{"act": "retry"}])
    assert att.props == {
        "name": "plan.pdf", "state": "uploading", "progress": 0.5, "removable": True, "actions": [{"act": "retry"}],
    }  # fmt: skip


def test_navigation_kinds() -> None:
    trail: list[ui.Crumb] = [{"id": "root", "label": "repo"}, {"id": 2, "label": "src"}]
    crumbs = ui.breadcrumb(trail, variant="path", max=3)
    assert crumbs.props == {"items": trail, "variant": "path", "max": 3}
    pager = ui.pager(2, total=312, size=12, label="rows")
    assert pager.props == {"page": 2, "total": 312, "size": 12, "label": "rows"}
    links: list[ui.NavItem] = [{"id": "docs", "items": [{"id": "api", "detail": 3}]}]
    assert ui.nav(links, active="api", open="docs").props == {"items": links, "active": "api", "open": "docs"}
    items: list[ui.MenuEntry] = [{"header": "Release"}, {"id": "follow", "checked": True}, "-", {"id": "drop"}]
    menu = ui.menu(items, text="Release", caret=True, align="end", on_action={"follow": noop})
    assert menu.props == {"items": items, "text": "Release", "caret": True, "align": "end"}
    assert route(menu.handlers, decode_event({"ev": "action", "act": "follow", "checked": False})) is noop
    menus: list[ui.MenubarMenu] = [{"id": "file", "label": "File", "items": ["open", {"id": "quit", "keys": "cmd+q"}]}]
    assert ui.menubar(menus, open="file").props == {"menus": menus, "open": "file"}


def test_rich_menu_entries_keep_handlers_once() -> None:
    entries: list[ui.MenuEntry] = [{"id": "open", "icon": "file", "keys": "enter"}, "-"]
    n = ui.text("deploy.log", actions={"menu": entries}, on_menu={"open": noop, "trash": noop})
    assert n.props["actions"] == {"menu": [*entries, "trash"]}
    assert route(n.handlers, decode_event({"ev": "action", "act": "trash"})) is noop


def test_work_kinds_and_slots() -> None:
    reactions: list[ui.Reaction] = [{"text": "👍", "count": 2}]
    msg = ui.message(ui.md("hi"), author="Ann", side="end", avatar=ui.avatar("Ann"), reactions=reactions)
    assert msg.to_json() == {
        "k": "message",
        "p": {"author": "Ann", "side": "end", "reactions": reactions, "avatar": "avatar"},
        "c": [{"k": "md", "p": {"text": "hi"}}, {"k": "avatar", "p": {"text": "Ann", "key": "avatar"}}],
    }
    nodes = View.build({"main": [ui.message(ui.md("hi"), avatar=ui.avatar("Ann"), key="m")]}).nodes()
    assert nodes["main.m"].props["avatar"] == "main.m.avatar"
    lines: list[ui.TraceLine] = [{"id": "s1", "title": "Searched", "status": "done"}]
    trace = ui.trace(lines, ui.text("3 hits", key="s1"), status="running")
    assert trace.props == {"lines": lines, "status": "running"} and len(trace.children) == 1
    frames: list[ui.StackFrame] = [{"fn": "main", "file": "a.py", "line": 3}]
    stack = ui.stacktrace(frames, error={"type": "KeyError", "message": "'x'"}, selected=0)
    assert stack.props == {"frames": frames, "error": {"type": "KeyError", "message": "'x'"}, "selected": 0}
    hints = ui.suggestions([{"text": "Fix the tests"}], variant="rows")
    assert hints.props == {"items": [{"text": "Fix the tests"}], "variant": "rows"}
    asked: list[ui.Question] = [{"id": "size", "prompt": "Which size?", "choices": ["s", {"value": "m"}]}]
    q = ui.questions(asked, ui.input("", key="size.other"), step=0, answers={"size": "m"})
    assert q.props == {"items": asked, "step": 0, "answers": {"size": "m"}} and len(q.children) == 1


def test_extended_kinds_take_their_new_props() -> None:
    rule = ui.rule("Checkpoint", variant="pin", time="14:02", end=ui.button("Restore"))
    assert rule.to_json() == {
        "k": "rule",
        "p": {"label": "Checkpoint", "variant": "pin", "time": "14:02", "end": "end"},
        "c": [{"k": "button", "p": {"text": "Restore", "key": "end"}}],
    }
    cited = ui.text([ui.span("CI fails", cite=[{"href": "https://a", "n": 2}])], cite="note", sources=True)
    spans = [{"t": "CI fails", "cite": [{"href": "https://a", "n": 2}]}]
    assert cited.props == {"spans": spans, "cite": "note", "sources": True}
    series: list[ui.ChartSeries] = [{"label": "p50", "values": [1, None, {"value": 3}]}]
    chart = ui.chart("line", series=series, labels=["a", "b", "c"], unit="ms", last=False)
    assert chart.props == {"kind": "line", "series": series, "labels": ["a", "b", "c"], "unit": "ms", "last": False}
    donut = ui.chart("donut", series=[{"value": 3, "label": "ok", "token": "green"}], total="3", caption="runs")
    assert donut.props["series"] == [{"value": 3, "label": "ok", "token": "green"}] and donut.props["caption"] == "runs"
    table = ui.table(
        [{"id": "age", "sortable": True}], [], sort={"col": "age", "dir": "asc"}, multi=True, picked=["r1"],
        variant="bar", bar=[{"act": "restart", "text": "Restart"}], hint="r restart",
    )  # fmt: skip
    assert table.props == {
        "cols": [{"id": "age", "sortable": True}], "rows": [], "sort": {"col": "age", "dir": "asc"}, "multi": True,
        "picked": ["r1"], "variant": "bar", "bar": [{"act": "restart", "text": "Restart"}], "hint": "r restart",
    }  # fmt: skip
    sort = decode_event({"ev": "action", "act": "sort", "value": {"col": "age", "dir": "desc"}})
    assert route(ui.table([], [], on_action={"sort": noop}).handlers, sort) is noop
    sheet = ui.overlay(ui.text("x"), side="right", variant="inset", size=0.4)
    assert sheet.props == {"side": "right", "variant": "inset", "size": 0.4}
    ask: ui.ToolAsk = {"scopes": [{"id": "once"}], "scope": "once", "always": False}
    assert ui.tool(status="ask", confirm="scopes", ask=ask).props == {"status": "ask", "confirm": "scopes", "ask": ask}
    assert ui.tool(status="done", approved="session").props == {"status": "done", "approved": "session"}


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
