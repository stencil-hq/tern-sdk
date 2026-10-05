# Canvas Dashboard

A window-only plugin that opens a persistent native canvas and handles its
Refresh button. No host entry, background program or worker is needed.

From where you cloned the SDK (`git clone https://github.com/stencil-hq/tern-sdk`):

```sh
tern plugin install tern-sdk/plugins/examples/canvas
```

Run **Open canvas dashboard** from the palette. Refresh replaces only the
keyed status node with the window's current monotonic timestamp. Running the
command again finds the owned canvas in `cx.canvas:list()` and focuses it,
including after plugin reload. Closing the pane removes it normally.

The canvas owner is `canvas-demo`; only this plugin can update it. Other
plugins and Carly may read it. Click handlers execute only in the clicking
window. If the plugin is disabled or absent there, clicking shows an info
toast instead. The content persists in the host across daemon restarts.

See `window.luau` for `open`, replica `list`, keyed `set` and `canvas_action`;
`plugin.toml` declares only the window entry. Full API and limits:
<https://docs.stencil.so/tern/reference/api-window.html#canvas-cx>.
