use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a position in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: usize,
    pub column: usize,
}

/// Represents a span in the source code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// Abstract Syntax Tree for NEX values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Symbol(String),
    List(Vec<Value>),
    Object {
        name: Option<String>,
        fields: HashMap<String, Value>,
    },
}

impl Value {
    /// Get the type name of this value
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Symbol(_) => "symbol",
            Value::List(_) => "list",
            Value::Object { .. } => "object",
        }
    }
}

/// Parse error with position information
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Parse error at line {line}, column {column}: {message}")]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl ParseError {
    pub fn new(line: usize, column: usize, message: String) -> Self {
        Self { line, column, message }
    }
}