# @stencil-hq/tern

TypeScript SDK for the Tern Surface Protocol (TSP): a program running in a
terminal pane describes its UI as a tree of nodes, and Tern lays it out,
draws and animates it natively. Outside Tern the same program falls back to
plain text.

ESM, zero runtime dependencies. Runs from source under Bun, and from `dist/`
under Node 22 or newer.

## Install

```sh
bun add @stencil-hq/tern
```

## TSX

Point the JSX runtime at the package in `tsconfig.json`:

```json
{
   "compilerOptions": {
      "jsx": "react-jsx",
      "jsxImportSource": "@stencil-hq/tern"
   }
}
```

Lowercase elements are TSP kinds with typed props (`<card>`, `<md>`,
`<table>`, …, every kind but Tern's own `block`), plus every common prop
(`key`, `role`, `tone`, `hidden`, `actions`, `min`, `max`, …). String
children become the `text` of text kinds; `<span s="accent">` inside
`<text>` builds styled spans. `el` nodes come from the `html` namespace:
`<html.form class="ask">`, `<html.input type="radio" name="size" value="s" />`.
Function components, fragments, arrays and `null`/`false`/`undefined`
children work as expected.

Kind-specific props replace common props of the same name (`picker.actions`,
`message.actions` and `attachment.actions`, `list.max` and the `max` counts of
`avatars` and `breadcrumb`, the numeric `min`/`max` of `slider`, the day
`min`/`max` of `calendar` and `date`, `split.min`, styled `tool.title`,
`picker.title`, `empty.title` and `trace.title`). `agent.status` uses
`AgentStatus`: pending, running, done, failed, aborted, idle or parked.
Table meter cells use `{ meter: { value: 0.5 } }` (or `parts`, thresholds,
tone and title inside `meter`); picker action `disabled` accepts `true` or
a reason string.

Controls and panels:

- `<button variant? size? icon? keys? on? disabled? busy?>Label</button>`:
  the `button` kind. A click sends `action` with `act: 'click'` (or
  `actions.click`); `onClick` runs on it. `variant` is `default`, `primary`
  (one per view), `quiet` or `danger`; `size` is `md` (bars, heads, forms)
  or `sm` (rows).
- `<stats items={[{ label, value, sub?, tone? }]} />`: a flat strip of stat
  tiles (at most four), placed directly on the page, never in a card.
- `<table selectable selected={id} empty="No jobs">`: rows take clicks
  (`onSelect`/`onActivate` get the row as `event.item`); rows take
  `disabled`, `menu` and `title`; columns take `mono`. Cells are spans, or
  `{ meter }`, `{ text, dot?, icon?, tone? }`, `{ badge: { text, tone? } }`,
  `{ button: { text, act, variant? } }` (sends `act` with the row as
  `event.item`, handled by `onAction`) or `{ spark: { values } }`.
  `sort={{ col, dir }}` with `sortable` columns sends `action` `sort`;
  `multi` with `picked={ids}` sends `change` with the picked ids, and `bar`
  buttons send their `act` with them as `event.value`.
- `<tabs>`: underline tabs (the default) switch a page's views;
  `<tabs variant="segmented">` is the inline well for a switch inside a view.
- `card` and `overlay` take `icon`, `meta` and `end`; `overlay` takes
  `foot`. `card.head`, `card.end`, `section.head`, `overlay.end` and
  `overlay.foot` accept a node: it becomes the child keyed `head`/`end`/
  `foot` (id `<parent>.end`, …) and its handlers run as usual; so do
  `rule.end` and `message.avatar`. `overlay` with `side` docks as a sheet
  (`left`, `right`) or a drawer (`bottom`).
- `<select>`, `<switch>`, `<slider>`, `<otp>`, `<calendar>`, `<date>`,
  `<pager>`, `<breadcrumb>`, `<nav>`, `<menu>` and `<menubar>` send `change`,
  `select`, `toggle` or `action` as their kind says; `<field>` labels its
  control children and `<buttons>` groups `button` children. `actions.menu`
  takes the same rows as `<menu items>`: ids, `'-'`, `{ header }` or
  `{ id, label?, icon?, keys?, checked?, radio?, disabled?, danger?, items? }`.
- A span's `cite` (`span('text', '', { cite: { href, title } })` or
  `<span cite={…}>`) marks the run with its sources; `<text cite="note">`
  picks the look, and `plain()` writes `[1]` after each cited run.

## Design rules

