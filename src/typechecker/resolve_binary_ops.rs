#![allow(clippy::result_large_err)]

use crate::ast::expressions::BinaryOp;
use crate::ast::typed::Type;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

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
                self.check_arithmetic_binary_op(left, right, span)
            }
            BinaryOp::Eq
            | BinaryOp::NotEq
            | BinaryOp::Lt
            | BinaryOp::Gt
            | BinaryOp::LtEq
            | BinaryOp::GtEq => Ok(Type::Bool),
            BinaryOp::And | BinaryOp::Or => self.check_logical_binary_op(left, right, span),
            BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor
            | BinaryOp::ShiftLeft
            | BinaryOp::ShiftRight => self.check_bitwise_binary_op(left, right, span),
        }
    }

    fn check_arithmetic_binary_op(
        &self,
        left: &Type,
        right: &Type,
        span: &Span,
    ) -> Result<Type, Diagnostic> {
        if *left == Type::Unknown || *right == Type::Unknown {
            return Ok(known_binary_operand(left, right).clone());
        }
        for ty in [left, right] {
            reject_binary_operand_if(
                !ty.is_numeric(),
                crate::error::CompilerDiagnosticCode::E3010,
                || format!("arithmetic on non-numeric type `{}`", ty.display_name()),
                span,
            )?;
        }
        if left != right {
            return Err(Diagnostic::error_code(
                crate::error::CompilerDiagnosticCode::E3013,
                format!(
                    "arithmetic operands must have the same type, found `{}` and `{}`",
                    left.display_name(),
                    right.display_name()
                ),
                *span,
            ));
        }
        Ok(left.clone())
    }

    fn check_logical_binary_op(
        &self,
        left: &Type,
        right: &Type,
        span: &Span,
    ) -> Result<Type, Diagnostic> {
        for ty in [left, right] {
            reject_binary_operand_if(
                *ty != Type::Bool && *ty != Type::Unknown,
                crate::error::CompilerDiagnosticCode::E3011,
                || {
                    format!(
                        "logical operator requires `bool`, found `{}`",
                        ty.display_name()
                    )
                },
                span,
            )?;
        }
        Ok(Type::Bool)
    }

    fn check_bitwise_binary_op(
        &self,
        left: &Type,
        right: &Type,
        span: &Span,
    ) -> Result<Type, Diagnostic> {
        if *left == Type::Unknown || *right == Type::Unknown {
            return Ok(known_binary_operand(left, right).clone());
        }
        for ty in [left, right] {
            reject_binary_operand_if(
                !ty.is_integer(),
                crate::error::CompilerDiagnosticCode::E3012,
                || {
                    format!(
                        "bitwise operator requires integer type, found `{}`",
                        ty.display_name()
                    )
                },
                span,
            )?;
        }
        Ok(left.clone())
    }
}

fn known_binary_operand<'a>(left: &'a Type, right: &'a Type) -> &'a Type {
    if *left != Type::Unknown {
        left
    } else {
        right
    }
}

fn reject_binary_operand_if(
    reject: bool,
    code: impl Into<crate::error::DiagnosticCode>,
    message: impl FnOnce() -> String,
    span: &Span,
) -> Result<(), Diagnostic> {
    if reject {
        return Err(Diagnostic::error_code(code, message(), *span));
    }
    Ok(())
}
