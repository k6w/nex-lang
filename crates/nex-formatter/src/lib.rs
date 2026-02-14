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
        return "[]".to_string();
    }

    let mut result = "[\n".to_string();
    let item_indent = "  ".repeat(indent + 1);

    for (i, item) in items.iter().enumerate() {
        result.push_str(&item_indent);
        result.push_str(&format_value(item, indent + 1));
        if i < items.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }

    result.push_str(&"  ".repeat(indent));
    result.push(']');
    result
}

fn format_object(name: &Option<String>, fields: &HashMap<String, Value>, indent: usize) -> String {
    let name_str = name.as_ref().map(|s| s.clone()).unwrap_or_else(|| "".to_string());
    if fields.is_empty() {
        return format!("{} {{}}", name_str);
    }

    let mut result = format!("{} {{\n", name_str);
    let field_indent = "  ".repeat(indent + 1);

    // Sort fields for consistent output
    let mut sorted_fields: Vec<_> = fields.iter().collect();
    sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

    for (i, (key, value)) in sorted_fields.iter().enumerate() {
        result.push_str(&field_indent);
        result.push_str(key);
        result.push_str(": ");
        result.push_str(&format_value(value, indent + 1));
        if i < sorted_fields.len() - 1 {
            result.push(',');
        }
        result.push('\n');
    }

    result.push_str(&"  ".repeat(indent));
    result.push('}');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use nex_parser::parse;

    #[test]
    fn test_round_trip() {
        let input = r#"
config {
  name: "My App",
  version: "1.0.0",
  debug: true,
  settings: {
    timeout: 30,
    features: [logging, caching]
  }
}
"#;

        let parsed = nex_parser::parse(input).unwrap();
        let formatted = format(&parsed);
        let reparsed = nex_parser::parse(&formatted).unwrap();

        assert_eq!(parsed, reparsed);
    }
}