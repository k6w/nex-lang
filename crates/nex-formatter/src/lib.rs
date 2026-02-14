use nex_parser::ast::Value;
use std::collections::HashMap;

/// Format a NEX value back to string
pub fn format(value: &Value) -> String {
    format_value(value, 0)
}

fn format_value(value: &Value, indent: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::String(s) => format!("\"{}\"", s.escape_default()),
        Value::Symbol(s) => s.clone(),
        Value::List(items) => format_list(items, indent),
        Value::Object { name, fields } => format_object(name, fields, indent),
    }
}

fn format_list(items: &[Value], indent: usize) -> String {
    if items.is_empty() {
        return "()".to_string();
    }

    let mut result = "(\n".to_string();
    let item_indent = "  ".repeat(indent + 1);

    for item in items.iter() {
        result.push_str(&item_indent);
        result.push_str(&format_value(item, indent + 1));
        result.push('\n');
    }

    result.push_str(&"  ".repeat(indent));
    result.push(')');
    result
}

fn format_object(name: &Option<String>, fields: &HashMap<String, Value>, indent: usize) -> String {
    match name {
        Some(name_str) => {
            if fields.is_empty() {
                return format!("{}()", name_str);
            }

            let mut result = format!("{}(\n", name_str);
            let field_indent = "  ".repeat(indent + 1);

            // Sort fields for consistent output
            let mut sorted_fields: Vec<_> = fields.iter().collect();
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            for (_i, (key, value)) in sorted_fields.iter().enumerate() {
                result.push_str(&field_indent);
                result.push_str(key);
                result.push(' ');
                result.push_str(&format_value(value, indent + 1));
                result.push('\n');
            }

            result.push_str(&"  ".repeat(indent));
            result.push(')');
            result
        }
        None => {
            // Anonymous object - format as just fields in parentheses
            if fields.is_empty() {
                return "()".to_string();
            }

            let mut result = "(\n".to_string();
            let field_indent = "  ".repeat(indent + 1);

            // Sort fields for consistent output
            let mut sorted_fields: Vec<_> = fields.iter().collect();
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            for (_i, (key, value)) in sorted_fields.iter().enumerate() {
                result.push_str(&field_indent);
                result.push_str(key);
                result.push(' ');
                result.push_str(&format_value(value, indent + 1));
                result.push('\n');
            }

            result.push_str(&"  ".repeat(indent));
            result.push(')');
            result
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_trip() {
        let input = r#"
config(
  debug true
  name "my app"
  settings(
    features(logging caching)
    timeout 30
  )
  version "1.0.0"
)
"#;

        let parsed = nex_parser::parse(input).unwrap();
        let formatted = format(&parsed);
        let reparsed = nex_parser::parse(&formatted).unwrap();

        assert_eq!(parsed, reparsed);
    }
}