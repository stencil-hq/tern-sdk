"""The session against a scripted terminal: handshake, flow control, events,
routing and the close sequence."""

from __future__ import annotations

import json
from pathlib import Path

import pytest
from conftest import Term

from tern_sdk import session as session_mod
from tern_sdk import ui
from tern_sdk import wire
from tern_sdk.keys import Key
from tern_sdk.reconcile import ReconcileError
from tern_sdk.session import Session
from tern_sdk.wire import ActionEvent, GoneEvent, ResizeEvent, ThemeEvent


def frames(term: Term) -> list[dict[str, object]]:
    return [m.body for m in term.messages() if m.verb == "f"]


def test_handshake_writes_hello_then_da1_and_enables_modes(term: Term) -> None:
    term.reply(credits=3, apc=1000, cols=99, dark=False)
    s = session_mod.start(term.in_r, term.out_w, app="demo", version="1.2", features=["send"], timeout=1.0)
    assert s is not None
    out = term.read()
    hello = b'\x1b_tsp;q;{"q":"hello","v":[1],"app":"demo","ver":"1.2","features":["send"]}\x1b\\'
    assert out == hello + b"\x1b[c" + b"\x1b[?2004h\x1b[>1u"
    assert (s.caps.credits, s.caps.apc, s.caps.cols, s.caps.dark) == (3, 1000, 99, False)
    assert s.caps.has("flow") and s.caps.draws("el")
    s.close()


def test_handshake_without_modes(term: Term) -> None:
    term.reply()
    s = session_mod.start(term.in_r, term.out_w, app="demo", paste=False, kitty=False)
    assert s is not None
    assert term.read().endswith(b"\x1b[c")
    s.close()
    assert b"\x1b[<u" not in term.read()


def test_da1_first_means_no_tsp(term: Term) -> None:
    term.da1()
    term.reply()
    assert session_mod.start(term.in_r, term.out_w, app="demo", timeout=1.0) is None


def test_no_answer_times_out(term: Term) -> None:
    assert session_mod.start(term.in_r, term.out_w, app="demo", timeout=0.05) is None


def test_input_before_the_reply_is_kept(term: Term) -> None:
    term.send(b"x")
    term.reply()
    s = session_mod.start(term.in_r, term.out_w, app="demo")
    assert s is not None
    assert s.poll(0) == Key("x", "x")
    s.close()


def test_connect_reports_unavailable(monkeypatch: pytest.MonkeyPatch) -> None:
    monkeypatch.setenv("TERN_TSP", "0")
    assert session_mod.connect(app="demo") is None
    monkeypatch.delenv("TERN_TSP")
    monkeypatch.setenv("TMUX", "/tmp/tmux-1/default,1,0")
    assert session_mod.connect(app="demo") is None
    monkeypatch.delenv("TMUX")
    # pytest's stdin is not a tty.
    assert session_mod.connect(app="demo") is None


def test_surface_ids_count_up_and_open_says_mode(connected: Session, term: Term) -> None:
    a = connected.open(mode="flow", title="T", role="r.x")
    b = connected.open(listen=False)
    assert (a.id, b.id) == ("s1", "s2")
    opens = [m.body for m in term.messages()]
    assert opens == [
        {"id": "s1", "mode": "flow", "title": "T", "role": "r.x"},
        {"id": "s2", "mode": "inline", "listen": False},
    ]


def test_credits_block_and_one_frame_carries_the_difference(connected: Session, term: Term) -> None:
    surface = connected.open(mode="flow")
    term.read()
    for n in range(4):
        surface.render([ui.text(f"line {n}", key="t")])
    sent = frames(term)
    assert [f["s"] for f in sent] == [1, 2]
    assert surface.blocked
    surface.focus("main.t")
    assert frames(term) == []
    term.event(ev="ack", sf="s1", s=1)
    assert connected.poll(0.05) is None
    third = frames(term)
    assert third == [
        {
            "sf": "s1",
            "s": 3,
            "ops": [["text", "main.t", "replace", "line 3"], ["focus", "main.t"]],
        }
    ]


