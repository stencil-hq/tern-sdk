"""The "Which size?" form from the protocol book; prints the answer.

Inside Tern the form is drawn natively and stays in the scrollback; anywhere
else the program asks on its own."""

from __future__ import annotations

import sys

import tern_sdk
from tern_sdk import ui

CSS = """
.ask { display: flex; flex-direction: column; gap: 8px; max-width: 460px;
  padding: 10px 12px; border-radius: 8px; background: var(--card);
  box-shadow: inset 0 0 0 1px var(--l2) }
.ask p { margin: 0 }
.ask .sizes { display: flex; gap: 14px }
.ask :checked+span { color: var(--accent) }
.ask button { align-self: flex-start; padding: 3px 12px; border-radius: 6px;
  background: var(--accent-fill); color: #fff }
"""

SIZES = (("s", "Small"), ("m", "Medium"), ("l", "Large"))


def form() -> ui.Node:
    """The form: a question, three radios and a submit button."""

    return ui.html.form(
        ui.html.p("Which size?", key="q"),
        ui.html.div(
            *(
                ui.html.label(
                    ui.html.input(type="radio", name="size", value=value, key="r"),
                    label,
                    key=value,
                )
                for value, label in SIZES
            ),
            class_="sizes",
            key="sizes",
        ),
        ui.html.button("Create", actions={"click": "submit"}, key="go"),
        class_="ask",
        key="ask",
    )


def main() -> int:
    try:
        answer = tern_sdk.ask(form(), css=CSS)
    except tern_sdk.Unsupported:
        try:
            size = input("Which size? [s/m/l] ").strip().lower()[:1]
        except EOFError:
            size = ""
        if size not in dict(SIZES):
            print("cancelled")
            return 1
        print(f"size: {dict(SIZES)[size]}")
        return 0
    if answer is None:
        print("cancelled")
        return 1
    choice = answer.values.get("size")
    print(f"size: {dict(SIZES)[choice]}" if isinstance(choice, str) and choice in dict(SIZES) else "no size picked")
    return 0


if __name__ == "__main__":
    sys.exit(main())
