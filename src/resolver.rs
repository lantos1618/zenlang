use std::collections::HashSet;

use crate::ast::{Declaration, Program};
use crate::error::Diagnostic;

#[cfg(test)]
mod symbol_table_test_support;

mod declaration_definition;
mod expression_validation;
mod expression_validation_constructs;
mod local_validation;
mod metadata_helpers;
mod symbol_table;
mod type_validation;

use metadata_helpers::behavior_ref_display;
use symbol_table::ScopeStack;
pub use symbol_table::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolId, SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_program(&self, program: &Program) -> Result<SymbolTable, Vec<Diagnostic>> {
        let mut table = SymbolTable::default();
        let mut diagnostics = Vec::new();

        for decl in &program.declarations {
            if let Err(diagnostic) = self.define_declaration(&mut table, decl) {
                diagnostics.push(*diagnostic);
            }
        }

        for decl in &program.declarations {
            self.validate_declaration_types(&mut table, decl, &mut diagnostics);
        }

        if diagnostics.is_empty() {
            Ok(table)
        } else {
            Err(diagnostics)
        }
    }

    fn validate_declaration_types(
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
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
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
            Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } => {
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
                        let mut locals =
                            self.param_locals(table, &method.params, scope_id, diagnostics);
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
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
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
                if let Some(behavior) = behavior {
                    let behavior_known = self.is_known_behavior_name(table, behavior);
                    if !behavior_known {
                        diagnostics.push(Diagnostic::error(
                            "E0202",
                            format!("unknown behavior symbol '{behavior}'"),
                            *span,
                        ));
                    }
                    if self.is_known_type_name(table, &[], type_name) && behavior_known {
                        let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                        if !table.record_behavior_impl(
                            type_name,
                            BehaviorRefMetadata {
                                name: behavior.clone(),
                                type_args: behavior_type_args.clone(),
                            },
                        ) {
                            diagnostics.push(Diagnostic::error(
                                "E0217",
                                format!(
                                    "duplicate behavior implementation `{behavior_display}` for `{type_name}`"
                                ),
                                *span,
                            ));
                        }
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
                for method in methods {
                    if let Declaration::Function {
                        type_params,
                        params,
                        return_type,
                        body,
                        span,
                        ..
                    } = method
                    {
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
                        self.validate_expr_refs(
                            table,
                            type_params,
                            body,
                            &mut locals,
                            true,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                if !self.is_known_behavior_name(table, behavior) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                } else if self.is_known_type_name(table, &[], type_name) {
                    let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                    if !table.record_behavior_required(
                        type_name,
                        BehaviorRefMetadata {
                            name: behavior.clone(),
                            type_args: behavior_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0216",
                            format!(
                                "duplicate required behavior `{behavior_display}` for `{type_name}`"
                            ),
                            *span,
                        ));
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
            }
            Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } => {
                let behavior_known = self.is_known_behavior_name(table, behavior);
                let parent_known = self.is_known_behavior_name(table, parent);
                if !behavior_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                }
                if !parent_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{parent}'"),
                        *span,
                    ));
                }
                let behavior_type_params = self.behavior_type_params_for_ref(table, behavior);
                for type_arg in parent_type_args {
                    self.validate_type_ref(
                        table,
                        &behavior_type_params,
                        type_arg,
                        *span,
                        false,
                        diagnostics,
                    );
                }
                if behavior_known && parent_known {
                    let parent_display = behavior_ref_display(parent, parent_type_args);
                    if !table.record_behavior_parent(
                        behavior,
                        BehaviorRefMetadata {
                            name: parent.clone(),
                            type_args: parent_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0215",
                            format!(
                                "duplicate behavior parent `{parent_display}` for `{behavior}`"
                            ),
                            *span,
                        ));
                    }
                }
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
