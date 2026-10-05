"""An `inline` surface: a transcript in `main` and an `editor` in `dock` that
the program edits from keys. Enter sends, Escape quits.

Outside Tern it is a line prompt that echoes what you send."""

from __future__ import annotations

import tern_sdk
from tern_sdk import FocusEvent, Key, ui


class Draft:
    """The editor's text and caret, edited from keys."""

    def __init__(self) -> None:
        self.text = ""
        self.cursor = 0  # in code points

    def utf16_cursor(self) -> int:
        """The caret in UTF-16 units, as the `cursor` prop counts it."""
        return len(self.text[: self.cursor].encode("utf-16-le")) // 2

    def insert(self, s: str) -> None:
        self.text = self.text[: self.cursor] + s + self.text[self.cursor :]
        self.cursor += len(s)

    def key(self, key: Key) -> None:
        """Applies an editing key."""
        if key.name == "paste" and key.text:
            self.insert(key.text)
        elif key.name == "backspace" and self.cursor > 0:
            self.text = self.text[: self.cursor - 1] + self.text[self.cursor :]
            self.cursor -= 1
        elif key.name == "delete":
            self.text = self.text[: self.cursor] + self.text[self.cursor + 1 :]
        elif key.name == "left" or (key.ctrl and key.name == "b"):
            self.cursor = max(self.cursor - 1, 0)
        elif key.name == "right" or (key.ctrl and key.name == "f"):
            self.cursor = min(self.cursor + 1, len(self.text))
        elif key.name == "home" or (key.ctrl and key.name == "a"):
            self.cursor = 0
        elif key.name == "end" or (key.ctrl and key.name == "e"):
            self.cursor = len(self.text)
        elif key.ctrl and key.name == "u":
            self.text, self.cursor = self.text[self.cursor :], 0
        elif key.text is not None and not (key.ctrl or key.alt or key.meta):
            self.insert(key.text)


def view(messages: list[str], draft: Draft) -> dict[str, ui.Node]:
    """The transcript and the composer."""
    transcript = [ui.card(ui.md(text), head="You", tone="user", key=f"m{n}") for n, text in enumerate(messages)] or [
        ui.md("Type a message and press **Enter**. **Escape** quits.", key="hint")
    ]
    return {
        "main": ui.col(*transcript, gap="sm"),
        "dock": ui.col(
            ui.editor(
                draft.text,
                cursor=draft.utf16_cursor(),
                placeholder="Message",
                prompt=[ui.span("> ", "accent")],
                key="ed",
            ),
            ui.status(ui.seg("chat", icon="message"), ui.seg(f"{len(messages)} sent", side="right"), key="bar"),
        ),
    }


def plain_chat() -> None:
    while True:
        try:
            line = input("> ")
        except EOFError:
            print()
            break
        print(f"you: {line}")


def main() -> None:
    session = tern_sdk.connect(app="chat")
    if session is None:
        plain_chat()
        return
    messages: list[str] = []
    draft = Draft()
    with session, session.open(mode="inline", title="chat") as surface:
        surface.render(view(messages, draft))
        surface.focus("dock.ed")
        for item in session.input():
            if isinstance(item, FocusEvent):
                surface.focus("dock.ed")
                continue
            if not isinstance(item, Key):
                continue
            if item.name == "escape" or (item.ctrl and item.name in ("c", "d")):
                break
            if item.name == "enter" and not (item.shift or item.alt):
                if draft.text.strip():
                    messages.append(draft.text)
                    draft = Draft()
            elif item.name == "enter":
                draft.insert("\n")
            else:
                draft.key(item)
            surface.render(view(messages, draft))


if __name__ == "__main__":
    main()
