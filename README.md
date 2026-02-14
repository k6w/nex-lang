# nex

nex is a human-readable data serialization language designed for configuration files, data exchange, and structured data representation. it emphasizes simplicity, readability, and toolability.

## design goals

nex minimizes syntactic noise while preserving unambiguous structure. it is optimized for both human readability and machine generation, especially large language models.

## non-goals

nex is not a programming language. nex does not support expressions, macros, or includes. nex avoids indentation-significant semantics.

## canonical formatting

nex has a single canonical formatting style. any valid nex document can be formatted into a stable representation. this enables clean diffs, reliable tooling, and deterministic ai output.

## error handling

nex errors are localized, descriptive, and recoverable. parsers should continue after errors when possible to enable editor diagnostics and partial analysis.

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

nex uses a minimal, punctuation-light syntax.

### value types

- null
- true / false
- integers: 42, -123
- floats: 3.14, -0.5
- strings: "hello world"
- symbols: unquoted_identifier

### lists

items are space or newline separated

```nex
colors(red green blue)
```

### objects

objects are introduced by an identifier followed by parentheses.

```nex
user(
  name Alex
  age 21
  active true
)
```

## example

```nex
config(
  name "my app"
  version "1.0.0"
  debug true
  settings(
    timeout 30
    features(logging caching auth)
  )
)
```

## schemas

nex supports optional schema definitions for validation and tooling. schemas enable type checking, autocomplete, and documentation in editors.

## versioning

nex follows semantic versioning. the v1 syntax is guaranteed to remain stable.

## encoding

nex files are utf-8 encoded. both lf and crlf newlines are supported.

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