def test_an_ack_covers_every_frame_up_to_it(connected: Session, term: Term) -> None:
    surface = connected.open()
    surface.render(ui.text("a"))
    surface.render(ui.text("b"))
    term.read()
    term.event(ev="ack", sf="s1", s=2)
    connected.poll(0.05)
    assert not surface.blocked
    surface.render(ui.text("c"))
    surface.render(ui.text("d"))
    assert [f["s"] for f in frames(term)] == [3, 4]


def test_a_frame_without_ops_is_never_sent(connected: Session, term: Term) -> None:
    surface = connected.open()
    surface.render(ui.text("same"))
    surface.render(ui.text("same"))
    surface.send([])
    assert [f["s"] for f in frames(term)] == [1]


def test_listen_false_sends_without_credits(connected: Session, term: Term) -> None:
    surface = connected.open(mode="flow", listen=False)
    for n in range(5):
        surface.render(ui.text(str(n)))
    assert [f["s"] for f in frames(term)] == [1, 2, 3, 4, 5]


def test_close_flushes_pending_view_then_closes(connected: Session, term: Term) -> None:
    surface = connected.open(mode="flow")
    for n in range(3):
        surface.render(ui.text(str(n)))
    term.read()
    surface.close()
    sent = term.messages()
    assert [(m.verb, m.body.get("s") if m.verb == "f" else m.body) for m in sent] == [
        ("f", 3),
        ("x", {"id": "s1", "keep": True}),
    ]
    surface.render(ui.text("after"))
    assert term.read() == b""


def test_a_view_with_duplicate_ids_is_rejected_and_sends_nothing(connected: Session, term: Term) -> None:
    surface = connected.open()
    term.read()
    with pytest.raises(ReconcileError):
        surface.render(ui.col(ui.text("a", key="x"), ui.text("b", key="x")))
    assert term.read() == b""


def test_handlers_consume_their_events(connected: Session, term: Term) -> None:
    clicked: list[ActionEvent] = []
    picked: list[str | None] = []
    menu: list[str] = []
    surface = connected.open()
    surface.render(
        [
            ui.html.button("Go", key="go", on_click=clicked.append, on_menu={"rerun": lambda e: menu.append(e.act)}),
            ui.list(ui.item("one", key="a"), key="l", on_select=lambda e: picked.append(e.item)),
        ]
    )
    term.event(ev="action", sf="s1", id="main.go", act="click")
    term.event(ev="action", sf="s1", id="main.go", act="rerun")
    term.event(ev="select", sf="s1", id="main.l", item="main.l.a")
    term.event(ev="action", sf="s1", id="main.go", act="other")
    item = connected.poll(0.2)
    assert isinstance(item, ActionEvent) and item.act == "other"
    assert [e.id for e in clicked] == ["main.go"]
    assert menu == ["rerun"] and picked == ["main.l.a"]


def test_events_for_other_surfaces_or_unknown_ids_reach_the_loop(connected: Session, term: Term) -> None:
    surface = connected.open()
    surface.render(ui.html.button("Go", key="go", on_click=lambda e: None))
    term.event(ev="action", sf="s9", id="main", act="click")
    item = connected.poll(0.2)
    assert isinstance(item, ActionEvent) and item.sf == "s9"


def test_capability_events_update_and_still_arrive(connected: Session, term: Term) -> None:
    term.event(ev="resize", sf="s1", cols=80, cell={"w": 9, "h": 18}, visible=True)
    term.event(ev="theme", dark=False)
    term.event(ev="motion", reduce=True)
    items = [connected.poll(0.2) for _ in range(3)]
    assert isinstance(items[0], ResizeEvent) and isinstance(items[1], ThemeEvent)
    assert (connected.caps.cols, connected.caps.dark, connected.caps.reduce_motion) == (80, False, True)
    assert connected.caps.cell is not None and connected.caps.cell.w == 9


def test_hour12_follows_the_hello_and_defaults_to_24_hour(term: Term) -> None:
    term.reply(hour12=True)
    s = session_mod.start(term.in_r, term.out_w, timeout=1.0)
    assert s is not None and s.caps.hour12 is True
    s.close()
    reply = wire.decode_reply({"r": "hello"})
    assert isinstance(reply, wire.HelloReply)
    assert session_mod.Capabilities.from_reply(reply).hour12 is False


