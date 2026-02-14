# nex

nex is a human-readable data serialization language designed for configuration files, data exchange, and structured data representation. it emphasizes simplicity, readability, and toolability.

## features

- **simple syntax**: easy to read and write
- **type-safe**: strong typing with clear semantics
- **toolable**: formatter, linter, and lsp support
- **json bridge**: convert to/from json seamlessly
- **fast**: hand-written parser with excellent performance

## installation

```bash
cargo install nex-cli
```

## usage

### check syntax
```bash
nex check config.nex
```

### format files
```bash
nex format config.nex
nex format --write config.nex  # modify in place
```

### convert to json
```bash
nex to-json config.nex
```

### convert from json
```bash
nex from-json config.json
```

### start lsp server
```bash
nex lsp
```

## syntax

nex supports the following value types:

- `null`
- `true` / `false`
- integers: `42`, `-123`
- floats: `3.14`, `-0.5`
- strings: `"hello world"`
- symbols: `unquoted_identifier`
- lists: `[item1, item2]`
- objects: `name { key: value }` or `{ key: value }`

### example

```nex
config {
  name: "my app",
  version: "1.0.0",
  debug: true,
  settings: {
    timeout: 30,
    features: [logging, caching, auth]
  }
}
```

## specification

see [spec/nex-v1.md](spec/nex-v1.md) for the complete language specification.

## architecture

nex is implemented as a rust workspace with the following crates:

- `nex-parser`: recursive descent parser
- `nex-formatter`: code formatter
- `nex-json`: json conversion bridge
- `nex-lsp`: language server protocol implementation
- `nex-cli`: command-line interface

## contributing

contributions are welcome! please see the issues for areas that need work.

## license

mit or apache-2.0