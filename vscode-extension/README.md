# nex language support

VS Code support for the [nex](https://github.com/k6w/nex-lang) data serialization language,
backed by the `nex-lsp` language server.

## features

| Capability | Status |
|---|---|
| Syntax highlighting | ✅ |
| Diagnostics on open and on change | ✅ live parse errors |
| Document formatting | ✅ whole-document, canonical |
| Document symbols | ✅ objects and fields in the outline |
| Hover | ⚠️ advertised, returns nothing yet |

## syntax

Brackets and parentheses are never interchangeable: `[` … `]` is always a list, `(` … `)` is
always an object, and a symbol immediately before `(` names the object's type.

```nex
# an object
config(
  name "my app"
  version "1.0.0"
  debug true
)

# a list
config(colors[red green blue])

# an anonymous nested object, and a type-tagged one
app(
  limits(timeout 30)
  server tcp(
    host "0.0.0.0"
    port 8080
  )
)
```

Comments run from `#` or `//` to the end of the line.

## building

```bash
npm install
npm run compile
```

The client launches the language server from `bin/`. Build it with `cargo build --release`
from the repository root and point the extension at `target/release/nex lsp`, or drop the
compiled server binary into `bin/`.

## license

MIT
