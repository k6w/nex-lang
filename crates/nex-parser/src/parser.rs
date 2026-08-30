use crate::ast::*;
use std::collections::HashMap;

/// A saved scanner position, used for lookahead and for pointing an error at
/// the token that *opened* a construct rather than at the byte where scanning
/// finally gave up.
#[derive(Debug, Clone, Copy)]
struct Mark {
    position: usize,
    line: usize,
    column: usize,
}

/// Parser state.
///
/// The input is held as a `Vec<char>` so that scanning is O(1) per character;
/// indexing a `&str` with `chars().nth()` made parsing quadratic in the length
/// of the document.
struct Parser {
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.chars.get(self.position).copied()
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.position + 1).copied()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }

    fn mark(&self) -> Mark {
        Mark {
            position: self.position,
            line: self.line,
            column: self.column,
        }
    }

    /// Rewind to a mark. Line and column travel with the position, so
    /// backtracking never corrupts the reported location of a later error.
    fn restore(&mut self, mark: Mark) {
        self.position = mark.position;
        self.line = mark.line;
        self.column = mark.column;
    }

    fn slice_from(&self, start: usize) -> String {
        self.chars[start..self.position].iter().collect()
    }

    fn error(&self, message: impl Into<String>) -> ParseError {
        ParseError::new(self.line, self.column, message.into())
    }

    fn error_at(&self, mark: Mark, message: impl Into<String>) -> ParseError {
        ParseError::new(mark.line, mark.column, message.into())
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '#' || (ch == '/' && self.peek_char() == Some('/')) {
                while let Some(ch) = self.current_char() {
                    if ch == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.mark();
        let mut has_dot = false;
        let mut has_e = false;

        if let Some('-') = self.current_char() {
            self.advance();
        }

        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() {
                self.advance();
            } else if ch == '.' && !has_dot && !has_e {
                has_dot = true;
                self.advance();
            } else if (ch == 'e' || ch == 'E') && !has_e {
                has_e = true;
                self.advance();
                if let Some(sign) = self.current_char() {
                    if sign == '+' || sign == '-' {
                        self.advance();
                    }
                }
            } else {
                break;
            }
        }

        let num_str = self.slice_from(start.position);

        if has_dot || has_e {
            num_str
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| self.error_at(start, format!("Invalid float: {}", num_str)))
        } else {
            num_str
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| self.error_at(start, format!("Invalid integer: {}", num_str)))
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        let open = self.mark();
        if self.current_char() != Some('"') {
            return Err(self.error("Expected string"));
        }
        self.advance();

        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                let Some(esc) = self.current_char() else {
                    return Err(self.error_at(open, "Unterminated string"));
                };
                let actual = match esc {
                    '"' => '"',
                    '\\' => '\\',
                    '/' => '/',
                    'b' => '\x08',
                    'f' => '\x0c',
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    'u' => {
                        self.advance();
                        let mut code = 0u32;
                        for _ in 0..4 {
                            let Some(digit) = self.current_char() else {
                                return Err(self.error("Incomplete unicode escape"));
                            };
                            let Some(d) = digit.to_digit(16) else {
                                return Err(self.error("Invalid unicode escape"));
                            };
                            code = code * 16 + d;
                            self.advance();
                        }
                        match char::from_u32(code) {
                            Some(c) => c,
                            None => return Err(self.error("Invalid unicode code point")),
                        }
                    }
                    _ => return Err(self.error(format!("Invalid escape sequence: \\{}", esc))),
                };
                result.push(actual);
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(self.error_at(open, "Unterminated string"))
    }

    fn parse_symbol(&mut self) -> Result<String, ParseError> {
        let start = self.position;

        match self.current_char() {
            Some(ch) if ch.is_alphabetic() || ch == '_' => self.advance(),
            Some(_) => return Err(self.error("Invalid symbol start")),
            None => return Err(self.error("Unexpected end of input")),
        }

        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Ok(self.slice_from(start))
    }

    /// `[ value* ]` -- the only list syntax. Parentheses always mean an object.
    fn parse_list(&mut self) -> Result<Vec<Value>, ParseError> {
        let open = self.mark();
        self.advance(); // consume '['

        let mut items = Vec::new();
        loop {
            self.skip_whitespace();
            match self.current_char() {
                Some(']') => {
                    self.advance();
                    return Ok(items);
                }
                None => return Err(self.error_at(open, "Unclosed '[': this list is never closed")),
                _ => items.push(self.parse_value()?),
            }
        }
    }

    /// `( (key value)* )` -- the body shared by named and anonymous objects.
    fn parse_fields(&mut self) -> Result<HashMap<String, Value>, ParseError> {
        let open = self.mark();
        self.advance(); // consume '('

        let mut fields = HashMap::new();
        loop {
            self.skip_whitespace();
            match self.current_char() {
                Some(')') => {
                    self.advance();
                    return Ok(fields);
                }
                None => {
                    return Err(self.error_at(open, "Unclosed '(': this object is never closed"))
                }
                _ => {}
            }

            let key = if self.current_char() == Some('"') {
                self.parse_string()?
            } else {
                self.parse_symbol()?
            };

            self.skip_whitespace();
            if self.current_char() == Some(')') {
                return Err(self.error(format!("Field '{}' has a key but no value", key)));
            }

            let value = self.parse_value()?;
            fields.insert(key, value);
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.skip_whitespace();

        match self.current_char() {
            Some('"') => self.parse_string().map(Value::String),
            Some('[') => self.parse_list().map(Value::List),
            Some('(') => self
                .parse_fields()
                .map(|fields| Value::Object { name: None, fields }),
            Some(ch) if ch.is_ascii_digit() || ch == '-' => self.parse_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let symbol = self.parse_symbol()?;
                match symbol.as_str() {
                    "true" => Ok(Value::Bool(true)),
                    "false" => Ok(Value::Bool(false)),
                    "null" => Ok(Value::Null),
                    _ => {
                        // A '(' after the symbol makes it a type name, as in
                        // `server tcp(host "x")`. Otherwise it is a bare symbol
                        // and the whitespace we consumed to find out is given back.
                        let after_symbol = self.mark();
                        self.skip_whitespace();
                        if self.current_char() == Some('(') {
                            let fields = self.parse_fields()?;
                            Ok(Value::Object {
                                name: Some(symbol),
                                fields,
                            })
                        } else {
                            self.restore(after_symbol);
                            Ok(Value::Symbol(symbol))
                        }
                    }
                }
            }
            Some(')') => Err(self.error("Unexpected ')': no object is open here")),
            Some(']') => Err(self.error("Unexpected ']': no list is open here")),
            Some(ch) => Err(self.error(format!("Unexpected character: {}", ch))),
            None => Err(self.error("Unexpected end of input")),
        }
    }
}

