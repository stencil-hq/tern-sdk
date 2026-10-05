# dirvars

Gives each new shell the variables of the `.tern-env` files between its
directory and your home directory, **only after explicit approval of each
file's exact contents**. A file nearer the shell's directory
overrides a farther one; `TERN_DIRVARS` lists the files that applied,
farthest first, separated by `:`.

```sh
# ~/src/webapp/.tern-env
APP_ENV=development
DATABASE_URL=postgres://localhost:5432/${APP_ENV}
```

The format is one `KEY=VALUE` per line with `#` comments, an optional
`export ` prefix, bare, `'single-quoted'` (literal) or `"double-quoted"`
values, and `${NAME}` expansion of earlier keys. `sample.tern-env` shows
every form.

A shell already running keeps its environment. When it `cd`s into a
directory whose approved `.tern-env` files differ from the ones it started
with, a toast says so, once per directory; new panes there pick those
approved files up.

## Install

```sh
git clone https://github.com/stencil-hq/tern-sdk
tern plugin install tern-sdk/plugins/examples/dirvars
```

or, to load it in place and reload on every save:

```sh
tern plugin link tern-sdk/plugins/examples/dirvars
```

## Approve a project

After creating `.tern-env`, open **Directory Variables** from the block
palette in that project. Review the complete escaped contents and click
**Approve these exact bytes** for each file you trust. Open a new shell
pane there to import them. The host-only block also runs on a remote host;
approval is stored in that host's existing plugin KV, per path, with at
most 64 KiB of content per approval. An optional block argument selects an
absolute project directory.

Unknown, changed or unreadable content is denied. Any byte change requires
review and approval again, even if only a comment changed. Refresh the block
to review edits; a file changed after display cannot be approved by the old
button. **Revoke approval** stops future imports; running shells retain
their environment. Do not approve untrusted checkouts: startup variables
such as `ZDOTDIR` can cause shell code execution. There is no variable-name
denylist that substitutes for trust in the whole file.

## Limits

- Only shells Tern starts after the plugin loads get the variables.
- A shell's startup files run after the variables are set and can change
  them; a login shell rebuilds `PATH`.
- `${NAME}` falls back to the Tern daemon's environment, not the shell's.
- Variables named `TERN_*` are reserved and skipped. Reads are bounded to
  64 KiB before allocation; directories, FIFOs, devices and symlinks are
  rejected on Unix and Windows, including raced-in nonregular targets.
  Regular-file IO on slow/network disks can still take time.
  Parse problems are logged under `tern::plugin`.

Walkthrough: <https://docs.stencil.so/tern/examples/dirvars.html>.
