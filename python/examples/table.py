"""The current directory as a native `table`: names with folder/file icons,
sizes right-aligned in a mono column, no stylesheet.

Inside Tern it is a native table kept in the scrollback like command output;
anywhere else it prints as plain text."""

from __future__ import annotations

from pathlib import Path

import tern_sdk
from tern_sdk import ui

COLS: list[ui.TableCol] = [
    {"id": "name", "head": "Name", "grow": 1},
    {"id": "size", "head": "Size", "align": "end", "mono": True},
]


def human(size: int) -> str:
    """A byte count as `312`, `2.1K`, `14M`."""
    value = float(size)
    for unit in ("", "K", "M", "G", "T"):
        if value < 1024 or unit == "T":
            if not unit:
                return str(int(value))
            return f"{value:.1f}{unit}" if value < 10 else f"{value:.0f}{unit}"
        value /= 1024
    return str(size)


def main() -> None:
    entries = sorted(Path.cwd().iterdir(), key=lambda p: (not p.is_dir(), p.name.lower()))
    rows: list[ui.TableRow] = []
    for path in entries:
        is_dir = path.is_dir()
        name: ui.TextCell = {"text": path.name + ("/" if is_dir else ""), "icon": "folder" if is_dir else "file"}
        size = "-" if is_dir else human(path.stat().st_size)
        rows.append({"id": path.name, "cells": {"name": name, "size": size}})
    tern_sdk.show(ui.table(COLS, rows, empty="Empty directory"), font="sans")


if __name__ == "__main__":
    main()
