"""The "Which size?" form from the protocol book; prints the answer.

Inside Tern the form is drawn natively and stays in the scrollback; anywhere
else the program asks on its own."""

from __future__ import annotations

import sys

import tern_sdk
from tern_sdk import ui

SIZES = (("s", "Small"), ("m", "Medium"), ("l", "Large"))


def form() -> ui.Node:
    """The form: a card asking the question, three radios and a submit button."""

    return ui.html.form(
        ui.card(
            ui.row(
                *(
                    ui.html.label(
                        ui.html.input(type="radio", name="size", value=value, key="r"),
                        ui.html.span(label, key="t"),
                        key=value,
                    )
                    for value, label in SIZES
                ),
                gap="lg",
                key="sizes",
            ),
            ui.button("Create", variant="primary", actions={"click": "submit"}, key="go"),
            head="Which size?",
            max={"w": "48ch"},
            key="q",
        ),
        key="ask",
    )


def main() -> int:
    try:
        answer = tern_sdk.ask(form(), font="sans")
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
