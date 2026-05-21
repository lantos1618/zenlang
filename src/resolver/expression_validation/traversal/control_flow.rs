use crate::ast::{Expression, TypeParam};
use crate::error::Diagnostic;

use super::*;

pub(in crate::resolver::expression_validation) struct IfOrWhileExprRef<'a> {
    pub(in crate::resolver::expression_validation) condition: &'a Expression,
    pub(in crate::resolver::expression_validation) body: &'a Expression,
    pub(in crate::resolver::expression_validation) else_body: Option<&'a Expression>,
}

pub(in crate::resolver::expression_validation) struct RangeExprRef<'a> {
    pub(in crate::resolver::expression_validation) start: &'a Expression,
    pub(in crate::resolver::expression_validation) end: &'a Expression,
}

impl Resolver {
    pub(in crate::resolver::expression_validation) fn validate_if_or_while_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: IfOrWhileExprRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            expr.condition,
            locals,
            allow_self_type,
            diagnostics,
        );
        self.validate_child_scope_expr_refs(
            table,
            type_params,
            expr.body,
            locals,
            allow_self_type,
            diagnostics,
        );
        if let Some(else_body) = expr.else_body {
            self.validate_child_scope_expr_refs(
                table,
                type_params,
                else_body,
                locals,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(in crate::resolver::expression_validation) fn validate_range_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: RangeExprRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            expr.start,
            locals,
            allow_self_type,
            diagnostics,
        );
        self.validate_expr_refs(
            table,
            type_params,
            expr.end,
            locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(in crate::resolver::expression_validation) fn validate_defer_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            expr,
            locals,
            allow_self_type,
            diagnostics,
        );
    }
}
