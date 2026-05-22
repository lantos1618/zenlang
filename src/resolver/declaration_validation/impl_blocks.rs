use crate::ast::{AstType, Declaration};
use crate::error::{Diagnostic, Span};

use super::super::metadata_helpers::behavior_ref_display;
use super::super::{BehaviorRefMetadata, Resolver, SymbolTable};

pub(super) struct ImplBlockValidationInput<'a> {
    pub(super) type_name: &'a str,
    pub(super) type_args: &'a [AstType],
    pub(super) behavior: Option<&'a str>,
    pub(super) behavior_type_args: &'a [AstType],
    pub(super) methods: &'a [Declaration],
    pub(super) span: Span,
}

impl Resolver {
    pub(super) fn validate_impl_block_declaration(
        &self,
        table: &mut SymbolTable,
        input: ImplBlockValidationInput<'_>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let ImplBlockValidationInput {
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            methods,
            span,
        } = input;

        let scoped_type_params = target_type_params(type_args);

        if !self.is_known_type_name(table, &scoped_type_params, type_name) {
            diagnostics.push(Diagnostic::error(
                "E0201",
                format!("unknown type symbol '{type_name}'"),
                span,
            ));
        }
        if let Some(behavior) = behavior {
            self.validate_behavior_impl_declaration(
                table,
                type_name,
                behavior,
                behavior_type_args,
                span,
                diagnostics,
            );
        }
        for type_arg in behavior_type_args {
            self.validate_type_ref(
                table,
                &scoped_type_params,
                type_arg,
                span,
                false,
                diagnostics,
            );
        }
        for method in methods {
            self.validate_impl_block_method(table, method, diagnostics);
        }
    }

    fn validate_behavior_impl_declaration(
        &self,
        table: &mut SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let behavior_known = self.is_known_behavior_name(table, behavior);
        if !behavior_known {
            diagnostics.push(Diagnostic::error(
                "E0202",
                format!("unknown behavior symbol '{behavior}'"),
                span,
            ));
        }
        if self.is_known_type_name(table, &[], type_name) && behavior_known {
            let behavior_display = behavior_ref_display(behavior, behavior_type_args);
            if !table.record_behavior_impl(
                type_name,
                BehaviorRefMetadata {
                    name: behavior.to_string(),
                    type_args: behavior_type_args.to_vec(),
                },
            ) {
                diagnostics.push(Diagnostic::error(
                    "E0217",
                    format!(
                        "duplicate behavior implementation `{behavior_display}` for `{type_name}`"
                    ),
                    span,
                ));
            }
        }
    }

    fn validate_impl_block_method(
        &self,
        table: &mut SymbolTable,
        method: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let Declaration::Function {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = method
        else {
            return;
        };

        self.validate_type_param_constraints(table, type_params, true, diagnostics);
        self.validate_params(table, type_params, params, true, diagnostics);
        if let Some(return_type) = return_type {
            self.validate_type_ref(table, type_params, return_type, *span, true, diagnostics);
        }
        let scope_id = table.new_scope();
        let mut locals = self.param_locals(table, params, scope_id, diagnostics);
        self.validate_expr_refs(table, type_params, body, &mut locals, true, diagnostics);
    }
}

fn target_type_params(type_args: &[AstType]) -> Vec<crate::ast::TypeParam> {
    type_args
        .iter()
        .filter_map(|arg| match arg {
            AstType::Named(name) => Some(crate::ast::TypeParam {
                name: name.clone(),
                constraint: None,
                constraint_type_args: Vec::new(),
                span: Span::dummy(),
            }),
            _ => None,
        })
        .collect()
}
