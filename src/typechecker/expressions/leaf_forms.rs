use crate::ast::expressions::{BinaryOp, LoopControlAction, UnaryOp};

use super::*;

impl TypeChecker {
    pub(super) fn check_int_literal_expr(
        &mut self,
        value: i64,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::IntLiteral(value),
            ty: Type::I32,
            span,
        })
    }

    pub(super) fn check_float_literal_expr(
        &mut self,
        value: f64,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::FloatLiteral(value),
            ty: Type::F64,
            span,
        })
    }

    pub(super) fn check_static_string_literal_expr(
        &mut self,
        value: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::StringLiteral(value.to_owned()),
            ty: Type::Str,
            span,
        })
    }

    pub(super) fn check_bool_literal_expr(
        &mut self,
        value: bool,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::BoolLiteral(value),
            ty: Type::Bool,
            span,
        })
    }

    pub(super) fn check_binary_expr(
        &mut self,
        op: BinaryOp,
        left: &Expression,
        right: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let left = self.check_expr(left)?;
        let right = self.check_expr(right)?;
        let ty = self.check_binary_op(op, &left.ty, &right.ty, &span)?;
        Ok(TypedExpression {
            kind: TypedExprKind::BinaryOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            },
            ty,
            span,
        })
    }

    pub(super) fn check_break_expr(&mut self, span: Span) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::Break,
            ty: Type::Never,
            span,
        })
    }

    pub(super) fn check_continue_expr(
        &mut self,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::Continue,
            ty: Type::Never,
            span,
        })
    }

    pub(super) fn check_loop_control_expr(
        &mut self,
        action: LoopControlAction,
        target_label: &str,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::LoopControl {
                action,
                label: target_label.to_owned(),
            },
            ty: Type::Never,
            span,
        })
    }

    pub(super) fn check_unary_expr(
        &mut self,
        op: UnaryOp,
        operand: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        let typed = self.check_expr(operand)?;
        let ty = typed.ty.clone();
        Ok(TypedExpression {
            kind: TypedExprKind::UnaryOp {
                op,
                operand: Box::new(typed),
            },
            ty,
            span,
        })
    }

    pub(super) fn check_range_expr(
        &mut self,
        start: &Expression,
        end: &Expression,
        span: Span,
    ) -> Result<TypedExpression, Diagnostic> {
        self.check_expr(start)?;
        self.check_expr(end)?;
        Err(Diagnostic::error(
            "E3053",
            "range expressions are not implemented; range typing remains gated",
            span,
        ))
    }

    pub(super) fn check_error_expr(&mut self, span: Span) -> Result<TypedExpression, Diagnostic> {
        Ok(TypedExpression {
            kind: TypedExprKind::Error,
            ty: Type::Unknown,
            span,
        })
    }
}
