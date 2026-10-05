"""Wire messages and decoding beyond the corpus."""

from __future__ import annotations

import pytest

from tern_sdk import wire
from tern_sdk.input import DA1, InputParser


def test_absent_fields_are_omitted_and_null_kept_where_it_means_something() -> None:
    assert wire.open_surface("s1").payload == {"id": "s1"}
    assert wire.stylesheet("s1", "main", None).payload == {"sf": "s1", "name": "main"}
    assert wire.Ops.focus(None) == ["focus", None]
    assert wire.dumps(wire.frame("s1", 1, [wire.Ops.add("a", "main", None, {"id": "a", "k": "rule"})]).payload) == (
        '{"sf":"s1","s":1,"ops":[["add","a","main",null,{"id":"a","k":"rule"}]]}'
    )
    assert wire.close_surface("s1", keep=False).body() == b'{"id":"s1","keep":false}'


def test_json_is_compact_utf8_with_escaped_controls() -> None:
    assert wire.dumps({"t": "é\x1b\n"}) == '{"t":"é\\u001b\\n"}'


def test_twelve_ops() -> None:
    ops = [
        wire.Ops.set("a", {"x": None}), wire.Ops.text("a", "append", "b"), wire.Ops.splice("a", 1, 2, "c"),
        wire.Ops.move("a", "p", None), wire.Ops.delete("a"), wire.Ops.settle("a"), wire.Ops.reveal("a"),
        wire.Ops.scroll("a", "page-down"), wire.Ops.suspend(), wire.Ops.resume(),
    ]  # fmt: skip
    assert [op[0] for op in ops] == [
        "set", "text", "splice", "move", "del", "settle", "reveal", "scroll", "suspend", "resume",
    ]  # fmt: skip
    assert ops[6] == ["reveal", "a", "nearest"]


def test_chunk_ids_are_base36_and_drawn_only_when_chunking() -> None:
    ids = wire.ChunkIds()
    assert [ids() for _ in range(37)][-2:] == ["10", "11"]
    calls: list[int] = []

    def chunk() -> str:
        calls.append(1)
        return "z"

    wire.encode("f", b"{}", limit=10, chunk=chunk)
    assert calls == []
    assert wire.encode("f", b"0123456789ab", limit=10, chunk=chunk).count(b"c=z;") == 2


def test_oversized_blob_is_refused() -> None:
    with pytest.raises(ValueError):
        wire.blob(bytes(wire.MAX_BLOB + 1))
    assert wire.blob(b"", None).params == (("id", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),)


def test_events_are_typed_and_tolerate_bad_fields() -> None:
    e = wire.decode_event(
        {"ev": "action", "sf": "s1", "id": "go", "act": "sort", "value": "name", "mods": ["shift", 3]}
    )
    assert isinstance(e, wire.ActionEvent)
    assert (e.act, e.value, e.mods, e.values) == ("sort", "name", ("shift",), None)
    edit = wire.decode_event({"ev": "edit", "id": "ed", "from": 1, "to": 3.0, "text": "x", "cursor": "?", "len": 9})
    assert isinstance(edit, wire.EditEvent) and (edit.from_, edit.to, edit.cursor) == (1, 3, None)
    ack = wire.decode_event({"ev": "ack", "s": True})
    assert isinstance(ack, wire.AckEvent) and ack.s is None
    change = wire.decode_event({"ev": "change", "id": "p", "item": "row", "value": [1]})
    assert isinstance(change, wire.ChangeEvent) and change.value == [1] and change.item == "row"
    unknown = wire.decode_event({"ev": "future", "x": 1})
    assert isinstance(unknown, wire.UnknownEvent) and unknown.raw == {"ev": "future", "x": 1}
    hello = wire.decode_reply({"r": "hello", "cell": {"w": "x"}, "reduceMotion": True})
    assert isinstance(hello, wire.HelloReply) and hello.cell is None and hello.reduce_motion is True


def test_split_keeps_body_with_semicolons_and_equals() -> None:
    m = wire.split(b'f;c=k7;m=1;{"a":"x=1;"}')
    assert m is not None and m.params == (("c", "k7"), ("m", "1")) and m.body == b'{"a":"x=1;"}'
    assert wire.split(b"") is None and wire.split(b";x") is None


def test_parser_drops_an_oversized_message_and_recovers(monkeypatch: pytest.MonkeyPatch) -> None:
    import tern_sdk.input as input_mod

    monkeypatch.setattr(input_mod, "MAX_MESSAGE", 64)
    parser = InputParser()
    items = parser.feed(b"\x1b_tsp;e;" + b"x" * 100)
    items += parser.feed(b"y" * 10 + b"\x1b\\ok\x1b[?1c")
    assert items == [b"ok", DA1]


def test_parser_holds_partial_paste_end() -> None:
    parser = InputParser()
    first = parser.feed(b"\x1b[200~ab\x1b_tsp;e;{}\x1b\\\x1b[20")
    assert b"".join(i for i in first if isinstance(i, bytes)) == b"\x1b[200~ab\x1b_tsp;e;{}\x1b\\"
    assert parser.feed(b'1~\x1b_tsp;e;{"ev":"ack"}\x1b\\')[0] == b"\x1b[201~"
