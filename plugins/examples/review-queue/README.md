# review-queue

A block listing the open pull requests that request your review, read
from the GitHub CLI (`gh search prs --review-requested=@me --state=open`)
and refreshed every two minutes. Each row shows the repository and number,
the title, the author and how long the pull request has been open, oldest
first; drafts are hidden until you ask for them.

- `↑`/`↓` (or `k`/`j`), `Home`/`End`, `PageUp`/`PageDown` move.
- `Enter` (or `o`, or a double click) opens the pull request in the browser.
- `r` refreshes now; `d` shows or hides drafts (remembered across restarts).
- **Open review queue** in the palette, bound to `alt+shift+cmd+r`, opens the
  block beside the focused pane or focuses the one already open.

When `gh` is missing or not signed in, the block says so and how to fix
it; after a later failure it keeps the last list and notes the problem in
its dock.

Each running block keeps one polling chain. Restart cancels the previous
run's timer, releases its list, and ignores its pending process result,
even though the pane ID stays the same. Closing the pane retires the state
on the next process callback or two-minute tick. An already-started `gh`
finishes or hits the existing 30-second timeout; it cannot revive the old
chain.

## Install

The block runs `gh` on the machine that runs your panes, so install and
sign in to the GitHub CLI there (`gh auth login`). Then:

```sh
git clone https://github.com/stencil-hq/tern-sdk
tern plugin install tern-sdk/plugins/examples/review-queue
```

or, to load it in place and reload on every save:

```sh
tern plugin link tern-sdk/plugins/examples/review-queue
```

Walkthrough: <https://docs.stencil.so/tern/examples/review-queue.html>.
