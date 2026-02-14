# NEX v1 Specification

## Overview

NEX is a human-readable data serialization language designed for configuration files, data exchange, and structured data representation. It emphasizes simplicity, readability, and toolability.

## Syntax

NEX documents consist of values. The top-level value can be any valid NEX value.

### Values

NEX supports the following value types:

1. **Null**: `null`
2. **Boolean**: `true` or `false`
3. **Integer**: Decimal numbers without decimal point, e.g., `42`, `-123`
4. **Float**: Decimal numbers with decimal point, e.g., `3.14`, `-0.5`
5. **String**: Double-quoted strings, e.g., `"hello world"`
6. **Symbol**: Unquoted identifiers, e.g., `foo`, `bar_baz`
7. **List**: Comma-separated values in square brackets, e.g., `[1, 2, "three"]`
8. **Object**: Named or anonymous collections of key-value pairs in curly braces, e.g., `config { key: "value" }` or `{ key: "value" }`

### Objects

Objects have a name followed by curly braces containing fields:

```
object_name {
  field1: value1,
  field2: value2
}
```

Field keys are symbols or strings. Values can be any NEX value.

### Lists

Lists are ordered collections:

```
[ item1, item2, item3 ]
```

Items can be any NEX value.

### Symbols

Symbols are identifiers that start with a letter or underscore, followed by letters, digits, or underscores. They do not need quotes.

### Strings

Strings are enclosed in double quotes. Standard escape sequences are supported: `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX`.

### Numbers

- Integers: `0`, `123`, `-456`
- Floats: `0.0`, `3.14159`, `-2.5`, `1e10`, `2.5E-3`

### Comments

Single-line comments start with `#` or `//`:

```
# This is a comment
config {
  // Another comment
  key: "value"
}
```

### Whitespace

Whitespace (spaces, tabs, newlines) is ignored except to separate tokens.

## Grammar

```
value ::= null | bool | number | string | symbol | list | object

null ::= "null"

bool ::= "true" | "false"

number ::= integer | float

integer ::= ["-"] digit+

float ::= ["-"] digit+ "." digit+ [["e"|"E"] ["+"|"-"] digit+]

string ::= '"' (char | escape)* '"'

symbol ::= letter (letter | digit | "_")*

list ::= "[" [value ("," value)*] "]"

object ::= [symbol] "{" [field ("," field)*] "}"

field ::= (symbol | string) ":" value
```

## Examples

### Simple object
```
config {
  name: "My App",
  version: "1.0.0",
  debug: true
}
```

### Nested structures
```
app {
  database: db {
    host: "localhost",
    port: 5432,
    credentials: {
      user: "admin",
      pass: "secret"
    }
  },
  features: [logging, caching, auth]
}
```

### Mixed types
```
data {
  null_value: null,
  boolean: false,
  integer: 42,
  float: 3.14159,
  string: "hello",
  symbol: unquoted,
  list: [1, "two", true],
  nested: inner {
    key: "value"
  }
}
```

## Implementation Notes

- Parsers should be case-sensitive
- Trailing commas in lists and objects are allowed
- Empty lists `[]` and empty objects `name {}` are valid
- Symbols cannot be reserved words: `null`, `true`, `false`