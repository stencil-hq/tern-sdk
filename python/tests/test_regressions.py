"""Contract regressions exercised through scripted input and failing I/O."""

from __future__ import annotations

import codecs
import ctypes
import importlib.util
import json
import signal
import sys
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import Term
from test_session import frames

import tern_sdk
from tern_sdk import session as session_mod
from tern_sdk import ui, wire
from tern_sdk.keys import Key, KeyDecoder
from tern_sdk.reconcile import View, as_view
from tern_sdk.session import Session


def test_nested_props_are_snapshotted(connected: Session, term: Term) -> None:
    surface = connected.open(listen=False)
    node = ui.text([ui.span("a")])
    surface.render(node)
    term.read()
    node.props["spans"][0]["t"] = "b"
    surface.render(node)
    assert frames(term)[0]["ops"] == [["set", "main.0", {"spans": [{"t": "b"}]}]]


def test_unknown_kitty_functional_codes_are_not_text() -> None:
    decoder = KeyDecoder()
    assert decoder.feed(b"\x1b[57364u\x1b[63743u") == []
    assert decoder.feed("😀界".encode()) == [Key("😀", "😀"), Key("界", "界")]


@pytest.mark.parametrize("suffix", [b"[1;5", b"O"])
def test_incomplete_key_sequences_drop_on_flush(suffix: bytes) -> None:
    decoder = KeyDecoder()
    assert decoder.feed(b"\x1b" + suffix) == []
    assert decoder.flush() == []
    assert decoder.feed(b"x") == [Key("x", "x")]


def test_typed_meter_cells_render_values_and_parts() -> None:
    cells: list[ui.MeterCell] = [{"meter": {"value": 0.5}}, {"meter": {"parts": [{"value": 0.2}, {"value": 0.3}]}}]
    node = ui.table([{"id": "m"}], [{"id": str(i), "cells": {"m": cell}} for i, cell in enumerate(cells)])
    assert node.props["rows"][0]["cells"]["m"] == {"meter": {"value": 0.5}}
    assert tern_sdk.plain(node).splitlines() == ["[#####-----] 50%", "[#####-----] 50%"]


def test_kind_props_replace_common_meanings() -> None:
    title = [ui.span("Title", "strong")]
    assert ui.tool(title=title).props["title"] == title
    actions: list[ui.PickerAction] = [{"id": "go", "label": "Go"}]
    picker = ui.picker(title=title, actions=actions)
    assert picker.props["title"] == title and picker.props["actions"] == actions
    assert ui.prefs(title="Settings").props["title"] == "Settings"
    assert ui.list(max=3).props["max"] == 3
    states: list[ui.AgentStatus] = ["pending", "running", "done", "failed", "aborted", "idle", "parked"]
    assert [ui.agent(status=s).props["status"] for s in states] == states


def test_adopt_reuses_last_sent_main_and_sequence(connected: Session, term: Term) -> None:
    old = connected.open("saved")
    old.render(main=[ui.text("before")], dock=[ui.editor("draft")], layer=[ui.text("popup")])
    old.close()
    term.read()
    new = connected.open("saved", adopt=True)
    new.render(main=[ui.text("after")], dock=[ui.editor("new draft")])
    frame = frames(term)[0]
    assert frame["s"] == 2
    assert frame["ops"][0] == ["text", "main.0", "replace", "after"]
    assert [op[0:2] for op in frame["ops"]] == [["text", "main.0"], ["add", "dock"]]


def test_unknown_adopt_deletes_main_once(connected: Session, term: Term) -> None:
    surface = connected.open("unknown", adopt=True, listen=False)
    surface.render(ui.text("first"))
    surface.render(ui.text("next"))
    first, second = frames(term)
    assert first["s"] == 1 and first["ops"][0] == ["del", "main"]
    assert first["ops"][1][0:2] == ["add", "main"]
    assert second["ops"] == [["text", "main.0", "replace", "next"]]


