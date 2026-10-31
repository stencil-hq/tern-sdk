"""Design kinds and props: slot hoisting, `font`, the kinds order and plain fallbacks."""

from __future__ import annotations

from conftest import Term

import tern_sdk
from tern_sdk import ui, wire
from tern_sdk.reconcile import View
from tern_sdk.session import Session


def test_kinds_keep_terns_order_with_stats_and_button() -> None:
    kinds = list(wire.KINDS)
    assert len(kinds) == 74
    assert kinds[kinds.index("kv") + 1 : kinds.index("kv") + 3] == ["stats", "table"]
    assert kinds[kinds.index("input") + 1 : kinds.index("input") + 3] == ["button", "status"]
    assert kinds[kinds.index("effort") + 1 : kinds.index("effort") + 3] == ["select", "switch"]
    assert kinds[-3:] == ["suggestions", "questions", "el"]
    assert "font" in wire.TERMINAL_FEATURES


def test_slot_nodes_become_children_named_by_derived_id() -> None:
    refresh = ui.button("Refresh", key="ignored")
    card = ui.card(ui.text("body"), head="Runs", end=refresh, key="runs")
    sheet = ui.overlay(ui.text("x"), foot=ui.row(ui.button("OK")), end=ui.badge("new"), key="o")
    nodes = View.build({"main": [card], "layer": [sheet]}).nodes()
    assert nodes["main.runs"].props["end"] == "main.runs.end"
    assert nodes["main.runs"].props["head"] == "Runs"
    assert nodes["main.runs.end"].kind == "button"
    assert nodes["layer.o"].props["foot"] == "layer.o.foot"
    assert nodes["layer.o"].props["end"] == "layer.o.end"
    assert ui.section(head=ui.text("h"), key="s").to_json()["p"]["head"] == "head"
    assert ui.card(end="main.x").props["end"] == "main.x"


def test_open_sends_font_only_when_given(connected: Session, term: Term) -> None:
    connected.open(mode="screen", font="sans")
    connected.open()
    assert [m.body.get("font") for m in term.messages() if m.verb == "o"] == ["sans", None]


def test_action_events_carry_the_row_item() -> None:
    ev = wire.decode_event({"ev": "action", "id": "main.t", "act": "approve", "item": "r1"})
    assert isinstance(ev, wire.ActionEvent) and ev.item == "r1"


