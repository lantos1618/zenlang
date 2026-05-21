use crate::ast::{Expression, StringPart, TypeParam};
use crate::error::Diagnostic;

use super::*;

mod control_flow;
pub(super) use control_flow::{IfOrWhileExprRef, RangeExprRef};

pub(super) struct BinaryExprRef<'a> {
    pub(super) left: &'a Expression,
    pub(super) right: &'a Expression,
}

pub(super) struct IndexExprRef<'a> {
    pub(super) object: &'a Expression,
    pub(super) index: &'a Expression,
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
}