def test_gone_closes_the_surface(connected: Session, term: Term) -> None:
    surface = connected.open()
    term.read()
    term.event(ev="gone", sf="s1", ids=["s1"])
    assert isinstance(connected.poll(0.2), GoneEvent)
    assert surface.closed
    surface.render(ui.text("x"))
    assert term.read() == b""


def test_keys_and_a_lone_escape_after_the_flush_delay(connected: Session, term: Term) -> None:
    term.send(b"a\x1b[A")
    assert [connected.poll(0.2), connected.poll(0.2)] == [Key("a", "a"), Key("up")]
    term.send(b"\x1b")
    assert connected.poll(0.5) == Key("escape")
    term.send(b"\x03")
    assert list(connected.input(timeout=0.1)) == [Key("c", ctrl=True)]


def test_end_of_input_ends_the_loop(connected: Session, term: Term) -> None:
    import os

    os.close(term.in_w)
    assert connected.poll(1.0) is None
    assert list(connected.input()) == []


def test_close_sequence(connected: Session, term: Term) -> None:
    connected.open(mode="flow")
    kept = connected.open(keep=False)
    kept.render(ui.text("x"))
    term.read()
    term.event(ev="ack", sf="s2", s=1)
    connected.close()
    out = term.read()
    sent = term.messages(out)
    assert [(m.verb, m.body) for m in sent] == [
        ("x", {"id": "s1", "keep": True}),
        ("x", {"id": "s2", "keep": False}),
    ]
    assert out.endswith(b"\x1b[<u\x1b[?2004l")
    assert connected.closed
    assert connected.poll(0) is None
    connected.close()
    assert term.read() == b""


def test_blobs_are_sent_once_and_queried(connected: Session, term: Term) -> None:
    first = connected.blob(b"hello", "text/plain")
    assert connected.blob(b"hello", "text/plain") == first
    sent = term.messages()
    assert [(m.verb, m.params, m.body) for m in sent] == [("b", {"mime": "text/plain"}, "aGVsbG8=")]
    term.send(b'\x1b_tsp;r;{"r":"blobs","have":["' + first.encode() + b'"]}\x1b\\')
    assert connected.blobs([first, "00"]) == [first]
    assert term.messages()[0].body == {"q": "blobs", "ids": [first, "00"]}


def test_large_frames_are_chunked_at_the_negotiated_limit(term: Term) -> None:
    term.reply(apc=64)
    s = session_mod.start(term.in_r, term.out_w, app="demo")
    assert s is not None
    term.read()
    surface = s.open(listen=False)
    surface.render(ui.md("x" * 500))
    raw = term.read()
    assert b";c=1;m=1;" in raw
    assert all(len(piece) <= 64 + 40 for piece in raw.split(b"\x1b\\"))
    (frame,) = [m for m in term.messages(raw) if m.verb == "f"]
    assert frame.body["ops"][0][4]["c"][0]["p"]["text"] == "x" * 500
    s.close()


def test_recording(term: Term, tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    path = tmp_path / "rec.jsonl"
    monkeypatch.setenv("TERN_TSP_RECORD", str(path))
    term.reply()
    s = session_mod.start(term.in_r, term.out_w, app="demo")
    assert s is not None
    s.open(mode="flow").render(ui.text("hi"))
    term.event(ev="ack", sf="s1", s=1)
    s.poll(0.05)
    s.close()
    lines = [json.loads(line) for line in path.read_text().splitlines()]
    assert [(r["dir"], r["verb"]) for r in lines] == [
        ("out", "q"), ("in", "r"), ("out", "o"), ("out", "f"), ("in", "e"), ("out", "x"),
    ]  # fmt: skip
    assert lines[3]["body"]["ops"][0][0] == "add" and isinstance(lines[0]["t"], int)


def test_render_keyword_regions(connected: Session, term: Term) -> None:
    surface = connected.open()
    surface.render(main=[ui.text("a")], dock=ui.col(ui.editor(placeholder="Ask", key="ed")))
    (frame,) = frames(term)
    assert [op[1] for op in frame["ops"]] == ["main", "dock"]
    assert frame["ops"][1][4]["c"][0]["id"] == "dock.ed"
    with pytest.raises(TypeError):
        surface.render({"main": ui.text("a")}, dock=ui.text("b"))