def test_failed_frame_preserves_view_sequence_and_queue(
    connected: Session, term: Term, monkeypatch: pytest.MonkeyPatch
) -> None:
    surface = connected.open()
    surface.render(ui.text("a"))
    surface.render(ui.text("b"))
    surface.render(ui.text("c"))
    surface.focus("main.0")
    term.read()
    write = connected._writer.write
    failed = False

    def fail_once(data: bytes) -> None:
        nonlocal failed
        if b"\x1b_tsp;f;" in data and not failed:
            failed = True
            raise OSError("frame failed")
        write(data)

    monkeypatch.setattr(connected._writer, "write", fail_once)
    term.event(ev="ack", sf=surface.id, s=1)
    with pytest.raises(OSError, match="frame failed"):
        connected.poll(0)
    assert surface._seq == 2 and surface._queued == [["focus", "main.0"]]
    assert surface._sent.main.children[0].props["text"] == "b"
    assert not surface.blocked
    surface.send([])
    assert frames(term) == [{"sf": "s1", "s": 3, "ops": [["text", "main.0", "replace", "c"], ["focus", "main.0"]]}]


def test_direct_failed_frame_can_retry(connected: Session, term: Term, monkeypatch: pytest.MonkeyPatch) -> None:
    surface = connected.open()
    term.read()
    with monkeypatch.context() as patch:

        def fail(data: bytes) -> None:
            raise OSError("write failed")

        patch.setattr(connected._writer, "write", fail)
        with pytest.raises(OSError, match="write failed"):
            surface.render(ui.text("a"))
    assert surface._seq == 0 and surface._sent.main is None
    surface.render(ui.text("a"))
    frame = frames(term)[0]
    assert frame["s"] == 1 and frame["ops"][0][0:2] == ["add", "main"]


def test_handshake_keeps_items_after_hello_in_the_same_read(term: Term) -> None:
    term.reply()
    term.da1()
    term.event(ev="resize", cols=91)
    term.send(b"x")
    session = session_mod.start(term.in_r, term.out_w)
    assert session is not None
    with session:
        assert isinstance(session.poll(0), wire.ResizeEvent)
        assert session.caps.cols == 91
        assert session.poll(0) == Key("x", "x")


def test_nonblocking_poll_dispatches_available_ack(connected: Session, term: Term) -> None:
    surface = connected.open()
    for text in ("a", "b", "c"):
        surface.render(ui.text(text))
    term.read()
    term.event(ev="ack", sf=surface.id, s=1)
    term.send(b"z")
    assert connected.poll(0) == Key("z", "z")
    assert frames(term)[0]["s"] == 3


def test_input_iterator_flushes_ack_before_waiting_for_another_key(
    connected: Session, term: Term, monkeypatch: pytest.MonkeyPatch
) -> None:
    surface = connected.open()
    surface.render(ui.text("first"))
    surface.render(ui.text("second"))
    term.read()
    term.send(b"a")
    items = connected.input()
    assert next(items) == Key("a", "a")
    surface.render(ui.text("coalesced"))
    surface.focus("main.0")
    assert frames(term) == []
    term.event(ev="ack", sf=surface.id, s=2)
    read = connected._reader.read
    reads = 0

    def read_until_frame(timeout: float | None) -> bytes | None:
        nonlocal reads
        reads += 1
        if reads == 2:
            # No new key has arrived: the ack must already have sent this frame.
            assert frames(term) == [
                {
                    "sf": "s1",
                    "s": 3,
                    "ops": [["text", "main.0", "replace", "coalesced"], ["focus", "main.0"]],
                }
            ]
            term.send(b"z")
        return read(timeout)

    monkeypatch.setattr(connected._reader, "read", read_until_frame)
    assert next(items) == Key("z", "z")
    monkeypatch.undo()


