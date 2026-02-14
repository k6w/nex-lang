# nex v1 specification

## overview

nex is a human-readable data serialization language designed for configuration files, data exchange, and structured data representation. it emphasizes simplicity, readability, and toolability.

## syntax

nex documents consist of values. the top-level value can be any valid nex value.

### values

nex supports the following value types:

1. **null**: `null`
2. **boolean**: `true` or `false`
3. **integer**: decimal numbers without decimal point, e.g., `42`, `-123`
4. **float**: decimal numbers with decimal point, e.g., `3.14`, `-0.5`
5. **string**: double-quoted strings, e.g., `"hello world"`
6. **symbol**: unquoted identifiers, e.g., `foo`, `bar_baz`
7. **list**: space or newline separated values in parentheses, e.g., `(1 2 "three")`
8. **object**: named collections of key-value pairs in parentheses, e.g., `config(name "my app" version "1.0.0")`

### objects

objects have a name followed by parentheses containing fields:

```
object_name(
  field1 value1
  field2 value2
)
```

field keys are symbols or strings. values can be any nex value.

### lists

lists are ordered collections:

```
(item1 item2 item3)
```

or named:

```
colors(red green blue)
```

items can be any nex value.

### symbols

symbols are identifiers that start with a letter or underscore, followed by letters, digits, or underscores. they do not need quotes.

### strings

strings are enclosed in double quotes. standard escape sequences are supported: `\"`, `\\`, `\/`, `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX`.

### numbers

- integers: `0`, `123`, `-456`
- floats: `0.0`, `3.14159`, `-2.5`, `1e10`, `2.5e-3`

### comments

single-line comments start with `#` or `//`:

```
# this is a comment
config(
  // another comment
  key "value"
)
```

### whitespace

whitespace (spaces, tabs, newlines) is ignored except to separate tokens.

## grammar

```
value ::= null | bool | number | string | symbol | list | object

null ::= "null"

bool ::= "true" | "false"

number ::= integer | float

integer ::= ["-"] digit+

float ::= ["-"] digit+ "." digit+ [["e"|"e"] ["+"|"-"] digit+]

string ::= '"' (char | escape)* '"'

symbol ::= letter (letter | digit | "_")*

list ::= "(" [value]+ ")"

object ::= symbol "(" [field]+ ")"

field ::= (symbol | string) value
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
  features(logging caching auth)
)
```

### mixed types
```
data(
  null_value null
  boolean false
  integer 42
  float 3.14159
  string "hello"
  symbol unquoted
  list(1 "two" true)
  nested inner(
    key "value"
  )
)
```

## implementation notes

- parsers should be case-sensitive
- empty lists `()` and empty objects `name()` are valid
- symbols cannot be reserved words: `null`, `true`, `false`