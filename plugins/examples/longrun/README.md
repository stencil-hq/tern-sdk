# longrun

Notices commands that keep you waiting. When a command line runs for 15
seconds or longer, a toast says so when it ends ("cargo build --release
finished in 1m 12s", or "failed after" with the exit status), and the
command joins a history of the last 50 slow commands.

- **Slow Commands block**: the commands running now with live elapsed
  times, then the history (command, directory, duration, status, when).
  `↑`/`↓` (or `j`/`k`) select, `Enter` copies the command line, `c` clears
  the history, `+`/`-` move the threshold by 5 seconds.
- **Show slow commands**: a palette command, bound to `cmd+alt+shift+l`
  (`ctrl+alt+shift+l` off macOS), that focuses the block or opens it in a
  new tab.
- **Status segment**: once the focused pane's command has run for a
  second, the status line shows its program and running time
  ("cargo · 42s"), ticking once a second until it ends; clicking it opens
  the block.

Shell integration must be on: the timings come from the shell's prompt
marks.

## Install

From where you cloned the SDK (`git clone https://github.com/stencil-hq/tern-sdk`):

```sh
tern plugin install tern-sdk/plugins/examples/longrun
```

or, to load it in place and reload on every save:

```sh
tern plugin link tern-sdk/plugins/examples/longrun
```

The threshold is stored in the plugin's `kv.json` as `threshold_ms`; the
block's `+` and `-` keys change it. Clearing history or changing the threshold
refreshes every open Slow Commands block on that host, even while idle.

The status clock refreshes focus and command liveness from window snapshots
on events and on its one-second ticks. The next command after a plugin reload
works without refocusing; a shell that exits without its next prompt mark
loses its segment and stops the clock. No timer runs for an idle focused pane.
Commands already running when the window half loads are not timed.

Walkthrough: <https://docs.stencil.so/tern/examples/longrun.html>.