def test_buffered_keys_do_not_starve_pending_frame_acks(connected: Session, term: Term) -> None:
    surface = connected.open()
    surface.render(ui.text("first"))
    surface.render(ui.text("second"))
    term.read()
    term.send(b"abc")
    items = connected.input()
    assert next(items) == Key("a", "a")
    surface.render(ui.text("coalesced"))
    surface.focus("main.0")
    assert frames(term) == []
    term.event(ev="ack", sf=surface.id, s=2)
    assert next(items) == Key("b", "b")
    assert frames(term) == [
        {
            "sf": "s1",
            "s": 3,
            "ops": [["text", "main.0", "replace", "coalesced"], ["focus", "main.0"]],
        }
    ]
    assert next(items) == Key("c", "c")


def test_short_poll_does_not_flush_before_thirty_ms(connected: Session, monkeypatch: pytest.MonkeyPatch) -> None:
    now = [100.0]
    inputs: list[bytes | None] = [b"\x1b", None, b"[A", b"\x1b", None, None]
    waits: list[float] = []
    monkeypatch.setattr(session_mod.time, "monotonic", lambda: now[0])

    def read(timeout: float | None) -> bytes | None:
        assert timeout is not None
        waits.append(timeout)
        data = inputs.pop(0)
        if data is None:
            now[0] += timeout
        return data

    monkeypatch.setattr(connected._reader, "read", read)
    assert connected.poll(0.001) is None
    now[0] += 0.009
    assert connected.poll(0.001) == Key("up")
    assert connected.poll(0.001) is None
    assert connected.poll(0.1) == Key("escape")
    assert waits[-1] == pytest.approx(0.029)
    monkeypatch.undo()


def test_submit_is_returned_after_its_handler(connected: Session, term: Term) -> None:
    seen: list[wire.ActionEvent] = []
    button = ui.html.button("Go", key="go", actions={"click": "submit"}, on_click=seen.append)
    term.event(ev="action", sf="s1", id="main.go", act="submit", values={"size": "l"})
    term.send(b"\x04")  # A swallowed submit would otherwise wait forever.
    answer = tern_sdk.ask(button, session=connected)
    assert answer == tern_sdk.Answer("main.go", "submit", {"size": "l"})
    assert len(seen) == 1 and answer.event is seen[0]


@pytest.mark.parametrize("gesture", ["click", "dblclick"])
def test_named_gesture_matches_action_and_value(connected: Session, term: Term, gesture: str) -> None:
    seen: list[wire.ActionEvent] = []
    node = ui.text("sort", actions={gesture: "sort=name"}, **{f"on_{gesture}": seen.append})
    connected.open().render(node)
    term.event(ev="action", sf="s1", id="main.0", act="sort", value="name")
    assert connected.poll(0) is None
    assert len(seen) == 1


def test_close_records_late_replies_without_dispatch(connected: Session, term: Term, tmp_path: Path) -> None:
    path = tmp_path / "drain.jsonl"
    connected._recorder = session_mod._Recorder(str(path))
    seen: list[object] = []
    surface = connected.open()
    surface.render(ui.text("x", on_click=seen.append))
    term.event(ev="ack", sf=surface.id, s=1)
    term.event(ev="action", sf=surface.id, id="main.0", act="click")
    term.send(b'\x1b_tsp;r;{"r":"future"}\x1b\\')
    connected.close()
    incoming = [r for line in path.read_text().splitlines() if (r := json.loads(line))["dir"] == "in"]
    assert [r["body"].get("ev", r["body"].get("r")) for r in incoming] == ["ack", "action", "future"]
    assert seen == []


def test_close_attempts_x_and_all_resets_after_write_errors(
    connected: Session, term: Term, monkeypatch: pytest.MonkeyPatch
) -> None:
    surface = connected.open()
    for text in ("a", "b", "c"):
        surface.render(ui.text(text))
    term.read()
    writes: list[bytes] = []
    restored: list[bool] = []
    monkeypatch.setattr(connected, "_tty", SimpleNamespace(restore=lambda: restored.append(True)))

    def fail(data: bytes) -> None:
        writes.append(data)
        raise OSError("broken output")

    monkeypatch.setattr(connected._writer, "write", fail)
    with pytest.raises(OSError, match="broken output"):
        connected.close()
    assert [message.verb for data in writes for message in term.messages(data)] == ["f", "x"]
    assert writes[-2:] == [b"\x1b[<u", b"\x1b[?2004l"]
    assert restored == [True] and connected.closed


