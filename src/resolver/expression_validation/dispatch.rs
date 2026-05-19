use crate::ast::{Expression, TypeParam};
use crate::error::Diagnostic;

use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};
use super::calls::{FunctionCallRef, MethodCallRef};
use super::traversal::{BinaryExprRef, IfOrWhileExprRef, IndexExprRef, RangeExprRef};

impl Resolver {
    pub(super) fn validate_call_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => self.validate_function_call_expr_refs(
                table,
                type_params,
                FunctionCallRef {
                    name,
                    module: module.as_deref(),
                    type_args,
                    args,
                    span: *span,
                },
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::Identifier { name, span } => {
                self.validate_identifier_expr_refs(table, name, *span, locals, diagnostics);
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => self.validate_method_call_expr_refs(
                table,
                type_params,
                MethodCallRef {
                    receiver,
                    type_args,
                    args,
                    span: *span,
                },
                locals,
                allow_self_type,
                diagnostics,
            ),
            _ => unreachable!("call expression dispatch received non-call expression"),
        }
    }

    pub(super) fn validate_traversal_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::BinaryOp { left, right, .. } => self.validate_binary_expr_refs(
                table,
                type_params,
                BinaryExprRef { left, right },
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::UnaryOp { operand, .. }
            | Expression::MemberAccess {
                object: operand, ..
            } => self.validate_unary_expr_refs(
                table,
                type_params,
                operand,
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::IndexAccess { object, index, .. } => self.validate_index_expr_refs(
                table,
                type_params,
                IndexExprRef { object, index },
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::WhileLoop {
                condition, body, ..
            }
            | Expression::If {
                condition,
                then_body: body,
                ..
            } => {
                let else_body = match expr {
                    Expression::If { else_body, .. } => else_body.as_deref(),
                    _ => None,
                };
                self.validate_if_or_while_expr_refs(
                    table,
                    type_params,
                    IfOrWhileExprRef {
                        condition,
                        body,
                        else_body,
                    },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_type_ref(
                    table,
                    type_params,
                    target_type,
                    *span,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StringInterpolation { parts, .. } => self
                .validate_string_interpolation_refs(
                    table,
                    type_params,
                    parts,
                    locals,
                    allow_self_type,
                    diagnostics,
                ),
            Expression::Range { start, end, .. } => self.validate_range_expr_refs(
                table,
                type_params,
                RangeExprRef { start, end },
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::Defer { expr, .. } => self.validate_defer_expr_refs(
                table,
                type_params,
                expr,
                locals,
                allow_self_type,
                diagnostics,
            ),
            _ => unreachable!("traversal dispatch received non-traversal expression"),
        }
    }
}
