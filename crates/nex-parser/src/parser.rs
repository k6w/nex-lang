use crate::ast::*;
use std::collections::HashMap;

/// Parser state
struct Parser<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.input.chars().nth(self.position)
    }

    fn peek_char(&self) -> Option<char> {
        self.input.chars().nth(self.position + 1)
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

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '#' || (ch == '/' && self.peek_char() == Some('/')) {
                // Skip comments
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
        let start_pos = self.position;
        let mut has_dot = false;
        let mut has_e = false;

        // Handle optional sign
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

        let num_str = &self.input[start_pos..self.position];

        if has_dot || has_e {
            match num_str.parse::<f64>() {
                Ok(f) => Ok(Value::Float(f)),
                Err(_) => Err(ParseError::new(self.line, self.column, format!("Invalid float: {}", num_str))),
            }
        } else {
            match num_str.parse::<i64>() {
                Ok(i) => Ok(Value::Int(i)),
                Err(_) => Err(ParseError::new(self.line, self.column, format!("Invalid integer: {}", num_str))),
            }
        }
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        if self.current_char() != Some('"') {
            return Err(ParseError::new(self.line, self.column, "Expected string".to_string()));
        }
        self.advance(); // skip opening quote

        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch == '"' {
                self.advance();
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                if let Some(esc) = self.current_char() {
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
                            // Unicode escape
                            self.advance();
                            let mut code = 0u32;
                            for _ in 0..4 {
                                if let Some(digit) = self.current_char() {
                                    if let Some(d) = digit.to_digit(16) {
                                        code = code * 16 + d;
                                        self.advance();
                                    } else {
                                        return Err(ParseError::new(self.line, self.column, "Invalid unicode escape".to_string()));
                                    }
                                } else {
                                    return Err(ParseError::new(self.line, self.column, "Incomplete unicode escape".to_string()));
                                }
                            }
                            match char::from_u32(code) {
                                Some(c) => c,
                                None => return Err(ParseError::new(self.line, self.column, "Invalid unicode code point".to_string())),
                            }
                        }
                        _ => return Err(ParseError::new(self.line, self.column, format!("Invalid escape sequence: \\{}", esc))),
                    };
                    result.push(actual);
                    self.advance();
                } else {
                    return Err(ParseError::new(self.line, self.column, "Incomplete escape sequence".to_string()));
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(ParseError::new(self.line, self.column, "Unterminated string".to_string()))
    }

    fn parse_symbol(&mut self) -> Result<String, ParseError> {
        let start_pos = self.position;

        if let Some(ch) = self.current_char() {
            if !ch.is_alphabetic() && ch != '_' {
                return Err(ParseError::new(self.line, self.column, "Invalid symbol start".to_string()));
            }
            self.advance();
        } else {
            return Err(ParseError::new(self.line, self.column, "Unexpected end of input".to_string()));
        }

        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Ok(self.input[start_pos..self.position].to_string())
    }

    fn parse_list(&mut self) -> Result<Vec<Value>, ParseError> {
        if self.current_char() != Some('[') {
            return Err(ParseError::new(self.line, self.column, "Expected '['".to_string()));
        }
        self.advance();
        self.skip_whitespace();

        let mut items = Vec::new();

        if self.current_char() == Some(']') {
            self.advance();
            return Ok(items);
        }

        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_whitespace();

            if self.current_char() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else if self.current_char() == Some(']') {
                self.advance();
                break;
            } else {
                return Err(ParseError::new(self.line, self.column, "Expected ',' or ']' in list".to_string()));
            }
        }

        Ok(items)
    }

    fn parse_object(&mut self) -> Result<(Option<String>, HashMap<String, Value>), ParseError> {
        let name = if self.current_char().map_or(false, |ch| ch.is_alphabetic() || ch == '_') {
            Some(self.parse_symbol()?)
        } else {
            None
        };

        self.skip_whitespace();

        if self.current_char() != Some('{') {
            return Err(ParseError::new(self.line, self.column, "Expected '{' after object name".to_string()));
        }
        self.advance();
        self.skip_whitespace();

        let mut fields = HashMap::new();

        if self.current_char() == Some('}') {
            self.advance();
            return Ok((name, fields));
        }

        loop {
            let key = if self.current_char() == Some('"') {
                self.parse_string()?
            } else {
                self.parse_symbol()?
            };

            self.skip_whitespace();

            if self.current_char() != Some(':') {
                return Err(ParseError::new(self.line, self.column, "Expected ':' after field key".to_string()));
            }
            self.advance();
            self.skip_whitespace();

            let value = self.parse_value()?;
            fields.insert(key, value);
            self.skip_whitespace();

            if self.current_char() == Some(',') {
                self.advance();
                self.skip_whitespace();
            } else if self.current_char() == Some('}') {
                self.advance();
                break;
            } else {
                return Err(ParseError::new(self.line, self.column, "Expected ',' or '}' in object".to_string()));
            }
        }

        Ok((name, fields))
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.skip_whitespace();

        match self.current_char() {
            Some('n') => {
                // Check for null
                if self.input[self.position..].starts_with("null") {
                    self.position += 4;
                    self.column += 4;
                    Ok(Value::Null)
                } else {
                    self.parse_symbol().map(Value::Symbol)
                }
            }
            Some('t') => {
                // Check for true
                if self.input[self.position..].starts_with("true") {
                    self.position += 4;
                    self.column += 4;
                    Ok(Value::Bool(true))
                } else {
                    self.parse_symbol().map(Value::Symbol)
                }
            }
            Some('f') => {
                // Check for false
                if self.input[self.position..].starts_with("false") {
                    self.position += 5;
                    self.column += 5;
                    Ok(Value::Bool(false))
                } else {
                    self.parse_symbol().map(Value::Symbol)
                }
            }
            Some('"') => self.parse_string().map(Value::String),
            Some('[') => self.parse_list().map(Value::List),
            Some('{') => {
                let (name, fields) = self.parse_object()?;
                Ok(Value::Object { name, fields })
            }
            Some(ch) if ch.is_ascii_digit() || ch == '-' => self.parse_number(),
            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                // Could be symbol or named object
                let start_pos = self.position;
                let symbol = self.parse_symbol()?;
                self.skip_whitespace();

                if self.current_char() == Some('{') {
                    // It's a named object
                    self.position = start_pos; // Reset position
                    self.line = 1; // Reset line/column tracking for simplicity
                    self.column = 1;
                    let (name, fields) = self.parse_object()?;
                    Ok(Value::Object { name, fields })
                } else {
                    Ok(Value::Symbol(symbol))
                }
            }
            Some(ch) => Err(ParseError::new(self.line, self.column, format!("Unexpected character: {}", ch))),
            None => Err(ParseError::new(self.line, self.column, "Unexpected end of input".to_string())),
        }
    }
}