def test_cleanup_is_armed_before_raw_setup_and_rolls_back(term: Term, monkeypatch: pytest.MonkeyPatch) -> None:
    restored: list[bool] = []
    previous = signal.getsignal(signal.SIGTERM)

    class FailingTty:
        def __init__(self, *_: object) -> None:
            pass

        def enter(self) -> None:
            assert signal.getsignal(signal.SIGTERM) != previous
            assert session_mod._TTY_LOCK.locked()
            raise OSError("partial raw setup")

        def restore(self) -> None:
            restored.append(True)

    monkeypatch.setattr(session_mod.os, "isatty", lambda fd: True)
    monkeypatch.setattr(session_mod, "_Tty", FailingTty)
    with pytest.raises(OSError, match="partial raw setup"):
        session_mod.start(term.in_r, term.out_w)
    assert restored == [True]
    assert signal.getsignal(signal.SIGTERM) == previous
    assert not session_mod._TTY_LOCK.locked()


def test_tty_cannot_be_armed_twice(term: Term, monkeypatch: pytest.MonkeyPatch) -> None:
    entries: list[bool] = []
    monkeypatch.setattr(session_mod.os, "isatty", lambda fd: True)
    monkeypatch.setattr(session_mod._Tty, "enter", lambda self: entries.append(True))
    monkeypatch.setattr(session_mod._Tty, "restore", lambda self: None)
    term.reply()
    session = session_mod.start(term.in_r, term.out_w)
    assert session is not None
    try:
        with pytest.raises(RuntimeError, match="already owns"):
            session_mod.start(term.in_r, term.out_w)
        assert entries == [True]
    finally:
        session.close()
    assert not session_mod._TTY_LOCK.locked()


def test_mode_setup_failure_resets_and_restores(term: Term, monkeypatch: pytest.MonkeyPatch) -> None:
    restored: list[bool] = []
    writes: list[bytes] = []
    original = session_mod._Writer.write
    monkeypatch.setattr(session_mod.os, "isatty", lambda fd: True)
    monkeypatch.setattr(session_mod._Tty, "enter", lambda self: None)
    monkeypatch.setattr(session_mod._Tty, "restore", lambda self: restored.append(True))

    def write(self: Any, data: bytes) -> None:
        writes.append(data)
        if data == b"\x1b[?2004h\x1b[>1u":
            raise OSError("partial modes")
        original(self, data)

    monkeypatch.setattr(session_mod._Writer, "write", write)
    term.reply()
    with pytest.raises(OSError, match="partial modes"):
        session_mod.start(term.in_r, term.out_w)
    assert writes[-2:] == [b"\x1b[<u", b"\x1b[?2004l"]
    assert restored == [True] and not session_mod._TTY_LOCK.locked()


@pytest.mark.parametrize("signum", [signal.SIGINT, signal.SIGTERM, signal.SIGHUP])
def test_signals_use_normal_close_and_drain(
    connected: Session, term: Term, monkeypatch: pytest.MonkeyPatch, signum: int
) -> None:
    connected.open().render(ui.text("x"))
    term.read()
    previous: list[int] = []
    original_handler = connected._signals[signum]
    connected._signals[signum] = lambda sig, frame: previous.append(sig)
    waits: list[float | None] = []
    original = connected._reader.read

    def read(timeout: float | None) -> bytes | None:
        waits.append(timeout)
        return original(timeout)

    monkeypatch.setattr(connected._reader, "read", read)
    try:
        connected._on_signal(signum, None)
    finally:
        signal.signal(signum, original_handler)
    assert connected.closed and previous == [signum]
    assert [m.verb for m in term.messages()] == ["x"]
    assert waits and 0 < waits[0] <= 0.05


