use crate::ast::{AstType, Expression, MatchArm, Param, Statement, TypeParam};
use crate::error::{Diagnostic, Span};

use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

mod aggregate_literals;
pub(super) use aggregate_literals::{EnumVariantRef, StructLiteralRef};

pub(super) struct BlockRef<'a> {
    pub statements: &'a [Statement],
    pub expr: Option<&'a Expression>,
}

pub(super) struct ClosureRef<'a> {
    pub params: &'a [Param],
    pub return_type: Option<&'a AstType>,
    pub body: &'a Expression,
    pub span: Span,
}

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

    pub(super) fn validate_child_scope_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let scope_id = table.new_scope();
        let mut child_locals = ScopeStack::with_parent(scope_id, locals);
        self.validate_expr_refs(
            table,
            type_params,
            expr,
            &mut child_locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_block_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        block: BlockRef<'_>,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let BlockRef { statements, expr } = block;

        let block_scope_id = table.new_scope();
        let mut block_locals = ScopeStack::with_parent(block_scope_id, locals);
        for statement in statements {
            self.validate_statement_refs(
                table,
                type_params,
                statement,
                &mut block_locals,
                allow_self_type,
                diagnostics,
            );
        }
        if let Some(expr) = expr {
            self.validate_expr_refs(
                table,
                type_params,
                expr,
                &mut block_locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_closure_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        closure: ClosureRef<'_>,
        locals: &ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let ClosureRef {
            params,
            return_type,
            body,
            span,
        } = closure;

        let closure_scope_id = table.new_scope();
        let mut closure_locals = ScopeStack::with_parent(closure_scope_id, locals);
        for param in params {
            self.validate_type_ref(
                table,
                type_params,
                &param.ty,
                param.span,
                allow_self_type,
                diagnostics,
            );
            self.define_local_symbol(
                table,
                &param.name,
                param.mutable,
                param.span,
                &mut closure_locals,
                diagnostics,
            );
        }
        if let Some(return_type) = return_type {
            self.validate_type_ref(
                table,
                type_params,
                return_type,
                span,
                allow_self_type,
                diagnostics,
            );
        }
        self.validate_expr_refs(
            table,
            type_params,
            body,
            &mut closure_locals,
            allow_self_type,
            diagnostics,
        );
    }
}
