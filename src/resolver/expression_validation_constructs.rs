use crate::ast::{AstType, Expression, MatchArm, TypeParam};
use crate::error::{Diagnostic, Span};

use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

mod aggregate_literals;
pub(super) use aggregate_literals::{EnumVariantRef, StructLiteralRef};
mod scoped_constructs;
pub(super) use scoped_constructs::{BlockRef, ClosureRef};

impl Resolver {
    pub(super) fn validate_type_arg_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        type_args: &[AstType],
        span: Span,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for type_arg in type_args {
            self.validate_type_ref(
                table,
                type_params,
                type_arg,
                span,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_expr_arg_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        args: &[Expression],
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for arg in args {
            self.validate_expr_refs(
                table,
                type_params,
                arg,
                locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_match_arm_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        arm: &MatchArm,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if let Some(guard) = &arm.guard {
            let arm_scope_id = table.new_scope();
            let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
            self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
            self.validate_expr_refs(
                table,
                type_params,
                guard,
                &mut arm_locals,
                allow_self_type,
                diagnostics,
            );
        }

        let arm_scope_id = table.new_scope();
        let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
        self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
        self.validate_expr_refs(
            table,
            type_params,
            &arm.body,
            &mut arm_locals,
            allow_self_type,
            diagnostics,
        );
    }
}
