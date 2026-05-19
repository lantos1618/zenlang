//! Expression checking — check_function and check_expr.
#![allow(clippy::result_large_err)]

mod aggregate_constructors;
mod aggregate_support;
mod call_support;
mod call_validation;
mod control_flow_support;
mod enum_variant;
mod function_checking;
mod generic_call_validation;
mod method_call_support;
mod simple_forms;
mod struct_literal;

use crate::ast::expressions::StringPart;
use crate::ast::typed::*;
use crate::ast::{AstType, Expression, Param};
use crate::error::{Diagnostic, Span};

use super::closures::collect_captures;
use super::monomorphize_inference::InferenceConflict;
use super::monomorphize_types::concrete_name_matches_generic;
use super::{BehaviorBound, FuncInfo, TypeChecker};

impl TypeChecker {
    pub(crate) fn check_expr(&mut self, expr: &Expression) -> Result<TypedExpression, Diagnostic> {
        match expr {
            Expression::IntLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::IntLiteral(*value),
                ty: Type::I32, // default int type
                span: *span,
            }),

            Expression::FloatLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::FloatLiteral(*value),
                ty: Type::F64, // default float type
                span: *span,
            }),

            Expression::StringLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::StringLiteral(value.clone()),
                ty: Type::Str,
                span: *span,
            }),

            Expression::BoolLiteral { value, span } => Ok(TypedExpression {
                kind: TypedExprKind::BoolLiteral(*value),
                ty: Type::Bool,
                span: *span,
            }),

            Expression::Identifier { name, span } => self.check_identifier_expr(name, *span),

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let left = self.check_expr(left)?;
                let right = self.check_expr(right)?;
                let ty = self.check_binary_op(*op, &left.ty, &right.ty, span)?;
                Ok(TypedExpression {
                    kind: TypedExprKind::BinaryOp {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => self.check_function_call_expr(name, module, type_args, args, *span),

            Expression::MethodCall {
                receiver,
                method,
                type_args,
                args,
                span,
            } => self.check_method_call_expr(receiver, method, type_args, args, *span),

            Expression::MemberAccess {
                object,
                field,
                span,
            } => self.check_member_access_expr(object, field, *span),

            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => self.check_struct_literal_expr(name, type_args, fields, *span),

            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => self.check_enum_variant_expr(enum_name, type_args, variant, payload, *span),

            Expression::ArrayLiteral { elements, span } => {
                self.check_array_literal_expr(elements, *span)
            }

            Expression::Block {
                statements,
                expr,
                span,
            } => self.check_block_expr(statements, expr, *span),

            Expression::Break { span } => Ok(TypedExpression {
                kind: TypedExprKind::Break,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Continue { span } => Ok(TypedExpression {
                kind: TypedExprKind::Continue,
                ty: Type::Never,
                span: *span,
            }),

            Expression::Match {
                scrutinee,
                arms,
                span,
            } => self.check_match_expr(scrutinee, arms, *span),

            Expression::If {
                condition,
                then_body,
                else_body,
                span,
            } => self.check_if_expr(condition, then_body, else_body, *span),

            Expression::WhileLoop {
                condition,
                body,
                span,
            } => self.check_while_loop_expr(condition, body, *span),

            Expression::Loop {
                body,
                control_label,
                span,
            } => self.check_loop_expr(body, control_label, *span),

            Expression::LoopControl {
                action,
                target_label,
                span,
            } => Ok(TypedExpression {
                kind: TypedExprKind::LoopControl {
                    action: *action,
                    label: target_label.clone(),
                },
                ty: Type::Never,
                span: *span,
            }),

            Expression::Cast {
                expr,
                target_type,
                span,
            } => self.check_cast_expr(expr, target_type, *span),

            Expression::StringInterpolation { parts, span } => {
                self.check_string_interpolation_expr(parts, *span)
            }

            Expression::Defer { expr, span } => self.check_defer_expr(expr, *span),

            Expression::IndexAccess {
                object,
                index,
                span,
            } => self.check_index_access_expr(object, index, *span),

            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => self.check_closure_expr(params, return_type, body, *span),

            Expression::UnaryOp { op, operand, span } => {
                let typed = self.check_expr(operand)?;
                let ty = typed.ty.clone();
                Ok(TypedExpression {
                    kind: TypedExprKind::UnaryOp {
                        op: *op,
                        operand: Box::new(typed),
                    },
                    ty,
                    span: *span,
                })
            }

            Expression::Range {
                start, end, span, ..
            } => {
                self.check_expr(start)?;
                self.check_expr(end)?;
                Err(Diagnostic::error(
                    "E3053",
                    "range expressions are not implemented; range typing remains gated",
                    *span,
                ))
            }

            Expression::Error { span } => Ok(TypedExpression {
                kind: TypedExprKind::Error,
                ty: Type::Unknown,
                span: *span,
            }),
        }
    }
}
