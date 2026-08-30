pub mod ast;
pub mod parser;

/// Parse a NEX string into a Value
pub fn parse(input: &str) -> Result<ast::Value, ast::ParseError> {
    parser::parse(input)
}
