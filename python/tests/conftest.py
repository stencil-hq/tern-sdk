"""A scripted terminal on `os.pipe()`s: the program reads what the test
sends and the test reads what the program writes."""

from __future__ import annotations

import json
import os
from collections.abc import Iterator
from dataclasses import dataclass
from typing import Any

import pytest

from tern_sdk import session as session_mod
from tern_sdk import wire

HELLO = {
    "r": "hello", "v": 1, "term": "tern", "ver": "0.4.3", "kinds": list(wire.KINDS),
    "features": ["flow", "styles", "blobs"], "apc": 65536, "credits": 2, "cols": 120,
    "cell": {"w": 8, "h": 17}, "dark": True, "reduceMotion": False,
}  # fmt: skip


@dataclass
class Sent:
    """One TSP message the program wrote."""

    verb: str
    params: dict[str, str]
    body: Any


class Term:
    """The terminal side of a session."""

    def __init__(self) -> None:
        self.in_r, self.in_w = os.pipe()
        self.out_r, self.out_w = os.pipe()
        os.set_blocking(self.out_r, False)
        self._out = bytearray()

    def send(self, data: bytes) -> None:
        os.write(self.in_w, data)

    def reply(self, **override: Any) -> None:
        self.send(b"\x1b_tsp;r;" + json.dumps({**HELLO, **override}).encode() + b"\x1b\\")

    def da1(self) -> None:
        self.send(b"\x1b[?62;52;c")

    def event(self, **body: Any) -> None:
        self.send(b"\x1b_tsp;e;" + json.dumps(body).encode() + b"\x1b\\")

    def read(self) -> bytes:
        """Everything written since the last read."""
        while True:
            try:
                chunk = os.read(self.out_r, 65536)
            except BlockingIOError:
                break
            if not chunk:
                break
            self._out += chunk
        out, self._out = bytes(self._out), bytearray()
        return out

    def messages(self, data: bytes | None = None) -> list[Sent]:
        """The TSP messages in `data` (default: a fresh read), chunks joined."""
        data = self.read() if data is None else data
        out: list[Sent] = []
        joining: tuple[str, dict[str, str], bytearray] | None = None
        pos = 0
        while (start := data.find(b"\x1b_tsp;", pos)) >= 0:
            end = data.index(b"\x1b\\", start)
            message = wire.split(data[start + 6 : end])
            assert message is not None
            params = dict(message.params)
            pos = end + 2
            if "c" in params:
                if joining is None:
                    joining = (message.verb, {k: v for k, v in params.items() if k not in ("c", "m")}, bytearray())
                joining[2].extend(message.body)
                if params.get("m") == "1":
                    continue
                verb, first, body = joining
                joining = None
                out.append(Sent(verb, first, _body(verb, bytes(body))))
                continue
            out.append(Sent(message.verb, params, _body(message.verb, message.body)))
        return out

    def close(self) -> None:
        for fd in (self.in_r, self.in_w, self.out_r, self.out_w):
            try:
                os.close(fd)
            except OSError:
                pass


def _body(verb: str, body: bytes) -> Any:
    return body.decode("ascii") if verb == "b" else json.loads(body)


@pytest.fixture
def term() -> Iterator[Term]:
    t = Term()
    yield t
    t.close()


@pytest.fixture
def connected(term: Term) -> Iterator[session_mod.Session]:
    """A session that completed its handshake on `term` (output drained)."""
    term.reply()
    term.da1()
    s = session_mod.start(term.in_r, term.out_w, app="test", timeout=1.0)
    assert s is not None
    term.read()
    yield s
    s.close()
