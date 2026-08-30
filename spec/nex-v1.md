# nex v1 specification

## overview

nex is a human-readable data serialization language for configuration files, data exchange,
and structured data representation. It emphasises simplicity, readability, and toolability.

## documents

A nex document is **exactly one value**, normally an object. Content after that value is an
error.

## values

| Kind | Syntax | Example |
|---|---|---|
| null | `null` | `null` |
| boolean | `true` or `false` | `true` |
| integer | optional `-`, then digits | `42`, `-123` |
| float | digits with `.` and/or an exponent | `3.14`, `-0.5`, `2.5e-3` |
| string | double-quoted | `"hello world"` |
| symbol | bare identifier | `foo`, `bar_baz` |
| list | `[` values `]` | `[1 2 "three"]` |
| object | optional name, then `(` fields `)` | `config(name "app")`, `(name "app")` |

## the bracket rule

Square brackets and parentheses are never interchangeable:

- `[` … `]` is **always a list** — an ordered sequence of values.
- `(` … `)` is **always an object** — a sequence of key/value pairs.

A symbol immediately before `(` names the object's type:

```
tags[cli parser lsp]          # list of three symbols
limits(timeout 30)            # anonymous object
server tcp(host "0.0.0.0")    # object of type tcp
```

Inside an object, each field is a key followed by a value. Because the value of a field is
just a value, all three forms above appear in field position without any extra syntax.

## objects

```
object_name(
  field1 value1
  field2 value2
)
```

Field keys are symbols or quoted strings. Values may be any nex value. An object's name is
optional; `(field value)` is an anonymous object. Empty objects — `()` and `name()` — are
valid.

Field order is not significant. Canonical output sorts fields by key, and a repeated key
keeps its last value.

## lists

```
[item1 item2 item3]
```

Items may be any nex value, and need not share a type. The empty list is `[]`.

## symbols

Symbols start with a letter or underscore, followed by letters, digits, or underscores. They
need no quotes. The words `null`, `true`, and `false` are reserved and are never symbols.

## strings

Strings are enclosed in double quotes and support the escape sequences `\"`, `\\`, `\/`,
`\b`, `\f`, `\n`, `\r`, `\t`, and `\uXXXX`.

## numbers

- integers: `0`, `123`, `-456`
- floats: `0.0`, `3.14159`, `-2.5`, `1e10`, `2.5e-3`

Canonical output always writes a float with a decimal point or exponent, so a float never
reads back as an integer.

## comments

Comments run from `#` or `//` to the end of the line.

```
# this is a comment
config(
  // another comment
  key "value"
)
```

## whitespace

Whitespace — spaces, tabs, and newlines — separates tokens and carries no other meaning.
Layout is decided entirely by the formatter.

## grammar

```
document ::= value

value    ::= null | bool | number | string | symbol | list | object

null     ::= "null"
bool     ::= "true" | "false"
number   ::= integer | float
integer  ::= ["-"] digit+
float    ::= ["-"] digit+ ["." digit+] [("e" | "E") ["+" | "-"] digit+]
string   ::= '"' (char | escape)* '"'
symbol   ::= letter (letter | digit | "_")*

list     ::= "[" value* "]"
object   ::= [symbol] "(" field* ")"
field    ::= (symbol | string) value

comment  ::= ("#" | "//") (any except newline)*
```

## examples

### simple object

```
config(
  name "my app"
  version "1.0.0"
  debug true
)
```

### nested structures

```
app(
  database db(
    host "localhost"
    port 5432
    credentials(
      user "admin"
      pass "secret"
    )
  )
  features[logging caching auth]
)
```

`database` is a field whose value is an object of type `db`; `credentials` is a field whose
value is an anonymous object; `features` is a field whose value is a list.

### mixed types

```
data(
  null_value null
  boolean false
  integer 42
  float 3.14159
  string "hello"
  symbol unquoted
  list [1 "two" true]
  nested inner(
    key "value"
  )
)
```

## errors

A parse error carries a 1-based line and column. Positions point at the token that opened the
failing construct — an unterminated string reports its opening quote, an unclosed `(` reports
that parenthesis — rather than at the point where scanning stopped.

## implementation notes

- parsing is case-sensitive
- empty lists `[]`, empty objects `()`, and empty named objects `name()` are valid
- symbols cannot be the reserved words `null`, `true`, or `false`
- documents are utf-8; both lf and crlf newlines are accepted

## json mapping

| nex | json |
|---|---|
| null, bool, int, float, string | the same |
| symbol | string |
| list | array |
| anonymous object | object |
| named object | object with the name under the reserved key `_type` |

The mapping from nex to json is lossy in one direction: a symbol and a quoted string both
become json strings, so converting back produces a string.
