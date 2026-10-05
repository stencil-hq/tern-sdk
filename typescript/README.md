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
`list.max`, styled `tool.title` and `picker.title`). `agent.status` uses
`AgentStatus`: pending, running, done, failed, aborted, idle or parked.
Table meter cells use `{ meter: { value: 0.5 } }` (or `parts`, thresholds,
tone and title inside `meter`); picker action `disabled` accepts `true` or
a reason string.

## Quick start

```tsx
import { connect, css, html } from '@stencil-hq/tern';

const session = await connect({ app: 'deploy' });
if (session) {
   const surface = session.open({ mode: 'flow' });
   surface.stylesheet('main', css`.go { color: var(--sf-c-accent) }`);
   surface.render(
      <card head="Deploy" status="running">
         <md>{'Uploading **api-gateway**'}</md>
         <progress value={0.4} label="40%" />
         <html.button class="go" onClick={() => surface.render(<md>{'Cancelled'}</md>)}>
            Cancel
         </html.button>
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

## Helpers

- `print(view, { css?, fallback? })`: a static view kept in the scrollback
  (a `flow` surface with `listen: false`, one frame, closed with `keep`).
  Outside Tern, or when stdout isn't a tty, it writes `plain(view)` or
  `fallback`.
- `ask(view, { css?, submit? = 'submit' })`: shows a form and resolves to
  `{ type: 'answer', id, act, values }` on the first `submit` action, `null`
  on Escape, Ctrl+C or Ctrl+D, or `{ type: 'unsupported' }` without TSP so
  the program can ask its own way.
- `plain(view, cols?)`: readable text without escapes, for fallbacks.
- `css`: a tagged template returning the stylesheet as written, for editor
  highlighting.

## Environment

| Variable | Effect |
| --- | --- |
| `TERN_TSP=0` | Never speak TSP |
| `TERN_TSP_RECORD=<file>` | Append every message, both ways, as JSONL (`surface-play` replays it) |
| `TMUX`, `STY`, `ZELLIJ` | Inside a multiplexer, don't probe |

## Examples

The `ask` example reads the selected size only from the submit action's form
values, without tracking radio changes. Outside Tern it asks on the command line.

```sh
bun examples/table.tsx      # print: the directory as a table
bun examples/ask.tsx        # ask: "Which size?"
bun examples/progress.tsx   # a flow surface with a running card
bun examples/chat.tsx       # an inline surface with an editor in the dock
```

## Development

```sh
bun test                    # unit and session tests
bun run typecheck
bun run build               # dist/ for Node
```
