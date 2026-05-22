use crate::ast::Declaration;
use crate::error::Diagnostic;

use super::symbol_table::ScopeStack;
use super::{Resolver, SymbolTable};

mod behavior_associations;
mod impl_blocks;
mod type_declarations;

impl Resolver {
    pub(super) fn validate_declaration_types(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match decl {
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                self.validate_params(table, type_params, params, false, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        false,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, false, diagnostics);
            }
            Declaration::Method {
                type_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                self.validate_type_param_constraints(table, type_params, true, diagnostics);
                self.validate_params(table, type_params, params, true, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        true,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, true, diagnostics);
            }
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => {
                self.validate_struct_declaration(table, name, type_params, fields, diagnostics);
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.validate_enum_declaration(table, type_params, variants, diagnostics);
            }
            Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } => {
                self.validate_behavior_declaration(table, name, type_params, methods, diagnostics);
            }
            Declaration::ImplBlock {
                type_name,
                type_args,
                behavior,
                behavior_type_args,
                methods,
                span,
                ..
            } => {
                self.validate_impl_block_declaration(
                    table,
                    impl_blocks::ImplBlockValidationInput {
                        type_name,
                        type_args,
                        behavior: behavior.as_deref(),
                        behavior_type_args,
                        methods,
                        span: *span,
                    },
                    diagnostics,
                );
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                self.validate_requires_declaration(
                    table,
                    type_name,
                    behavior,
                    behavior_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::Derive {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                self.validate_derive_declaration(
                    table,
                    type_name,
                    behavior,
                    behavior_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } => {
                self.validate_behavior_extends_declaration(
                    table,
                    behavior,
                    parent,
                    parent_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::TopLevelExpr { expr, .. } => {
                let scope_id = table.new_scope();
                self.validate_expr_refs(
                    table,
                    &[],
                    expr,
                    &mut ScopeStack::new(scope_id),
                    false,
                    diagnostics,
                );
            }
        }
    }
}
