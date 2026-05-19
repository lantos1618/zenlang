use crate::ast::{Expression, TypeParam};
use crate::error::Diagnostic;

use super::super::expression_validation_constructs::{
    BlockRef, ClosureRef, EnumVariantRef, StructLiteralRef,
};
use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_construct_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => self.validate_struct_literal_refs(
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
            ),
            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => self.validate_enum_variant_refs(
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
            ),
            Expression::ArrayLiteral { elements, .. } => self.validate_expr_arg_refs(
                table,
                type_params,
                elements,
                locals,
                allow_self_type,
                diagnostics,
            ),
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
            Expression::Loop { body, .. } => self.validate_child_scope_expr_refs(
                table,
                type_params,
                body,
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::Block {
                statements, expr, ..
            } => self.validate_block_refs(
                table,
                type_params,
                BlockRef {
                    statements,
                    expr: expr.as_deref(),
                },
                locals,
                allow_self_type,
                diagnostics,
            ),
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => self.validate_closure_refs(
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
            ),
            _ => unreachable!("construct dispatch received non-construct expression"),
        }
    }
}
