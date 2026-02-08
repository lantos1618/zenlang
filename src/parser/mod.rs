pub mod behaviors;
pub mod core;
pub mod enums;
pub mod expressions;
pub mod external;
pub mod functions;
pub mod patterns;
pub mod program;
pub mod statements;
pub mod statements_guard;
pub mod structs;
pub mod types;

pub use core::Parser;

use crate::ast::AstType;
use crate::error::CompileError;
use crate::lexer::Lexer;

/// Parse a type from a string using the actual parser.
/// This is the canonical way to convert type strings to AstType.
pub fn parse_type_from_string(s: &str) -> Result<AstType, CompileError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(AstType::Void);
    }

    let lexer = Lexer::new(trimmed);
    let mut parser = Parser::new(lexer);
    parser.parse_type()
}

/// Extract base type name and type arguments from a generic type string.
/// Returns (base_name, type_args) - e.g. "Vec<i32>" -> ("Vec", [I32])
pub fn parse_generic_type_string(s: &str) -> (String, Vec<AstType>) {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return (String::new(), vec![]);
    }

    match parse_type_from_string(trimmed) {
        Ok(AstType::Generic { name, type_args }) => (name, type_args),
        Ok(other) => {
            // For non-generic types, return the type name with empty args
            // Use centralized primitive name lookup
            let name = other.primitive_name().unwrap_or(trimmed);
            (name.to_string(), vec![])
        }
        Err(_) => (trimmed.to_string(), vec![]),
    }
}

/// Parse comma-separated type arguments from a string like "i32, String, Vec<u8>"
pub fn parse_type_args_from_string(s: &str) -> Result<Vec<AstType>, CompileError> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Ok(vec![]);
    }

    let mut types = Vec::new();
    let mut depth = 0;
    let mut current_start = 0;

    for (i, c) in trimmed.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                let type_str = &trimmed[current_start..i];
                types.push(parse_type_from_string(type_str)?);
                current_start = i + 1;
            }
            _ => {}
        }
    }

    // Parse the last type
    let remaining = &trimmed[current_start..];
    if !remaining.trim().is_empty() {
        types.push(parse_type_from_string(remaining)?);
    }

    Ok(types)
}
