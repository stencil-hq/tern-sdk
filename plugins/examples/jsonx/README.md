# jsonx

A JSON Explorer block: a JSON file as a collapsible tree, members in
document order and numbers exactly as written, with a breadcrumb and the
jq path of the selected value.

- `↑`/`↓` (or `k`/`j`), `Home`/`End`, `PageUp`/`PageDown` move.
- `→` (or `l`) expands a container or steps into it; `←` (or `h`)
  collapses it or steps out to its parent; `Enter`/`Space` toggles.
- `y` copies the jq path (`.features[3].properties["name:en"]`).
- `c` collapses everything, `r` reloads the file, `o` opens it as text.
- Which containers are expanded, and the selected row, survive restarts.

Files up to 4 MiB open; larger ones, unreadable files and invalid JSON
get an error view (the parse error with its line and column) with **Open
as text** and **Reload**.

Ways to open it:

- **Open with › JSON Explorer** in the menu of any `*.json` or `*.geojson`
  file.
- Plain opens of `.json` files from the Files pane or the palette go to the
  explorer. **Toggle JSON explorer for .json opens** in the palette turns
  that off (and on again); the choice is kept in the plugin's `kv.json`.
  Opens with a placement (beside, split, preview) or a line number, and
  links in panes, still open the editor.

## Install

From where you cloned the SDK (`git clone https://github.com/stencil-hq/tern-sdk`):

```sh
tern plugin install tern-sdk/plugins/examples/jsonx
```

or, to load it in place and reload on every save:

```sh
tern plugin link tern-sdk/plugins/examples/jsonx
```

Walkthrough: <https://docs.stencil.so/tern/examples/jsonx.html>.
