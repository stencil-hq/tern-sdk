# workspaces

Opens the tabs a project describes in `.tern/workspace.json` at its root.
`.tern/workspace.json` in this folder is a sample:

```json
{
  "name": "webapp",
  "tabs": [
    { "name": "code", "split": "right", "panes": [
      { "command": "nvim ." },
      { "split": "down", "panes": [
        { "cwd": "web", "command": "npm run dev" },
        { "cwd": "api", "command": "cargo watch -x run" }
      ] }
    ] },
    { "name": "shell", "panes": [{}, { "block": "longrun.slow" }] },
    { "name": "infra", "cwd": "infra" }
  ]
}
```

- **Open workspace** (palette, `cmd+alt+shift+o`; `ctrl+alt+shift+o` off
  macOS) finds the nearest `.tern/workspace.json` at or above the focused
  pane's directory and opens its tabs.
- **Edit workspace file** opens that file beside the focused pane.
- **At launch**, `TERN_WORKSPACE` (an absolute path to the root or to its
  `workspace.json`) in Tern's environment opens that workspace; tabs a
  restored session already has are adopted instead of opened twice.
- **New Tab** from a pane inside an opened workspace starts at the
  workspace root; elsewhere it behaves as usual.
- **Status line**: panes inside an opened workspace show its name.

The plugin is window-only: it needs no daemon side and nothing on remote
hosts.

## Install

```sh
tern plugin install github.com/stencil-hq/tern-sdk/plugins/examples/workspaces
```

or, to load a clone in place and reload on every save:

```sh
git clone https://github.com/stencil-hq/tern-sdk
tern plugin link tern-sdk/plugins/examples/workspaces
```

To try the sample, open a shell in this folder and run **Open workspace**:
the tabs' directories that don't exist start at your home directory.

Walkthrough: <https://docs.stencil.so/tern/examples/workspaces.html>.
