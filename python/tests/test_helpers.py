"""`show`, `ask` and `plain`."""

from __future__ import annotations

import pytest
from conftest import Term

import tern_sdk
from tern_sdk import ui
from tern_sdk.session import Session


def form() -> ui.Node:
    return ui.html.form(
        ui.html.p("Which size?"),
        ui.html.label(ui.html.input(type="radio", name="size", value="s"), "Small", key="s"),
        ui.html.label(ui.html.input(type="radio", name="size", value="l", checked=True), "Large", key="l"),
        ui.html.button("Create", key="go", actions={"click": "submit"}),
        key="ask",
    )


def test_plain_layout_kinds() -> None:
    view = ui.col(
        ui.card(ui.md("body\nmore"), ui.progress(0.25, "uploading"), head="Deploy", status="running"),
        ui.row(ui.badge("v2"), ui.text([ui.span("ok", "success")]), ui.kbd("cmd", "k")),
        ui.kv([("service", "api"), ("region", "eu-west-1")]),
        ui.rule("log"),
        ui.ansi("\x1b[31mred\x1b[0m"),
        ui.list(ui.item("one", value="1"), ui.item("two")),
        ui.meter(0.5, label="50%", total="200K"),
        ui.spinner("Waiting"),
    )
    assert tern_sdk.plain(view, cols=20).split("\n") == [
        "Deploy (running)",
        "  body",
        "  more",
        "  [###-------] 25% uploading",
        "[v2] ok cmd+k",
        "service  api",
        "region   eu-west-1",
        "------- log -------",
        "red",
        "- one  1",
        "- two",
        "[#####-----] 50% 50% 200K",
        "Waiting",
    ]


def test_plain_tables_align_columns() -> None:
    cols: list[ui.TableCol] = [{"id": "n", "head": "Name"}, {"id": "s", "head": "Size", "align": "end"}]
    rows: list[ui.TableRow] = [
        {"id": "a", "cells": {"n": "Cargo.toml", "s": "2.1K"}},
        {"id": "b", "cells": {"n": "x", "s": "14K"}},
    ]
    assert tern_sdk.plain(ui.table(cols, rows)).split("\n") == [
        "Name        Size",
        "Cargo.toml  2.1K",
        "x            14K",
    ]
    el = ui.html.table(
        ui.html.tr(ui.html.td("Cargo.toml"), ui.html.td("2.1K")),
        ui.html.tr(ui.html.td("README.md"), ui.html.td("14K")),
    )
    assert tern_sdk.plain(el).split("\n") == ["Cargo.toml  2.1K", "README.md   14K"]


def test_plain_el_blocks_and_controls() -> None:
    assert tern_sdk.plain(form()).split("\n") == ["Which size?", "( ) Small (*) Large [ Create ]"]
    lst = ui.html.ul(ui.html.li("a"), ui.html.li("b"))
    assert tern_sdk.plain(lst) == "- a\n- b"


def test_show_without_tsp_prints_plain_or_fallback(capsys: pytest.CaptureFixture[str]) -> None:
    tern_sdk.show(ui.text("hello"))
    tern_sdk.show(ui.text("hello"), fallback="custom")
    assert capsys.readouterr().out == "hello\ncustom\n"


def test_ask_without_tsp_is_unsupported() -> None:
    with pytest.raises(tern_sdk.Unsupported):
        tern_sdk.ask(form())


def test_show_sends_one_static_flow_surface(connected: Session, term: Term) -> None:
    tern_sdk.show(ui.text("hi"), css=".x{}", session=connected)
    sent = term.messages()
    assert [(m.verb, m.body) for m in sent] == [
        ("o", {"id": "s1", "mode": "flow", "listen": False}),
        ("s", {"sf": "s1", "name": "main", "css": ".x{}"}),
        (
            "f",
            {
                "sf": "s1",
                "s": 1,
                "ops": [
                    [
                        "add",
                        "main",
                        "s1",
                        None,
                        {"id": "main", "k": "col", "c": [{"id": "main.0", "k": "text", "p": {"text": "hi"}}]},
                    ]
                ],
            },
        ),
        ("x", {"id": "s1", "keep": True}),
    ]
    assert not connected.closed


def test_ask_returns_the_submitting_action(connected: Session, term: Term) -> None:
    changes: list[object] = []
    view = [form(), ui.html.input(type="checkbox", key="cb", on_change=changes.append)]
    term.event(ev="change", sf="s1", id="main.cb", value="on", checked=True)
    term.event(ev="action", sf="s1", id="main.0.go", act="cancel")
    term.event(ev="action", sf="s1", id="main.0.go", act="submit", values={"size": "l"})
    answer = tern_sdk.ask(view, session=connected)
    assert answer == tern_sdk.Answer("main.0.go", "submit", {"size": "l"})
    assert len(changes) == 1
    assert [m.verb for m in term.messages()][-1] == "x"


@pytest.mark.parametrize("key", [b"\x1b", b"\x03", b"\x04"])
def test_ask_is_dismissed_by_escape_and_ctrl_c_or_d(connected: Session, term: Term, key: bytes) -> None:
    term.send(key)
    assert tern_sdk.ask(form(), session=connected) is None
    assert term.messages()[-1].body == {"id": "s1", "keep": True}
