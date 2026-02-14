use nex_parser::ast::Value;
use std::collections::HashMap;

/// Schema validation result
#[derive(Debug)]
pub enum ValidationResult {
    Valid,
    Invalid(Vec<String>),
}

/// Basic schema validation
/// For now, just checks basic structure
pub fn validate_against_schema(value: &Value, schema: &Value) -> ValidationResult {
    match (value, schema) {
        (Value::Object { name: val_name, fields: val_fields }, Value::Object { name: schema_name, fields: schema_fields }) => {
            let mut errors = Vec::new();

            // Check name if schema specifies one
            if let Some(schema_name) = schema_name {
                if let Some(val_name) = val_name {
                    if val_name != schema_name {
                        errors.push(format!("Object name mismatch: expected {}, got {}", schema_name, val_name));
                    }
                } else {
                    errors.push(format!("Expected named object '{}', got anonymous", schema_name));
                }
            }

            // Check required fields
            for (key, expected_type) in schema_fields {
                match val_fields.get(key) {
                    Some(actual_value) => {
                        if let Err(type_error) = check_type(actual_value, expected_type) {
                            errors.push(format!("Field '{}': {}", key, type_error));
                        }
                    }
                    None => {
                        errors.push(format!("Missing required field: {}", key));
                    }
                }
            }

            if errors.is_empty() {
                ValidationResult::Valid
            } else {
                ValidationResult::Invalid(errors)
            }
        }
        _ => ValidationResult::Invalid(vec!["Schema root must be an object".to_string()]),
    }
}

fn check_type(value: &Value, expected: &Value) -> Result<(), String> {
    match (value, expected) {
        (Value::String(_), Value::Symbol(s)) if s == "string" => Ok(()),
        (Value::Int(_), Value::Symbol(s)) if s == "int" => Ok(()),
        (Value::Float(_), Value::Symbol(s)) if s == "float" => Ok(()),
        (Value::Bool(_), Value::Symbol(s)) if s == "bool" => Ok(()),
        (Value::List(_), Value::Symbol(s)) if s == "list" => Ok(()),
        (Value::Object { .. }, Value::Symbol(s)) if s == "object" => Ok(()),
        (Value::Null, Value::Symbol(s)) if s == "null" => Ok(()),
        (Value::Symbol(_), Value::Symbol(s)) if s == "symbol" => Ok(()),
        _ => Err(format!("Type mismatch: expected {}, got {}", type_name(expected), value.type_name())),
    }
}

fn type_name(value: &Value) -> &str {
    match value {
        Value::Symbol(s) => s,
        _ => value.type_name(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nex_parser::parse;

    #[test]
    fn test_basic_validation() {
        let data = parse(r#"config { name: "test", count: 42 }"#).unwrap();
        let schema = parse(r#"config { name: string, count: int }"#).unwrap();

        match validate_against_schema(&data, &schema) {
            ValidationResult::Valid => {},
            ValidationResult::Invalid(errors) => panic!("Validation failed: {:?}", errors),
        }
    }
}