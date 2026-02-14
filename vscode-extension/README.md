# nex language support

vscode extension for nex data serialization language.

## features

- syntax highlighting
- diagnostics
- formatting
- document symbols
- hover

## syntax

nex uses a minimal, parentheses-based syntax:

```nex
# Objects
config(
  name "my app"
  version "1.0.0"
  debug true
)

# Lists
colors(red green blue)

# Nested structures
user(
  name "john"
  settings(
    theme "dark"
    notifications true
  )
)
```

## installation

install from vsix or marketplace.

## usage

create .nex files and enjoy full language support.