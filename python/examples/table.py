"""The current directory as a table, sizes right-aligned by a stylesheet.

Inside Tern it is a native table kept in the scrollback like command output;
anywhere else it prints as plain text."""

from __future__ import annotations

from pathlib import Path

import tern_sdk
from tern_sdk import ui

CSS = """
.ls { width: auto; border-spacing: 0 1px }
.ls th, .ls td { padding: 0 28px 0 0 }
.ls th { text-align: left; color: var(--sf-c-muted); font-weight: 500 }
.ls .size { text-align: right; font-variant-numeric: tabular-nums }
.ls .dir { color: var(--sf-c-accent) }
"""


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
    rows = [ui.html.tr(ui.html.th("Name"), ui.html.th("Size", class_="size"), key="head")]
    for path in entries:
        is_dir = path.is_dir()
        size = "-" if is_dir else human(path.stat().st_size)
        name = path.name + ("/" if is_dir else "")
        rows.append(
            ui.html.tr(
                ui.html.td(name, class_="dir" if is_dir else None, key="n"),
                ui.html.td(size, class_="size", key="s"),
                key=f"entry:{path.name}",
            )
        )
    tern_sdk.show(ui.html.table(*rows, class_="ls"), css=CSS)


if __name__ == "__main__":
    main()
