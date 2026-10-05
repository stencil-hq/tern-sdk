"""A `flow` surface: a running card with a spinner and an elapsed timer, log
lines streamed into it, a progress bar, then done. Ctrl+C cancels.

Outside Tern the same steps print as plain lines."""

from __future__ import annotations

import time

import tern_sdk
from tern_sdk import Key, ui

STEPS = (
    "resolving dependencies",
    "fetching 12 crates",
    "compiling stencil-css",
    "compiling stencil-term",
    "compiling tern",
    "linking",
    "writing target/release/tern",
)
PAUSE = 0.4


def view(log: list[str], done: int, status: ui.Status, took: float | None) -> ui.Node:
    """The build card at `done` of the steps."""
    timer = ui.elapsed(0, key="t") if took is None else ui.elapsed(stopped=took, key="t")
    head = ui.row(
        ui.spinner("Building", style="dots", key="sp")
        if status == "running"
        else ui.text("Built" if status == "done" else "Cancelled", key="sp"),
        timer,
        gap="sm",
        key="h",
    )
    return ui.card(
        head,
        ui.ansi("\n".join(log), key="log"),
        ui.progress(done / len(STEPS), f"{done}/{len(STEPS)}", tone="success" if status == "done" else None, key="p"),
        head="cargo build --release",
        status=status,
        key="build",
    )


def plain_run() -> None:
    for n, step in enumerate(STEPS, 1):
        print(f"[{n}/{len(STEPS)}] {step}", flush=True)
        time.sleep(PAUSE)
    print("done")


def main() -> None:
    session = tern_sdk.connect(app="progress")
    if session is None:
        plain_run()
        return
    with session:
        surface = session.open(mode="flow")
        started = time.monotonic()
        log: list[str] = []
        status: ui.Status = "running"
        surface.render(view(log, 0, status, None))
        for n, step in enumerate(STEPS, 1):
            deadline = time.monotonic() + PAUSE
            while (left := deadline - time.monotonic()) > 0:
                item = session.poll(left)
                if isinstance(item, Key) and item.ctrl and item.name == "c":
                    status = "cancelled"
                    break
            if status == "cancelled":
                break
            log.append(f"   \x1b[32mok\x1b[0m {step}")
            surface.render(view(log, n, status, None))
        took = (time.monotonic() - started) * 1000
        surface.render(view(log, len(log), "done" if status == "running" else status, took))
        surface.close()
    print("done" if status == "running" else status)


if __name__ == "__main__":
    main()
