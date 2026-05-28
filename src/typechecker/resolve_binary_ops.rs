use crate::ast::expressions::BinaryOp;
use crate::ast::typed::Type;
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::{type_display_pair, TypeChecker};

impl TypeChecker {
    pub(crate) fn check_binary_op(
        &self,
        op: BinaryOp,
        left: &Type,
        right: &Type,
        span: &Span,
    ) -> Result<Type, Diagnostic> {
        match op {
            BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                if *left == Type::Unknown || *right == Type::Unknown {
                    return Ok(known_binary_operand(left, right).clone());
                }
                for ty in [left, right] {
                    if !ty.is_integer() && !ty.is_float() {
                        return Err(Diagnostic::error_code(
                            E3010,
                            format!("arithmetic on non-numeric type `{}`", ty.display_name()),
                            *span,
                        ));
                    }
                }
                if left != right {
                    let (left_name, right_name) = type_display_pair(left, right);
                    return Err(Diagnostic::error_code(
                        E3013,
                        format!("arithmetic operands must have the same type, found `{left_name}` and `{right_name}`"),
                        *span,
                    ));
                }
                Ok(left.clone())
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => Ok(Type::Bool),
            BinaryOp::And | BinaryOp::Or => {
                for ty in [left, right] {
                    if *ty != Type::Bool && *ty != Type::Unknown {
                        return Err(Diagnostic::error_code(
                            E3011,
                            format!(
                                "logical operator requires `bool`, found `{}`",
                                ty.display_name()
                            ),
                            *span,
                        ));
                    }
                }
                Ok(Type::Bool)
            }
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => {
                if *left == Type::Unknown || *right == Type::Unknown {
                    return Ok(known_binary_operand(left, right).clone());
                }
                for ty in [left, right] {
                    if !ty.is_integer() {
                        return Err(Diagnostic::error_code(
                            E3012,
                            format!(
                                "bitwise operator requires integer type, found `{}`",
                                ty.display_name()
                            ),
                            *span,
                        ));
                    }
                }
                Ok(left.clone())
            }
        }
    }
}

fn known_binary_operand<'a>(left: &'a Type, right: &'a Type) -> &'a Type {
    if *left != Type::Unknown {
        left
    } else {
        right
    }
}
