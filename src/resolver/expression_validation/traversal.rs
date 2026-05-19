use crate::ast::{Expression, StringPart, TypeParam};
use crate::error::Diagnostic;

use super::*;

pub(super) struct BinaryExprRef<'a> {
    pub(super) left: &'a Expression,
    pub(super) right: &'a Expression,
}

pub(super) struct IndexExprRef<'a> {
    pub(super) object: &'a Expression,
    pub(super) index: &'a Expression,
}

pub(super) struct IfOrWhileExprRef<'a> {
    pub(super) condition: &'a Expression,
    pub(super) body: &'a Expression,
    pub(super) else_body: Option<&'a Expression>,
}

pub(super) struct RangeExprRef<'a> {
    pub(super) start: &'a Expression,
    pub(super) end: &'a Expression,
}

impl Resolver {
    pub(super) fn validate_binary_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: BinaryExprRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            expr.left,
            locals,
            allow_self_type,
            diagnostics,
        );
        self.validate_expr_refs(
            table,
            type_params,
            expr.right,
            locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_unary_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        operand: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            operand,
            locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_index_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: IndexExprRef<'_>,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_expr_refs(
            table,
            type_params,
            expr.object,
            locals,
            allow_self_type,
            diagnostics,
        );
        self.validate_expr_refs(
            table,
            type_params,
            expr.index,
            locals,
            allow_self_type,
            diagnostics,
        );
    }

    pub(super) fn validate_if_or_while_expr_refs(
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

    pub(super) fn validate_string_interpolation_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        parts: &[StringPart],
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for part in parts {
            if let StringPart::Expr(expr) = part {
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
    }

    pub(super) fn validate_range_expr_refs(
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

    pub(super) fn validate_defer_expr_refs(
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