def test_plain_covers_the_wave_two_kinds() -> None:
    cites = [
        ui.span("CI fails", cite={"href": "https://a"}),
        ui.span(" and "),
        ui.span("drifts", cite=[{"href": "https://b", "n": 1}, {"href": "https://a"}]),
    ]
    view = ui.col(
        ui.select(["opus", {"value": "haiku", "label": "Haiku 4.5"}], value="haiku", label="Model"),
        ui.switch("Wrap", on=True),
        ui.slider(72, label="Opacity", unit="%"),
        ui.slider((200, 1200), unit="ms"),
        ui.field(ui.input("ann"), label="Name", required=True, error="Taken", hint="Unique"),
        ui.otp("12", length=4, mask=True),
        ui.buttons(ui.button("Undo"), ui.button("Redo")),
        ui.avatars([{"text": "Ann"}, {"text": "Bo"}], label="reviewers"),
        ui.breadcrumb([{"id": "r", "label": "repo"}, {"id": "s", "label": "src"}]),
        ui.pager(2, total=30, size=12),
        ui.empty(ui.button("Retry"), title="No runs", text="Start one"),
        ui.skeleton(),
        ui.calendar(("2026-10-01", "2026-10-07"), range=True),
        ui.date(placeholder="Pick a date"),
        ui.menu([], text="Release"),
        ui.menubar([{"id": "file", "label": "File", "items": []}, {"id": "edit", "items": []}]),
        ui.nav([{"id": "a", "label": "Home"}, {"id": "b", "label": "Docs", "items": [{"id": "api"}]}], active="api"),
        ui.carousel(ui.text("one"), ui.text("two"), index=5),
        ui.message(
            ui.md("hi"),
            author="Ann",
            time="14:02",
            avatar=ui.avatar("Ann"),
            reactions=[{"text": "👍", "count": 2}, {"text": "🎉", "count": 0}],
            status="Read",
        ),
        ui.attachment("plan.pdf"),
        ui.trace(
            [{"id": "s1", "title": "Searched", "target": "tern", "status": "done"}, {"title": "Read"}],
            ui.text("3 hits", key="s1"),
            ui.text("tail"),
            title="Worked",
            status="done",
        ),
        ui.stacktrace(
            [{"fn": "main", "file": "a.py", "line": 3, "col": 5}, {"file": ""}],
            error={"type": "KeyError", "message": "'x'"},
        ),
        ui.suggestions([{"text": "Fix tests"}]),
        ui.questions([{"id": "size", "prompt": "Which size?", "choices": ["s", {"value": "m", "label": "Medium"}]}]),
        ui.text(cites),
        ui.input("hunter2", secret=True),
        ui.chart("line", series=[1, None, 3]),
        ui.chart("donut", series=[{"value": 3, "label": "ok"}, {"value": 1.0, "label": "err"}], unit="%"),
        ui.chart("area", series=[1, 2], summary="p50 12 ms"),
    )
    assert tern_sdk.plain(view).split("\n") == [
        "Model: Haiku 4.5",
        "[x] Wrap",
        "Opacity: 72%",
        "200–1200 ms",
        "Name *",
        "ann",
        "Taken",
        "••__",
        "[Undo] [Redo]",
        "Ann, Bo reviewers",
        "repo › src",
        "Page 2 of 3",
        "No runs",
        "Start one",
        "[Retry]",
        "2026-10-01 – 2026-10-07",
        "Pick a date",
        "[Release]",
        "File  edit",
        "Home | [Docs]",
        "two",
        "Ann 14:02",
        "  hi",
        "  👍 2  🎉",
        "  Read",
        "[attachment: plan.pdf]",
        "Worked (done)",
        "- Searched tern (done)",
        "    3 hits",
        "- Read",
        "tail",
        "KeyError: 'x'",
        "  at main (a.py:3:5)",
        "  at <anonymous>",
        "- Fix tests",
        "Which size?",
        "  - s",
        "  - Medium",
        "CI fails[2] and drifts[1][2]",
        "•••••••",
        "▁█",
        "ok 3%  err 1%",
        "p50 12 ms",
    ]
    marker = ui.rule("Checkpoint", variant="pin", time="14:02", end=ui.button("Restore"))
    assert tern_sdk.plain(marker, cols=24) == "-- Checkpoint · 14:02 --"


def test_plain_fallbacks_for_new_kinds_and_cells() -> None:
    view = ui.card(
        ui.stats([{"label": "Runs", "value": "12", "sub": "today"}, {"label": "Failed", "value": "1"}]),
        ui.table(
            [{"id": "n", "head": "Name"}, {"id": "s"}, {"id": "v"}, {"id": "a"}],
            [
                {
                    "id": "r1",
                    "cells": {
                        "n": {"text": "api", "dot": "success"},
                        "s": {"spark": {"values": [0, 1]}},
                        "v": {"badge": {"text": "v2"}},
                        "a": {"button": {"text": "Approve", "act": "approve"}},
                    },
                }
            ],
            selected="r1",
        ),
        ui.table([{"id": "n", "head": "Name"}], [], empty="No runs"),
        head="Jobs",
        meta="prod",
        end=ui.button("Refresh"),
    )
    assert tern_sdk.plain(view).split("\n") == [
        "Jobs  prod  [Refresh]",
        "  Runs: 12 (today)  ·  Failed: 1",
        "     Name",
        "  >  ● api  ▁█  [v2]  [Approve]",
        "  Name",
        "  No runs",
    ]
