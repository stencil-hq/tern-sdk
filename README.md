# Tern SDK

Build for [Tern](https://stencil.so/tern): libraries for programs that draw
native UI in a terminal pane, and example plugins that extend Tern itself.

## Surface Protocol SDKs

A program describes its UI as a tree of nodes over the
[Tern Surface Protocol](https://docs.stencil.so/tern/protocol/index.html);
Tern lays it out, draws and animates it natively. Outside Tern the same
program prints plain text.

| Language | Path | Package |
| --- | --- | --- |
| Rust | [`rust/`](rust) | crate `tern-sdk` |
| Python | [`python/`](python) | `tern-sdk` |
| Go | [`go/`](go) | `github.com/stencil-hq/tern-sdk/go` |
| TypeScript | [`typescript/`](typescript) | `@stencil-hq/tern` |

## Plugins

[`plugins/examples/`](plugins/examples) holds example plugins, one
installable folder each; `plugins/tern.d.luau` is the API they type-check
against. See the [plugin docs](https://docs.stencil.so/tern/).

```sh
tern plugin install github.com/stencil-hq/tern-sdk/plugins/examples/jsonx
```

[MIT](LICENSE).
