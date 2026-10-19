# pane-hints

Show large numbers **over the panes themselves**, then focus a pane by number.
This window-only plugin uses temporary CSS hints and a small native input bar,
not a pane list. Tested with desktop Tern 0.9.4.

1. On macOS, press **Ctrl+Cmd+G** together, or run **Show pane numbers** from the command palette.
2. Type a number displayed over a pane and press **Enter** to focus it.
3. **Escape** cancels and removes the numbers without changing focus.

Numbers follow the split tree (left/top before right/bottom), then floating
panes in their layout order. Other tabs and parked panes are excluded. Zoomed
layouts only offer the focused tiled pane and floating panes; hidden panes
retain their positions, so visible numbers need not start at 1. Multi-digit
numbers are accepted (`1`, `10`, `12`); leading zeroes also work.

The original pane IDs are captured when hints open. Closing/creating a pane,
changing tabs, or changing pane focus invalidates the hints. A remaining
input bar can be dismissed with Escape. Selection also rechecks the layout
order and zoom state before focusing. A reorder that preserves focus has no
SDK event: its hints may remain until Enter, at which point the stale choice
is rejected rather than redirected to another program. No global number or Escape bindings are installed, and selection
keys are consumed by the native input bar, not the terminal.

## Install from a checkout

```sh
tern plugin link /path/to/tern-sdk/plugins/examples/pane-hints
```

To add an optional tmux-style **Ctrl+A → q** shortcut, merge this entry into
Tern's existing `settings.json`. The default **Ctrl+Cmd+G** remains available:

```json
{
  "keybinds": {
    "ctrl+a>q": "plugin.pane-hints.show"
  }
}
```

## Implementation limits

This is a prototype for
[stencil-hq/tern-sdk#103](https://github.com/stencil-hq/tern-sdk/issues/103).
`tern.css` can render generated content over existing panes, but the native
chrome selectors and their ordering are not a stable plugin API. The current
implementation targets `#panel > .tn-stage > section.tn-pane`, in the all-window `session:panes()` order, including hidden tabs and parked
panes. The displayed numbers independently follow the active split tree and floats. It uses `:first-of-type` for the first hint
because `:nth-of-type(1)` did not draw the first generated glyph in the tested
build. Recheck the rendered numbers and their targets after Tern upgrades.

The public prompt API has no change callback or dismiss handle, so choosing
still requires **Enter** and there is no automatic timeout. A native pane-hint
action would remove the private CSS dependency and could support immediate
single-key selection. Parked-pane restoration is not included.

## Verify

`verify.py` starts an isolated desktop window and daemon under a temporary
directory and leaves the normal Tern profile alone. It requires Python 3, Pillow (`python3 -m pip install Pillow`),
and a logged-in Tern desktop installation; allow two to three minutes.

```sh
python3 plugins/examples/pane-hints/verify.py
luau-lsp analyze --definitions=@tern=plugins/tern.d.luau \
  plugins/examples/pane-hints/window.luau
```

The integration check covers keyboard invocation, no-list presentation,
selection/cancellation, selection mapping with 12 panes, zoom, floating panes, other-tab
and parked-pane exclusion, disappearing targets, and selection-key isolation.
Rendered-image checks verify large bright glyphs at visible pane centers,
including a tab whose panes follow hidden and parked nodes. Also manually
inspect a mixed split and floating layout to verify the actual digit values.