def test_windows_console_skips_noncharacters_and_preserves_unicode(monkeypatch: pytest.MonkeyPatch) -> None:
    now = [10.0]
    batches = [[(2, 0, 0)], [(1, 1, 0)], [(1, 0, ord("z"))], [(1, 1, 0xD83D)], [(1, 1, 0xDE00)]]

    class Kernel:
        def WaitForSingleObject(self, handle: int, ms: int) -> int:
            now[0] += 0.001
            return 0 if batches else 258

        def ReadConsoleInputW(self, handle: int, records: Any, size: int, count: Any) -> int:
            batch = batches.pop(0)
            count._obj.value = len(batch)
            for record, (kind, down, char) in zip(records, batch, strict=False):
                record.kind = kind
                record.event.key.down = down
                record.event.key.char = char
                record.event.key.repeat = 1
            return 1

    reader = session_mod._WinWait.__new__(session_mod._WinWait)
    reader._console = True
    reader._kernel = Kernel()
    reader._handle = 1
    reader._decoder = codecs.getincrementaldecoder("utf-16-le")("replace")
    monkeypatch.setattr(session_mod.time, "monotonic", lambda: now[0])
    assert reader.read(0) is None
    assert reader.read(0.001) is None
    assert reader.read(0.02) == "😀".encode()
    assert reader.read(0) is None
    assert ctypes.sizeof(session_mod._ConsoleRecord) == 20


def test_windows_partial_raw_setup_rolls_back(monkeypatch: pytest.MonkeyPatch) -> None:
    calls: list[tuple[int, int]] = []

    class Kernel:
        def GetConsoleMode(self, handle: int, mode: Any) -> int:
            mode._obj.value = 7
            return 1

        def SetConsoleMode(self, handle: int, mode: int) -> int:
            calls.append((handle, mode))
            return int(len(calls) != 2)

    monkeypatch.setitem(sys.modules, "msvcrt", SimpleNamespace(get_osfhandle=lambda fd: fd))
    monkeypatch.setattr(session_mod, "_console_kernel", Kernel)
    saved: list[tuple[int, int]] = []
    with pytest.raises(OSError, match="output failed"):
        session_mod._win_console_raw(1, 2, saved)
    assert calls[-2:] == [(1, 7), (2, 7)]


def example(name: str) -> Any:
    path = Path(__file__).resolve().parents[1] / "examples" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(f"example_{name}", path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_table_example_accepts_entry_named_head(tmp_path: Path, monkeypatch: pytest.MonkeyPatch) -> None:
    (tmp_path / "head").write_text("entry")
    monkeypatch.chdir(tmp_path)
    views: list[ui.Node] = []
    monkeypatch.setattr(tern_sdk, "show", lambda node, **kwargs: views.append(node))
    example("table").main()
    built = View.build(as_view(views[0]))
    assert {"main.0.head", "main.0.entry:head"} <= built.nodes().keys()


@pytest.mark.parametrize(
    ("values", "expected"),
    [
        ({"size": "s"}, "size: Small"),
        ({"size": "m"}, "size: Medium"),
        ({"size": "l"}, "size: Large"),
        ({}, "no size picked"),
        ({"size": ["s", "l"]}, "no size picked"),
    ],
)
def test_ask_example_reads_only_submit_values(
    values: dict[str, Any],
    expected: str,
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    def submit(node: ui.Node, **kwargs: Any) -> tern_sdk.Answer:
        nodes = View.build(as_view(node)).nodes()
        radios = [n for n in nodes.values() if n.props.get("type") == "radio"]
        assert len(radios) == 3
        assert all(not radio.handlers for radio in radios)
        return tern_sdk.Answer("main.ask.go", "submit", values)

    monkeypatch.setattr(tern_sdk, "ask", submit)
    assert example("ask").main() == 0
    assert capsys.readouterr().out == f"{expected}\n"


def test_chat_draft_edits_code_points() -> None:
    draft = example("chat").Draft()
    draft.insert("a😀界")
    assert draft.utf16_cursor() == 4
    draft.key(Key("left"))
    assert draft.utf16_cursor() == 3
    draft.key(Key("backspace"))
    assert draft.text == "a界" and draft.utf16_cursor() == 1
    draft.key(Key("delete"))
    assert draft.text == "a"
