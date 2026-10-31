# Tern SDK

Build for [Tern](https://stencil.so/tern): libraries for programs that draw
native UI in a terminal pane, and example plugins that extend Tern itself.

## Surface Protocol SDKs

A program describes its UI as a tree of nodes over the
[Tern Surface Protocol](https://docs.stencil.so/tern/protocol/index.html);
Tern lays it out, draws and animates it natively. Outside Tern the same
program prints plain text.

Before you build a UI, read
[Designing Views](https://docs.stencil.so/tern/guides/design.html): an app as
a page (a head row, a `stats` strip, `tabs` between views, a full-width
selectable `table` beside the selected item's `card`), which kind does which
job, the sans face for apps, and the mistakes that make a view look like a
web page. Every SDK's `dashboard` example follows it.

| Language | Path | Package |
| --- | --- | --- |
| Rust | [`rust/`](rust) | crate `tern-sdk` |
| Python | [`python/`](python) | `tern-sdk` |
| Go | [`go/tern/`](go/tern) | `github.com/stencil-hq/tern-sdk/go/tern` |
| TypeScript | [`typescript/`](typescript) | `@stencil-hq/tern` |

## Plugins

[`plugins/examples/`](plugins/examples) holds example plugins, one
installable folder each; `plugins/tern.d.luau` is the API they type-check
against. See the [plugin docs](https://docs.stencil.so/tern/).

```sh
tern plugin install github.com/stencil-hq/tern-sdk/plugins/examples/jsonx
```

[MIT](LICENSE).
