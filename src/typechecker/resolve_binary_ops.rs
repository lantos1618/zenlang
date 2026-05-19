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
            let known = if *left != Type::Unknown {
                left
            } else {
                right
            };
            return Ok(known.clone());
        }
        if !left.is_numeric() {
            return Err(Diagnostic::error(
                "E3010",
                format!("arithmetic on non-numeric type `{}`", left.display_name()),
                *span,
            ));
        }
        if !right.is_numeric() {
            return Err(Diagnostic::error(
                "E3010",
                format!("arithmetic on non-numeric type `{}`", right.display_name()),
                *span,
            ));
        }
        if left != right {
            return Err(Diagnostic::error(
                "E3013",
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
        if *left != Type::Bool && *left != Type::Unknown {
            return Err(Diagnostic::error(
                "E3011",
                format!(
                    "logical operator requires `bool`, found `{}`",
                    left.display_name()
                ),
                *span,
            ));
        }
        if *right != Type::Bool && *right != Type::Unknown {
            return Err(Diagnostic::error(
                "E3011",
                format!(
                    "logical operator requires `bool`, found `{}`",
                    right.display_name()
                ),
                *span,
            ));
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
            let known = if *left != Type::Unknown {
                left
            } else {
                right
            };
            return Ok(known.clone());
        }
        if !left.is_integer() {
            return Err(Diagnostic::error(
                "E3012",
                format!(
                    "bitwise operator requires integer type, found `{}`",
                    left.display_name()
                ),
                *span,
            ));
        }
        if !right.is_integer() {
            return Err(Diagnostic::error(
                "E3012",
                format!(
                    "bitwise operator requires integer type, found `{}`",
                    right.display_name()
                ),
                *span,
            ));
        }
        Ok(left.clone())
    }
}
