use crate::ast::{AstType, Expression, Param, Statement, TypeParam};
use crate::error::{Diagnostic, Span};

use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};

pub(in crate::resolver) struct BlockRef<'a> {
    pub statements: &'a [Statement],
    pub expr: Option<&'a Expression>,
}

pub(in crate::resolver) struct ClosureRef<'a> {
    pub params: &'a [Param],
    pub return_type: Option<&'a AstType>,
    pub body: &'a Expression,
    pub span: Span,
}

impl Resolver {
    pub(in crate::resolver) fn validate_child_scope_expr_refs(
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

    pub(in crate::resolver) fn validate_block_refs(
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

    pub(in crate::resolver) fn validate_closure_refs(
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
