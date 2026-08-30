use nex_parser::ast::Value;
use std::collections::HashMap;

/// Format a nex value into its single canonical representation.
///
/// The canonical form is two-space indentation, one entry per line, and object
/// fields sorted by key. Formatting is idempotent, and re-parsing the result
/// yields a value equal to the input.
pub fn format(value: &Value) -> String {
    format_value(value, 0)
}

fn format_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => format_float(*f),
        Value::String(s) => quote(s),
        Value::Symbol(s) => s.clone(),
        Value::List(items) => format_list(items, indent),
        Value::Object { name, fields } => format_object(name.as_deref(), fields, indent),
    }
}

/// Floats keep a decimal point so that a round trip cannot silently turn
/// `3.0` into the integer `3`.
fn format_float(f: f64) -> String {
    let s = f.to_string();
    if s.contains(['.', 'e', 'E', 'n', 'i']) {
        s
    } else {
        format!("{}.0", s)
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.escape_default())
}

/// A key may be written bare only if it lexes back as a symbol.
fn format_key(key: &str) -> String {
    let mut chars = key.chars();
    let starts_ok = chars.next().is_some_and(|c| c.is_alphabetic() || c == '_');
    let rest_ok = chars.all(|c| c.is_alphanumeric() || c == '_');
    let reserved = matches!(key, "true" | "false" | "null");

    if starts_ok && rest_ok && !reserved {
        key.to_string()
    } else {
        quote(key)
    }
}

fn block(open: &str, close: char, indent: usize, entries: Vec<String>) -> String {
    if entries.is_empty() {
        return format!("{}{}", open, close);
    }
    let inner = "  ".repeat(indent + 1);
    let mut result = format!("{}\n", open);
    for entry in entries {
        result.push_str(&inner);
        result.push_str(&entry);
        result.push('\n');
    }
    result.push_str(&"  ".repeat(indent));
    result.push(close);
    result
}

fn format_list(items: &[Value], indent: usize) -> String {
    let entries = items
        .iter()
        .map(|item| format_value(item, indent + 1))
        .collect();
    block("[", ']', indent, entries)
}

fn format_object(name: Option<&str>, fields: &HashMap<String, Value>, indent: usize) -> String {
    let mut sorted: Vec<_> = fields.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));

    let entries = sorted
        .into_iter()
        .map(|(key, value)| format!("{} {}", format_key(key), format_value(value, indent + 1)))
        .collect();

    block(&format!("{}(", name.unwrap_or("")), ')', indent, entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn round_trip(src: &str) {
        let parsed = nex_parser::parse(src).unwrap();
        let formatted = format(&parsed);
        let reparsed = nex_parser::parse(&formatted).unwrap_or_else(|e| {
            panic!("formatter emitted unparseable output: {}\n{}", e, formatted)
        });
        assert_eq!(
            parsed, reparsed,
            "round trip changed the value\n{}",
            formatted
        );
    }

    #[test]
    fn test_round_trip() {
        round_trip(
            r#"
config(
  debug true
  name "my app"
  settings(
    features[logging caching]
    timeout 30
  )
  server tcp(host "0.0.0.0" port 8080)
  version "1.0.0"
)
"#,
        );
    }

    #[test]
    fn lists_use_square_brackets() {
        let value = nex_parser::parse("app(tags[cli lsp])").unwrap();
        let out = format(&value);
        assert!(
            out.contains("tags ["),
            "expected a bracketed list, got:\n{}",
            out
        );
        assert!(
            !out.contains("tags ("),
            "list must not use parens:\n{}",
            out
        );
    }

    #[test]
    fn empty_collections_stay_compact() {
        assert_eq!(format(&nex_parser::parse("[]").unwrap()), "[]");
        assert_eq!(format(&nex_parser::parse("()").unwrap()), "()");
        assert_eq!(format(&nex_parser::parse("app()").unwrap()), "app()");
    }

    #[test]
    fn anonymous_nested_objects_keep_their_parens() {
        let out = format(&nex_parser::parse(r#"app(server(host "h"))"#).unwrap());
        assert!(out.contains("server ("), "got:\n{}", out);
        round_trip(r#"app(server(host "h"))"#);
    }

    #[test]
    fn keys_that_are_not_symbols_are_quoted() {
        round_trip(r#"app("my key" 1 "with \"quote\"" 2)"#);
    }

    #[test]
    fn floats_survive_a_round_trip() {
        round_trip("app(ratio 3.0 tiny 0.5 big 1e10)");
    }
}
