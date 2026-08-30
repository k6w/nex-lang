use nex_parser::ast::Value;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

/// Convert NEX value to JSON
pub fn nex_to_json(value: &Value) -> JsonValue {
    match value {
        Value::Null => JsonValue::Null,
        Value::Bool(b) => json!(*b),
        Value::Int(i) => json!(*i),
        Value::Float(f) => json!(*f),
        Value::String(s) => json!(s),
        Value::Symbol(s) => json!(s), // Symbols become strings in JSON
        Value::List(items) => {
            let json_items: Vec<JsonValue> = items.iter().map(nex_to_json).collect();
            json!(json_items)
        }
        Value::Object { name, fields } => {
            let mut map = serde_json::Map::new();
            if let Some(n) = name {
                map.insert("_type".to_string(), json!(n));
            }
            for (key, val) in fields {
                map.insert(key.clone(), nex_to_json(val));
            }
            JsonValue::Object(map)
        }
    }
}

/// Convert JSON to NEX (lossy conversion)
pub fn json_to_nex(json: &JsonValue) -> Value {
    match json {
        JsonValue::Null => Value::Null,
        JsonValue::Bool(b) => Value::Bool(*b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Float(0.0) // fallback
            }
        }
        JsonValue::String(s) => Value::String(s.clone()),
        JsonValue::Array(arr) => {
            let items: Vec<Value> = arr.iter().map(json_to_nex).collect();
            Value::List(items)
        }
        JsonValue::Object(obj) => {
            let mut fields = HashMap::new();
            let mut name = None;

            for (key, val) in obj {
                if key == "_type" {
                    if let JsonValue::String(type_name) = val {
                        name = Some(type_name.clone());
                    }
                } else {
                    fields.insert(key.clone(), json_to_nex(val));
                }
            }

            Value::Object { name, fields }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nex_parser::parse;

    #[test]
    fn test_nex_to_json_round_trip() {
        let nex = r#"config(name "test" count 42)"#;
        let parsed = parse(nex).unwrap();
        let json = nex_to_json(&parsed);
        let back_to_nex = json_to_nex(&json);

        // Note: This is lossy due to type annotations
        // But the structure should be preserved
        if let Value::Object { fields, .. } = &back_to_nex {
            assert_eq!(fields.get("name"), Some(&Value::String("test".to_string())));
            assert_eq!(fields.get("count"), Some(&Value::Int(42)));
        } else {
            panic!("Expected object");
        }
    }
}
