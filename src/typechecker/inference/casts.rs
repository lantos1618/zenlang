//! Type casting inference

use crate::ast::{AstType, Expression};
use crate::error::{CompileError, Result};

/// Infer the target type for a cast() expression
pub fn infer_cast_type(args: &[Expression], span: Option<crate::error::Span>) -> Result<AstType> {
    if args.len() == 2 {
        if let Expression::Identifier(type_name) = &args[1] {
            // Use the centralized type parser instead of manual matching
            return match crate::parser::parse_type_from_string(type_name) {
                Ok(ast_type) => {
                    // Only allow numeric primitive types for cast()
                    if ast_type.is_numeric() {
                        Ok(ast_type)
                    } else {
                        Err(CompileError::TypeError(
                            format!(
                                "cast() target type '{}' is not a valid numeric type. Supported types: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64",
                                type_name
                            ),
                            span.clone(),
                        ))
                    }
                }
                Err(e) => Err(CompileError::TypeError(
                    format!(
                        "cast() target type '{}' is not a valid type: {}",
                        type_name, e
                    ),
                    span.clone(),
                )),
            };
        }
    }

    Err(CompileError::TypeError(
        format!(
            "cast() expects 2 arguments: cast(value, type), but got {} argument(s). Example: cast(42, i64)",
            args.len()
        ),
        span,
    ))
}
