use crate::ast::{Expression, TypeParam};
use crate::error::Diagnostic;

use super::expression_validation_constructs::{
    BlockRef, ClosureRef, EnumVariantRef, StructLiteralRef,
};
use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

mod calls;
mod traversal;
use calls::{FunctionCallRef, MethodCallRef};
use traversal::{BinaryExprRef, IfOrWhileExprRef, IndexExprRef, RangeExprRef};

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
        match expr {
            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => {
                self.validate_function_call_expr_refs(
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
                );
            }
            Expression::Identifier { name, span } => {
                self.validate_identifier_expr_refs(table, name, *span, locals, diagnostics);
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_method_call_expr_refs(
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
                );
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_binary_expr_refs(
                    table,
                    type_params,
                    BinaryExprRef { left, right },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::UnaryOp { operand, .. }
            | Expression::MemberAccess {
                object: operand, ..
            } => {
                self.validate_unary_expr_refs(
                    table,
                    type_params,
                    operand,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_index_expr_refs(
                    table,
                    type_params,
                    IndexExprRef { object, index },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => {
                self.validate_struct_literal_refs(
                    table,
                    type_params,
                    StructLiteralRef {
                        name,
                        type_args,
                        fields,
                        span: *span,
                    },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => {
                self.validate_enum_variant_refs(
                    table,
                    type_params,
                    EnumVariantRef {
                        enum_name,
                        type_args,
                        variant,
                        payload: payload.as_deref(),
                        span: *span,
                    },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::ArrayLiteral { elements, .. } => {
                self.validate_expr_arg_refs(
                    table,
                    type_params,
                    elements,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    scrutinee,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                for arm in arms {
                    self.validate_match_arm_refs(
                        table,
                        type_params,
                        arm,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
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
            Expression::Loop { body, .. } => {
                self.validate_child_scope_expr_refs(
                    table,
                    type_params,
                    body,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Block {
                statements, expr, ..
            } => {
                self.validate_block_refs(
                    table,
                    type_params,
                    BlockRef {
                        statements,
                        expr: expr.as_deref(),
                    },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                self.validate_closure_refs(
                    table,
                    type_params,
                    ClosureRef {
                        params,
                        return_type: return_type.as_ref(),
                        body,
                        span: *span,
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
            Expression::StringInterpolation { parts, .. } => {
                self.validate_string_interpolation_refs(
                    table,
                    type_params,
                    parts,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Range { start, end, .. } => {
                self.validate_range_expr_refs(
                    table,
                    type_params,
                    RangeExprRef { start, end },
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Defer { expr, .. } => {
                self.validate_defer_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::LoopControl { .. }
            | Expression::Error { .. } => {}
        }
    }
}
