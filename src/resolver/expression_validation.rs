use crate::ast::{Expression, StringPart, TypeParam};
use crate::error::{CompilerDiagnosticCode::E0203, Diagnostic, Span};

use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

pub(super) struct ExprRefContext<'a, 'b> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_params: &'a [TypeParam],
    pub(super) locals: &'b mut ScopeStack,
    pub(super) allow_self_type: bool,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

impl Resolver {
    pub(super) fn validate_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut ctx = ExprRefContext {
            table,
            type_params,
            locals,
            allow_self_type,
            diagnostics,
        };
        self.validate_expr_refs_in(expr, &mut ctx);
    }

    pub(super) fn validate_expr_refs_in(
        &self,
        expr: &Expression,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        match expr {
            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => {
                self.validate_type_arg_refs(type_args, *span, ctx);
                if module.is_none() {
                    self.validate_known_value_name(name, *span, ctx);
                }
                self.validate_expr_arg_refs(args, ctx);
            }
            Expression::Identifier { name, span } => {
                self.validate_known_value_name(name, *span, ctx);
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_expr_refs_in(receiver, ctx);
                self.validate_type_arg_refs(type_args, *span, ctx);
                self.validate_expr_arg_refs(args, ctx);
            }
            Expression::BinaryOp { left, right, .. }
            | Expression::IndexAccess {
                object: left,
                index: right,
                ..
            } => {
                self.validate_expr_refs_in(left, ctx);
                self.validate_expr_refs_in(right, ctx);
            }
            Expression::UnaryOp { operand, .. }
            | Expression::MemberAccess {
                object: operand, ..
            }
            | Expression::Defer { expr: operand, .. }
            | Expression::Await { expr: operand, .. } => self.validate_expr_refs_in(operand, ctx),
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_expr_refs_in(expr, ctx);
                self.validate_expr_type_ref(target_type, *span, ctx);
            }
            Expression::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                self.validate_expr_refs_in(condition, ctx);
                self.validate_child_scope_expr_refs(then_body, ctx);
                if let Some(else_body) = else_body {
                    self.validate_child_scope_expr_refs(else_body, ctx);
                }
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        self.validate_expr_refs_in(expr, ctx);
                    }
                }
            }
            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => self.validate_struct_literal_refs(name, type_args, fields, *span, ctx),
            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => self.validate_enum_variant_refs(
                enum_name,
                type_args,
                variant,
                payload.as_deref(),
                *span,
                ctx,
            ),
            Expression::ArrayLiteral { elements, .. } => {
                self.validate_expr_arg_refs(elements, ctx);
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_expr_refs_in(scrutinee, ctx);
                for arm in arms {
                    self.validate_match_arm_refs(arm, ctx);
                }
            }
            Expression::Loop { body, .. } => self.validate_child_scope_expr_refs(body, ctx),
            Expression::Block {
                statements, expr, ..
            } => self.validate_block_refs(statements, expr.as_deref(), ctx),
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => self.validate_closure_refs(params, return_type.as_ref(), body, *span, ctx),
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::LoopControl { .. } => {}
        }
    }

    fn validate_known_value_name(&self, name: &str, span: Span, ctx: &mut ExprRefContext<'_, '_>) {
        if !self.is_known_value_name(ctx.table, ctx.locals, name) {
            ctx.diagnostics.push(Diagnostic::error_code(
                E0203,
                format!("unknown value symbol '{name}'"),
                span,
            ));
        }
    }
}