Views built from kinds and tones look like Tern's own; the full guide is
[Designing views](https://docs.stencil.so/tern/guides/design.html).

- Open apps with `font: 'sans'`; machine strings (versions, paths, numbers)
  go in `mono` columns or `mono`/`code`/`path` span tokens.
- Use `button`, `table`, `stats`, `kv`, `tabs` and `card`s, not `html.*`
  plus CSS; write no stylesheet for looks the kinds already have.
- A page, not a pile of cards: a head row (`row justify="between"`: a `text`
  with the name in the `title` span token and muted meta, status badges and
  global buttons on the right), an optional `stats` strip, underline `tabs`
  when there are more than two views (Tab/⇧Tab switch; render only the
  active one), then the view: a full-width table, beside a detail `card`
  (`min={{ w: '44ch' }} max={{ w: '56ch' }}`) for master/detail. Cards hold
  objects (a selected item, a confirm strip), never a frame around a table
  or another card. The dock holds the confirm strip and a key legend.
- Color carries meaning only: `tone` and span tokens, never custom colors.
- One `primary` action per view: with a detail card it sits in the card
  head's `end` and table rows carry no buttons; without one, only the
  selected row's `sm` button cell is primary. Destructive actions ask first
  in a `card tone="error"` strip with `quiet` and `danger` buttons.
- Tables have exactly one `grow` column, the main text one; numbers are
  `align: 'end'` + `mono`; spark, meter and badge columns never grow.
  Narrow tables hide the lowest `priority` columns first and a missing
  `priority` is 0: give the `grow` column the highest, numbers next,
  sparks, meters and secondary columns the lowest. Show `keys` on buttons
  that have shortcuts.
- Design all four states: loading, `empty`, error and success.

## Quick start

```tsx
import { connect } from '@stencil-hq/tern';

const session = await connect({ app: 'deploy' });
if (session) {
   const surface = session.open({ mode: 'screen', font: 'sans' });
   surface.render(
      <card head="Deploy" icon="rocket" meta="api-gateway" status="running">
         <md>{'Uploading **api-gateway**'}</md>
         <progress value={0.4} label="40%" />
         <button
            variant="danger"
            keys={['esc']}
            onClick={() => surface.render(<md>{'Cancelled'}</md>)}
         >
            Cancel
         </button>
      </card>
   );
   for await (const input of session) {
      if (input.type === 'key' && input.key.name === 'escape') break;
   }
   await session.close();
}
```

`await using session = await connect(…)` closes the session at the end of
the block; `Surface` is async-disposable too.

## Layers

Each layer is usable without the ones above it.

| Layer | Module | What |
| --- | --- | --- |
| Wire | `wire.ts` | Constants (`KINDS`, `TEXT_KINDS`, features, limits), typed messages both ways, `encodeMessage`/`encodeJson` (chunking at the APC limit), `encodeBlob`, `decodeEvent`/`decodeReply` |
| Input | `input.ts`, `keys.ts` | `InputParser` splits replies, events and DA1 answers out of the pty input; `KeyDecoder` turns key bytes into `Key`s (legacy xterm, kitty `CSI u`, bracketed paste) |
| Nodes | `nodes.ts`, `props.ts`, `ui.ts`, `jsx-runtime.ts` | `Node`, `span()`, typed props by kind, `ui.<kind>(props, …children)`, `html.<tag>`, untyped `node(kind, props, children)`, TSX |
| Reconcile | `reconcile.ts` | `View.from(view)` and `view.ops(next, surface)`: Tern's plugin diff with ids derived from keys and positions |
| Session | `session.ts`, `surface.ts` | `connect`, raw mode, modes, surfaces, credit flow control, blobs, input loop, recording, clean exit |
| Helpers | `print.ts`, `ask.ts`, `plain.ts` | `print`, `ask`, `plain` |

### Views

`surface.render(view)` takes:

- a node, a list of nodes or a fragment: the whole view, shown as `main`'s
  children under a `col` region root;
- `{ main?, dock?, layer? }`: a node given for a region is that region's
  root itself (Tern ignores a region root's own kind and props, so make it a
  container such as `<col>`); a list given for a region is wrapped in a `col`.

A child's id is `<parent id>.<key>`, its `key` prop or else its index; two
siblings with one id reject the whole view with a `ViewError` before
anything is sent.

### Flow control

Frames number from 1 per surface, at most `credits` unacknowledged. A
`render` while blocked keeps only the newest view; view ops (`focus`,
`reveal`, `scroll`, `settle`, `suspend`, `resume`) and raw `send(ops)` wait
with it. When an ack returns credit, one frame carries the difference from
the last view sent, then the queued ops. `listen: false` surfaces send at
once. `surface.close()` sends whatever is still pending as one last frame
(credit or not), then `x`.

Frames commit their sequence, sent view and queued ops only after writing
successfully. A synchronous write failure throws; an ack-triggered failure
rejects the next input read. Closing still attempts `x`, resets modes and
restores the tty after the 50 ms drain if output fails. Raw-mode ownership
and exit/SIGINT/SIGTERM/SIGHUP cleanup are armed before the handshake; a
second session cannot acquire the same input.

`open({closable: true})` tells Tern closing the pane loses nothing: while the
surface is open, the pane closes without the "a program is running" prompt.

`open({id, adopt: true})` reuses this session's last-sent view (without dock
or layer) and frame sequence for a closed surface. For an unknown id, the
first frame deletes `main` before adding the new view.

