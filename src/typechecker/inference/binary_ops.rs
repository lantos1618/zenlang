//! Binary operation type inference

use super::helpers::is_string_type;
use crate::ast::{int_from_bit_size, AstType, BinaryOperator, Expression};
use crate::error::{CompileError, Result};
use crate::typechecker::TypeChecker;

/// Infer the type of a binary operation
pub fn infer_binary_op_type(
    checker: &mut TypeChecker,
    left: &Expression,
    op: &BinaryOperator,
    right: &Expression,
) -> Result<AstType> {
    let mut left_type = checker.infer_expression_type(left)?;
    let mut right_type = checker.infer_expression_type(right)?;

    // DESIGN: Unresolved generic T defaults to i32 for binary op type inference.
    // This handles cases like `Option.None + 1` where the T type parameter hasn't been
    // resolved yet. The monomorphizer will resolve actual types later.
    if let AstType::Generic { name, type_args } = &left_type {
        if name == "T" && type_args.is_empty() {
            left_type = AstType::I32;
        } else if name == "E" && type_args.is_empty() {
            // DESIGN: Unresolved generic E defaults to String for binary op type inference.
            // Error types default to String (dynamic with allocator) to allow binary operations
            // on Result values before monomorphization resolves the actual error type.
            left_type = crate::ast::resolve_string_struct_type();
        }
    }
    if let AstType::Generic { name, type_args } = &right_type {
        if name == "T" && type_args.is_empty() {
            right_type = AstType::I32;
        } else if name == "E" && type_args.is_empty() {
            // DESIGN: Unresolved generic E defaults to String for binary op type inference.
            // Error types default to String (dynamic with allocator) to allow binary operations
            // on Result values before monomorphization resolves the actual error type.
            right_type = crate::ast::resolve_string_struct_type();
        }
    }

    match op {
        BinaryOperator::Add
        | BinaryOperator::Subtract
        | BinaryOperator::Multiply
        | BinaryOperator::Divide
        | BinaryOperator::Modulo => {
            // Numeric operations
            if left_type.is_numeric() && right_type.is_numeric() {
                // Promote to the larger type
                promote_numeric_types(&left_type, &right_type, checker.get_current_span())
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Cannot apply {} to {} and {}: arithmetic operators require numeric types (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64)",
                        op, left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::Equals
        | BinaryOperator::NotEquals
        | BinaryOperator::LessThan
        | BinaryOperator::GreaterThan
        | BinaryOperator::LessThanEquals
        | BinaryOperator::GreaterThanEquals => {
            // Comparison operations return bool
            if super::binary_ops::types_comparable(&left_type, &right_type) {
                Ok(AstType::Bool)
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Cannot compare {} with {}: comparison requires compatible types (same type, both numeric, or both pointers)",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::And | BinaryOperator::Or => {
            // Logical operations
            if matches!(left_type, AstType::Bool) && matches!(right_type, AstType::Bool) {
                Ok(AstType::Bool)
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Logical operators (and, or) require boolean operands, got {} and {}",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        BinaryOperator::StringConcat => {
            // String concatenation - all string types can be concatenated
            if is_string_type(&left_type) && is_string_type(&right_type) {
                // Result is always String (dynamic) when concatenating
                Ok(crate::ast::resolve_string_struct_type())
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "String concatenation (++) requires string operands, got {} and {}. Tip: use cast() to convert numbers to strings",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
        // Bitwise operators - require integer operands, return promoted integer type
        BinaryOperator::BitwiseAnd
        | BinaryOperator::BitwiseOr
        | BinaryOperator::BitwiseXor
        | BinaryOperator::ShiftLeft
        | BinaryOperator::ShiftRight => {
            if left_type.is_integer() && right_type.is_integer() {
                promote_numeric_types(&left_type, &right_type, checker.get_current_span())
            } else {
                Err(CompileError::TypeError(
                    format!(
                        "Bitwise operators (&, |, ^, <<, >>) require integer operands, got {} and {}",
                        left_type, right_type
                    ),
                    checker.get_current_span(),
                ))
            }
        }
    }
}

/// Promote two numeric types to their common type
pub fn promote_numeric_types(
    left: &AstType,
    right: &AstType,
    span: Option<crate::error::Span>,
) -> Result<AstType> {
    // If either is a float, promote to float
    if left.is_float() || right.is_float() {
        if matches!(left, AstType::F64) || matches!(right, AstType::F64) {
            Ok(AstType::F64)
        } else {
            Ok(AstType::F32)
        }
    }
    // Both are integers
    else if left.is_integer() && right.is_integer() {
        // Special case: if both are Usize, keep Usize
        if matches!(left, AstType::Usize) && matches!(right, AstType::Usize) {
            return Ok(AstType::Usize);
        }
        // If one is Usize and other is compatible unsigned, promote to Usize
        if (matches!(left, AstType::Usize) || matches!(right, AstType::Usize))
            && left.is_unsigned_integer()
            && right.is_unsigned_integer()
        {
            return Ok(AstType::Usize);
        }

        // Promote to the larger size
        let left_size = left.bit_size().unwrap_or(32);
        let right_size = right.bit_size().unwrap_or(32);
        let max_size = left_size.max(right_size);

        // DESIGN: Mixed signed/unsigned promotion prefers signed integer of the max size.
        // This prevents silent truncation of negative values when mixing i32 and u32.
        // The trade-off is that large unsigned values may not fit in the signed type.
        if left.is_unsigned_integer() != right.is_unsigned_integer() {
            Ok(int_from_bit_size(max_size, true).unwrap_or(AstType::I32))
        } else if left.is_unsigned_integer() {
            // Both unsigned
            Ok(int_from_bit_size(max_size, false).unwrap_or(AstType::U32))
        } else {
            // Both signed
            Ok(int_from_bit_size(max_size, true).unwrap_or(AstType::I32))
        }
    } else {
        Err(CompileError::TypeError(
            format!(
                "Cannot promote types {} and {}: type promotion requires both operands to be numeric",
                left, right
            ),
            span,
        ))
    }
}

/// Check if two types can be compared
pub fn types_comparable(left: &AstType, right: &AstType) -> bool {
    if std::mem::discriminant(left) == std::mem::discriminant(right) {
        return true;
    }

    if left.is_numeric() && right.is_numeric() {
        return true;
    }

    if left.is_ptr_type() && right.is_ptr_type() {
        return true;
    }

    if (left.is_ptr_type() && right.is_numeric()) || (left.is_numeric() && right.is_ptr_type()) {
        return true;
    }

    if is_string_type(left) && is_string_type(right) {
        return true;
    }

    if matches!(left, AstType::Bool) && matches!(right, AstType::Bool) {
        return true;
    }

    // Allow bool-to-integer and integer-to-bool comparisons (e.g. atomic ops return i32)
    if (matches!(left, AstType::Bool) && right.is_numeric())
        || (left.is_numeric() && matches!(right, AstType::Bool))
    {
        return true;
    }

    // Allow enum-to-enum comparison by name (e.g. TaskState == TaskState)
    if matches!((left, right), (AstType::Enum { .. }, AstType::Enum { .. })) {
        return true;
    }

    // Allow Generic/Enum cross-comparison (field typed as Generic "TaskState" vs enum literal TaskState.Completed)
    match (left, right) {
        (
            AstType::Generic {
                name: left_name,
                type_args,
            },
            AstType::Enum {
                name: right_name, ..
            },
        ) if type_args.is_empty() && left_name == right_name => return true,
        (
            AstType::Enum {
                name: left_name, ..
            },
            AstType::Generic {
                name: right_name,
                type_args,
            },
        ) if type_args.is_empty() && left_name == right_name => return true,
        _ => {}
    }

    false
}
