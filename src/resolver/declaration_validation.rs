use crate::ast::{declarations::CallableDeclaration, Declaration};
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
        if let Some(callable) = decl.as_callable() {
            let allow_self_type = match decl {
                Declaration::Method { type_name, .. } => {
                    if !self.is_known_type_name(table, &[], type_name) {
                        self.push_unknown_type_symbol(diagnostics, type_name, callable.span);
                    }
                    true
                }
                _ => false,
            };
            self.validate_callable_declaration_types(table, callable, allow_self_type, diagnostics);
            return;
        }

        match decl {
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
            Declaration::ImplBlock { .. } => {
                self.validate_impl_block_declaration(table, decl, diagnostics);
            }
            Declaration::Import { .. } => {}
            Declaration::Function { .. } | Declaration::Method { .. } => {}
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

    fn validate_callable_declaration_types(
        &self,
        table: &mut SymbolTable,
        callable: CallableDeclaration<'_>,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        self.validate_type_param_constraints(
            table,
            callable.type_params,
            allow_self_type,
            diagnostics,
        );
        self.validate_params(
            table,
            callable.type_params,
            callable.params,
            allow_self_type,
            diagnostics,
        );
        if let Some(return_type) = callable.return_type {
            self.validate_type_ref(
                table,
                callable.type_params,
                return_type,
                callable.span,
                allow_self_type,
                diagnostics,
            );
        }
        let mut locals = self.param_locals(table, callable.params, diagnostics);
        self.validate_expr_refs(
            table,
            callable.type_params,
            callable.body,
            &mut locals,
            allow_self_type,
            diagnostics,
        );
    }
}
