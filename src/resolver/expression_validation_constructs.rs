use crate::ast::{AstType, Expression, MatchArm, Param, Statement};
use crate::error::Span;

use super::expression_validation::ExprRefContext;
use super::symbol_table::ScopeStack;
use super::Resolver;

mod aggregate_literals;

impl Resolver {
    pub(super) fn validate_type_arg_refs(
        &self,
        type_args: &[AstType],
        span: Span,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        for type_arg in type_args {
            self.validate_expr_type_ref(type_arg, span, ctx);
        }
    }

    pub(super) fn validate_expr_type_ref(
        &self,
        ty: &AstType,
        span: Span,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        self.validate_type_ref(
            ctx.table,
            ctx.type_params,
            ty,
            span,
            ctx.allow_self_type,
            ctx.diagnostics,
        );
    }

    pub(super) fn validate_expr_arg_refs(
        &self,
        args: &[Expression],
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        for arg in args {
            self.validate_expr_refs_in(arg, ctx);
        }
    }

    pub(super) fn validate_match_arm_refs(&self, arm: &MatchArm, ctx: &mut ExprRefContext<'_, '_>) {
        let arm_scope_id = ctx.table.new_scope();
        let mut arm_locals = ScopeStack::with_parent(arm_scope_id, ctx.locals);
        self.bind_pattern_locals(ctx.table, &arm.pattern, &mut arm_locals, ctx.diagnostics);
        for expr in arm.guard.iter().chain(std::iter::once(&arm.body)) {
            self.validate_expr_refs_with_locals(expr, &mut arm_locals, ctx);
        }
    }

    pub(super) fn validate_child_scope_expr_refs(
        &self,
        expr: &Expression,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        let scope_id = ctx.table.new_scope();
        let mut child_locals = ScopeStack::with_parent(scope_id, ctx.locals);
        self.validate_expr_refs_with_locals(expr, &mut child_locals, ctx);
    }

    pub(super) fn validate_block_refs(
        &self,
        statements: &[Statement],
        expr: Option<&Expression>,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        let block_scope_id = ctx.table.new_scope();
        let mut block_locals = ScopeStack::with_parent(block_scope_id, ctx.locals);
        let mut block_ctx = ExprRefContext {
            table: ctx.table,
            type_params: ctx.type_params,
            locals: &mut block_locals,
            allow_self_type: ctx.allow_self_type,
            diagnostics: ctx.diagnostics,
        };
        for statement in statements {
            self.validate_statement_refs(statement, &mut block_ctx);
        }
        if let Some(expr) = expr {
            self.validate_expr_refs_in(expr, &mut block_ctx);
        }
    }

    pub(super) fn validate_closure_refs(
        &self,
        params: &[Param],
        return_type: Option<&AstType>,
        body: &Expression,
        span: Span,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        let closure_scope_id = ctx.table.new_scope();
        let mut closure_locals = ScopeStack::with_parent(closure_scope_id, ctx.locals);
        for param in params {
            self.validate_expr_type_ref(&param.ty, param.span, ctx);
            self.define_local_symbol(
                ctx.table,
                &param.name,
                param.mutable,
                param.span,
                &mut closure_locals,
                ctx.diagnostics,
            );
        }
        if let Some(return_type) = return_type {
            self.validate_expr_type_ref(return_type, span, ctx);
        }
        self.validate_expr_refs_with_locals(body, &mut closure_locals, ctx);
    }

    fn validate_expr_refs_with_locals(
        &self,
        expr: &Expression,
        locals: &mut ScopeStack,
        ctx: &mut ExprRefContext<'_, '_>,
    ) {
        let mut scoped_ctx = ExprRefContext {
            table: ctx.table,
            type_params: ctx.type_params,
            locals,
            allow_self_type: ctx.allow_self_type,
            diagnostics: ctx.diagnostics,
        };
        self.validate_expr_refs_in(expr, &mut scoped_ctx);
    }
}