/// Parse a complete nex document. A document is exactly one value.
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input);
    let result = parser.parse_value()?;
    parser.skip_whitespace();

    if parser.position < parser.chars.len() {
        return Err(parser.error("Unexpected content after the document's root value"));
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_null() {
        assert_eq!(parse("null").unwrap(), Value::Null);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
        assert_eq!(parse("false").unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_parse_int() {
        assert_eq!(parse("42").unwrap(), Value::Int(42));
        assert_eq!(parse("-123").unwrap(), Value::Int(-123));
    }

    #[test]
    fn test_parse_float() {
        assert_eq!(parse("2.75").unwrap(), Value::Float(2.75));
        assert_eq!(parse("-0.5").unwrap(), Value::Float(-0.5));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(
            parse("\"hello\"").unwrap(),
            Value::String("hello".to_string())
        );
        assert_eq!(
            parse("\"hello\\nworld\"").unwrap(),
            Value::String("hello\nworld".to_string())
        );
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse("foo").unwrap(), Value::Symbol("foo".to_string()));
        assert_eq!(
            parse("bar_baz").unwrap(),
            Value::Symbol("bar_baz".to_string())
        );
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse("[]").unwrap(), Value::List(vec![]));
        assert_eq!(
            parse("[1 2 3]").unwrap(),
            Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)])
        );
    }

    fn fields(pairs: &[(&str, Value)]) -> HashMap<String, Value> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    fn field_of(doc: &str, key: &str) -> Value {
        match parse(doc).unwrap() {
            Value::Object { fields, .. } => fields.get(key).unwrap().clone(),
            other => panic!("expected an object, got {}", other.type_name()),
        }
    }

    #[test]
    fn bare_parens_are_an_anonymous_object() {
        assert_eq!(
            parse("(a 1)").unwrap(),
            Value::Object {
                name: None,
                fields: fields(&[("a", Value::Int(1))])
            }
        );
        assert_eq!(
            parse("()").unwrap(),
            Value::Object {
                name: None,
                fields: HashMap::new()
            }
        );
    }

    #[test]
    fn key_paren_nests_an_anonymous_object() {
        assert_eq!(
            field_of(r#"app(server(host "h" port 80))"#, "server"),
            Value::Object {
                name: None,
                fields: fields(&[
                    ("host", Value::String("h".to_string())),
                    ("port", Value::Int(80)),
                ]),
            }
        );
    }

    #[test]
    fn key_name_paren_nests_a_named_object() {
        assert_eq!(
            field_of(r#"app(server tcp(host "h"))"#, "server"),
            Value::Object {
                name: Some("tcp".to_string()),
                fields: fields(&[("host", Value::String("h".to_string()))]),
            }
        );
    }

    #[test]
    fn brackets_are_the_only_list_syntax() {
        assert_eq!(
            field_of("app(tags[cli parser lsp])", "tags"),
            Value::List(vec![
                Value::Symbol("cli".to_string()),
                Value::Symbol("parser".to_string()),
                Value::Symbol("lsp".to_string()),
            ])
        );
    }

    #[test]
    fn odd_field_count_is_an_error() {
        assert!(parse("app(a 1 b)").is_err());
    }

    #[test]
    fn error_points_at_the_opening_quote_of_an_unterminated_string() {
        let err = parse("app(\n  name \"oops\n)\n").unwrap_err();
        assert_eq!((err.line, err.column), (2, 8), "got: {}", err);
    }

    #[test]
    fn error_position_survives_nesting() {
        let src = "app(\n  server tcp(\n    host \"x\"\n    port @\n  )\n)\n";
        let err = parse(src).unwrap_err();
        assert_eq!((err.line, err.column), (4, 10), "got: {}", err);
    }

    #[test]
    fn unclosed_delimiter_points_at_the_opening_delimiter() {
        let err = parse("app(\n  tags[a b\n").unwrap_err();
        assert_eq!((err.line, err.column), (2, 7), "got: {}", err);
    }

    #[test]
    fn test_parse_object() {
        let mut fields = HashMap::new();
        fields.insert("key".to_string(), Value::String("value".to_string()));
        assert_eq!(
            parse("obj(key \"value\")").unwrap(),
            Value::Object {
                name: Some("obj".to_string()),
                fields
            }
        );
    }
}
