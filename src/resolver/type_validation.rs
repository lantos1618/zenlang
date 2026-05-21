use std::collections::HashSet;

use crate::ast::{Param, TypeParam};
use crate::error::{Diagnostic, Span};

use super::{Namespace, Resolver, SymbolTable};

mod type_refs;

impl Resolver {
    pub(super) fn validate_params(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        params: &[Param],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_params = HashSet::new();
        for param in params {
            if !seen_params.insert(param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0214",
                    format!("duplicate parameter `{}`", param.name),
                    param.span,
                ));
            }
            self.validate_type_ref(
                table,
                type_params,
                &param.ty,
                param.span,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_type_param_constraints(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_type_params = HashSet::new();
        for type_param in type_params {
            if !seen_type_params.insert(type_param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0213",
                    format!("duplicate type parameter `{}`", type_param.name),
                    type_param.span,
                ));
            }
            if let Some(constraint) = &type_param.constraint {
                if !self.is_known_behavior_name(table, constraint) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{constraint}'"),
                        type_param.span,
                    ));
                }
                for type_arg in &type_param.constraint_type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        type_param.span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
        }
    }

    pub(super) fn is_known_behavior_name(&self, table: &SymbolTable, name: &str) -> bool {
        table.lookup(Namespace::Behavior, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
    }

    pub(super) fn behavior_type_params_for_ref(
        &self,
        table: &SymbolTable,
        behavior: &str,
    ) -> Vec<TypeParam> {
        table
            .lookup(Namespace::Behavior, behavior)
            .and_then(|symbol| symbol.type_parameter_names.as_ref())
            .map(|names| {
                names
                    .iter()
                    .map(|name| TypeParam {
                        name: name.clone(),
                        constraint: None,
                        constraint_type_args: Vec::new(),
                        span: Span::dummy(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
