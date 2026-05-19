use std::collections::HashSet;

use crate::ast::{BehaviorMethod, EnumVariant, StructField, TypeParam};
use crate::error::Diagnostic;

use super::super::symbol_table::ScopeStack;
use super::super::{Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_struct_declaration(
        &self,
        table: &mut SymbolTable,
        name: &str,
        type_params: &[TypeParam],
        fields: &[StructField],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_type_param_constraints(table, type_params, false, diagnostics);
        let mut seen_fields = HashSet::new();
        for field in fields {
            if !seen_fields.insert(field.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0211",
                    format!("duplicate field `{}` for struct `{name}`", field.name),
                    field.span,
                ));
            }
            self.validate_type_ref(
                table,
                type_params,
                &field.ty,
                field.span,
                false,
                diagnostics,
            );
            if let Some(default) = &field.default {
                let scope_id = table.new_scope();
                let mut locals = ScopeStack::new(scope_id);
                self.validate_expr_refs(
                    table,
                    type_params,
                    default,
                    &mut locals,
                    false,
                    diagnostics,
                );
            }
        }
    }

    pub(super) fn validate_enum_declaration(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        variants: &[EnumVariant],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_type_param_constraints(table, type_params, false, diagnostics);
        for variant in variants {
            if let Some(payload) = &variant.payload {
                self.validate_type_ref(
                    table,
                    type_params,
                    payload,
                    variant.span,
                    false,
                    diagnostics,
                );
            }
        }
    }

    pub(super) fn validate_behavior_declaration(
        &self,
        table: &mut SymbolTable,
        name: &str,
        type_params: &[TypeParam],
        methods: &[BehaviorMethod],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_type_param_constraints(table, type_params, true, diagnostics);
        let mut seen_methods = HashSet::new();
        for method in methods {
            if !seen_methods.insert(method.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0212",
                    format!("duplicate behavior method `{}` in `{name}`", method.name),
                    method.span,
                ));
            }
            self.validate_params(table, type_params, &method.params, true, diagnostics);
            if let Some(return_type) = &method.return_type {
                self.validate_type_ref(
                    table,
                    type_params,
                    return_type,
                    method.span,
                    true,
                    diagnostics,
                );
            }
            if let Some(default_body) = &method.default_body {
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, &method.params, scope_id, diagnostics);
                self.validate_expr_refs(
                    table,
                    type_params,
                    default_body,
                    &mut locals,
                    true,
                    diagnostics,
                );
            }
        }
    }
}