## Handlers

Nodes carry handlers instead of naming actions by hand:

| Prop | Runs on |
| --- | --- |
| `onClick`, `onDblClick` | the node's click/double-click action (sets `actions.click`/`dblclick` to `"click"`/`"dblclick"` unless named) |
| `onMenu={{ rerun: fn }}` | a context-menu pick (names join `actions.menu`) |
| `onAction={{ name: fn }}` | `action` events with that `act` |
| `onToggle`, `onSelect`, `onActivate`, `onChange`, `onFocus`, `onEdit`, `onUndo`, `onSend` | those events |

`toggle` carries `collapsed` (a fold) or `open` (a menu, a select, a trace
step, with the menu or step as `item`). `action` and `change` carry the
kind's `value` as JSON: a string, a number, a pair, ids or an object of
answers; a menu's check rows add `checked`, a message reaction `on`.

Events route by `sf` and `id` to the node with that id in the view last
rendered (a list's `select`/`activate` go to the list). Handlers run before
anything is yielded; a handled event is consumed, everything else reaches
`for await (const input of session)` as `{ type: 'event', event }`, and keys
as `{ type: 'key', key }`. A handler that throws rejects the next read.

Named click/double-click actions such as `sort=name` match both the event's
`act` and `value`. `open({forwardActions: ['submit']})` also yields those
actions after their node handlers run; `ask` uses this so a handled submit
still answers the question.

Input is asynchronous (no caller poll timeout). Undecided prefixes flush
after 30 ms without input. Incomplete key sequences are dropped, but a
bracketed paste waits for its terminator across idle flushes. Printable
Unicode is preserved; unmapped kitty functional codes are dropped.

### Keys

A `Key` is `{ name, text?, ctrl, alt, shift, meta }` (`meta` is Cmd on
macOS, Super elsewhere). Match on `name`; read `text` only for what the key
types:

| Key | `name` | `text` |
| --- | --- | --- |
| Letters | lowercase: `'a'` (Shift+A is `'a'` with `shift: true`) | `'a'`, `'A'` |
| Digits, punctuation, other characters | the character: `'1'`, `'+'`, `'é'` | the same |
| Space | `'space'` | `' '` |
| Named keys | `'enter'`, `'escape'`, `'tab'`, `'backspace'`, `'up'`, `'down'`, `'left'`, `'right'`, `'home'`, `'end'`, `'page_up'`, `'page_down'`, `'insert'`, `'delete'`, `'f1'`… | none |
| Bracketed paste | `'paste'` | the pasted text |

With Ctrl, Alt or Meta held a key has no `text` (`Ctrl+C` is
`{ name: 'c', ctrl: true }`).

## Helpers

- `print(view, { css?, font?, fallback? })`: a static view kept in the scrollback
  (a `flow` surface with `listen: false`, one frame, closed with `keep`).
  Outside Tern, or when stdout isn't a tty, it writes `plain(view)` or
  `fallback`.
- `ask(view, { css?, font?, submit? = 'submit' })`: shows a form and resolves to
  `{ type: 'answer', id, act, values }` on the first `submit` action, `null`
  on Escape, Ctrl+C or Ctrl+D, or `{ type: 'unsupported' }` without TSP so
  the program can ask its own way.
- `plain(view, cols?)`: readable text without escapes, for fallbacks. It
  prints a `row`'s children on one line, so build fallback-friendly views
  (a `col` of cards) or pass `fallback` to `print`.
- `bar(fraction?, width = 20)`: `[####----] 50%`, the plain form of a
  progress bar (`[--------]` when unknown).
- `css`: a tagged template returning the stylesheet as written, for editor
  highlighting.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every message, both ways, as JSONL (`surface-play` replays it) |
| `TMUX`, `TMUX_PANE` | Inside tmux: probe only when Tern draws the session (`@tern-tsp`), with the pane's `allow-set-title` off while connected |
| `STY`, `ZELLIJ` | Inside screen or zellij, don't probe |

## Examples

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

```sh
bun examples/table.tsx      # print: the directory as a table
bun examples/ask.tsx        # ask: "Which size?"
bun examples/progress.tsx   # a flow surface with a running card
bun examples/chat.tsx       # an inline surface with an editor in the dock
bun examples/dashboard.tsx  # a sans screen app: head row, stats, tabbed views, table + detail
```

## Development

```sh
bun test                    # unit and session tests
bun run typecheck
bun run build               # dist/ for Node
```
