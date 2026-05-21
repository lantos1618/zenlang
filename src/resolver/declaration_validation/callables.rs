use crate::ast::{AstType, Expression, Param, TypeParam};
use crate::error::{Diagnostic, Span};

use super::super::{Resolver, SymbolTable};

mod impl_blocks;

pub(super) use impl_blocks::ImplBlockDeclarationValidation;

pub(super) struct FunctionDeclarationValidation<'a> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_params: &'a [TypeParam],
    pub(super) params: &'a [Param],
    pub(super) return_type: &'a Option<AstType>,
    pub(super) body: &'a Expression,
    pub(super) span: Span,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

pub(super) struct MethodDeclarationValidation<'a> {
    pub(super) table: &'a mut SymbolTable,
    pub(super) type_name: &'a str,
    pub(super) type_params: &'a [TypeParam],
    pub(super) params: &'a [Param],
    pub(super) return_type: &'a Option<AstType>,
    pub(super) body: &'a Expression,
    pub(super) span: Span,
    pub(super) diagnostics: &'a mut Vec<Diagnostic>,
}

struct CallableValidation<'a> {
    table: &'a mut SymbolTable,
    type_params: &'a [TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
    self_type_allowed: bool,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl Resolver {
    pub(super) fn validate_function_declaration(&self, input: FunctionDeclarationValidation<'_>) {
        self.validate_callable_declaration(CallableValidation {
            table: input.table,
            type_params: input.type_params,
            params: input.params,
            return_type: input.return_type,
            body: input.body,
            span: input.span,
            self_type_allowed: false,
            diagnostics: input.diagnostics,
        });
    }

    pub(super) fn validate_method_declaration(&self, input: MethodDeclarationValidation<'_>) {
        if !self.is_known_type_name(input.table, &[], input.type_name) {
            input.diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{}'", input.type_name),
                input.span,
            ));
        }
        self.validate_callable_declaration(CallableValidation {
            table: input.table,
            type_params: input.type_params,
            params: input.params,
            return_type: input.return_type,
            body: input.body,
            span: input.span,
            self_type_allowed: true,
            diagnostics: input.diagnostics,
        });
    }

    fn validate_callable_declaration(&self, input: CallableValidation<'_>) {
        let table = input.table;
        let diagnostics = input.diagnostics;
        self.validate_type_param_constraints(
            table,
            input.type_params,
            input.self_type_allowed,
            diagnostics,
        );
        self.validate_params(
            table,
            input.type_params,
            input.params,
            input.self_type_allowed,
            diagnostics,
        );
        if let Some(return_type) = input.return_type {
            self.validate_type_ref(
                table,
                input.type_params,
                return_type,
                input.span,
                input.self_type_allowed,
                diagnostics,
            );
        }
        let scope_id = table.new_scope();
        let mut locals = self.param_locals(table, input.params, scope_id, diagnostics);
        self.validate_expr_refs(
            table,
            input.type_params,
            input.body,
            &mut locals,
            input.self_type_allowed,
            diagnostics,
        );
    }
}