pub fn parse(input: &str) -> Result<Value, ParseError> {
    let mut parser = Parser::new(input);
    let result = parser.parse_value()?;
    parser.skip_whitespace();

    if parser.position < input.len() {
        return Err(ParseError::new(parser.line, parser.column, "Unexpected content after value".to_string()));
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
        assert_eq!(parse("3.14").unwrap(), Value::Float(3.14));
        assert_eq!(parse("-0.5").unwrap(), Value::Float(-0.5));
    }

    #[test]
    fn test_parse_string() {
        assert_eq!(parse("\"hello\"").unwrap(), Value::String("hello".to_string()));
        assert_eq!(parse("\"hello\\nworld\"").unwrap(), Value::String("hello\nworld".to_string()));
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse("foo").unwrap(), Value::Symbol("foo".to_string()));
        assert_eq!(parse("bar_baz").unwrap(), Value::Symbol("bar_baz".to_string()));
    }

    #[test]
    fn test_parse_list() {
        assert_eq!(parse("[]").unwrap(), Value::List(vec![]));
        assert_eq!(parse("[1, 2, 3]").unwrap(), Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]));
    }

    #[test]
    fn test_parse_object() {
        let mut fields = HashMap::new();
        fields.insert("key".to_string(), Value::String("value".to_string()));
        assert_eq!(parse("obj { key: \"value\" }").unwrap(), Value::Object { name: Some("obj".to_string()), fields });
    }
}