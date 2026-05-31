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
use crate::error::CompilerDiagnosticCode::*;
use crate::error::{Diagnostic, Span};

use super::generics::monomorphize_inference::InferenceConflict;
use super::{type_display_pair, FuncInfo, TypeChecker};

fn typed_expr(kind: TypedExprKind, ty: Type, span: Span) -> TypedExpression {
    TypedExpression { kind, ty, span }
}

fn typed_ok(kind: TypedExprKind, ty: Type, span: Span) -> Result<TypedExpression, Diagnostic> {
    Ok(typed_expr(kind, ty, span))
}

fn typed_call_expr(
    function: String,
    args: Vec<TypedExpression>,
    ty: Type,
    span: Span,
) -> TypedExpression {
    typed_expr(TypedExprKind::FunctionCall { function, args }, ty, span)
}

fn typed_block_from_expr(expr: TypedExpression) -> TypedBlock {
    TypedBlock {
        ty: expr.ty.clone(),
        span: expr.span,
        statements: Vec::new(),
        expr: Some(Box::new(expr)),
    }
}

fn typed_match_expr(
    scrutinee: TypedExpression,
    arms: Vec<TypedMatchArm>,
    kind: MatchKind,
    ty: Type,
    span: Span,
) -> Result<TypedExpression, Diagnostic> {
    typed_ok(
        TypedExprKind::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            kind,
        },
        ty,
        span,
    )
}

impl TypeChecker {
    fn check_exprs(&mut self, exprs: &[Expression]) -> Result<Vec<TypedExpression>, Diagnostic> {
        exprs.iter().map(|expr| self.check_expr(expr)).collect()
    }

    pub(crate) fn check_expr(&mut self, expr: &Expression) -> Result<TypedExpression, Diagnostic> {
        match expr {
            Expression::IntLiteral { value, span } => {
                typed_ok(TypedExprKind::IntLiteral(*value), Type::I32, *span)
            }
            Expression::FloatLiteral { value, span } => {
                typed_ok(TypedExprKind::FloatLiteral(*value), Type::F64, *span)
            }
            Expression::StringLiteral { value, span } => typed_ok(
                TypedExprKind::StringLiteral(value.clone()),
                Type::Str,
                *span,
            ),
            Expression::BoolLiteral { value, span } => {
                typed_ok(TypedExprKind::BoolLiteral(*value), Type::Bool, *span)
            }

            Expression::Identifier { name, span } => self.check_identifier_expr(name, *span),

            Expression::BinaryOp {
                op,
                left,
                right,
                span,
            } => {
                let mut left = self.check_expr(left)?;
                let mut right = self.check_expr(right)?;
                coerce_binop_numeric_literals(&mut left, &mut right);
                let ty = self.check_binary_op(*op, &left.ty, &right.ty, span)?;
                typed_ok(
                    TypedExprKind::BinaryOp {
                        op: *op,
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                    ty,
                    *span,
                )
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

            Expression::Loop {
                body,
                control_label,
                span,
            } => self.check_loop_expr(body, control_label, *span),

            Expression::LoopControl {
                action,
                target_label,
                span,
            } => typed_ok(
                TypedExprKind::LoopControl {
                    action: *action,
                    label: target_label.clone(),
                },
                Type::Never,
                *span,
            ),

            Expression::Cast {
                expr,
                target_type,
                span,
            } => self.check_cast_expr(expr, target_type, *span),

            Expression::StringInterpolation { parts, span } => {
                self.check_string_interpolation_expr(parts, *span)
            }

            Expression::Defer { expr, span } => self.check_defer_expr(expr, *span),

            Expression::Await { expr, span } => self.check_await_expr(expr, *span),

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
                typed_ok(
                    TypedExprKind::UnaryOp {
                        op: *op,
                        operand: Box::new(typed),
                    },
                    ty,
                    *span,
                )
            }
        }
    }
}

/// An untyped integer/float literal operand adopts the other operand's numeric
/// type, so `n - 1` (where `n: u32`) type-checks. Only a literal side is retyped,
/// and only when the kinds are compatible — a float literal must never silently
/// become an integer (that would emit a C double where the type says integer),
/// while an int literal may promote to a float (`1 + 2.5`).
fn coerce_binop_numeric_literals(left: &mut TypedExpression, right: &mut TypedExpression) {
    if literal_adopts(left, &right.ty) {
        left.ty = right.ty.clone();
    } else if literal_adopts(right, &left.ty) {
        right.ty = left.ty.clone();
    }
}

/// Whether `operand` is a numeric literal that may soundly adopt `target`.
fn literal_adopts(operand: &TypedExpression, target: &Type) -> bool {
    if !is_numeric(target) || !is_numeric(&operand.ty) || operand.ty == *target {
        return false;
    }
    match operand.kind {
        TypedExprKind::IntLiteral(_) => true,
        TypedExprKind::FloatLiteral(_) => target.is_float(),
        _ => false,
    }
}

fn is_numeric(ty: &Type) -> bool {
    ty.is_integer() || ty.is_float()
}
