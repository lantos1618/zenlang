use super::*;
use crate::ast::{gated_builtin_type_name, is_builtin_type_name};

impl TypeChecker {
    pub(super) fn validate_named_type_ref_bounds(
        &mut self,
        name: &str,
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        if scoped_type_params.contains(name) {
            return;
        }

        if let Some(gated) = gated_builtin_type_name(name) {
            self.diagnostics
                .push(Diagnostic::error("E0202", gated.gate_message(), span));
            return;
        }

        if !self.is_known_named_type(name) {
            if reject_unknown {
                self.diagnostics.push(Diagnostic::error(
                    "E0201",
                    format!("unknown type symbol '{name}'"),
                    span,
                ));
            }
            return;
        }

        let generic = self
            .structs
            .get(name)
            .map(|info| ("struct", info.type_params.len()))
            .or_else(|| {
                self.enums
                    .get(name)
                    .map(|info| ("enum", info.type_params.len()))
            });
        if let Some((kind, type_param_count)) = generic {
            if type_param_count > 0 {
                self.diagnostics.push(Diagnostic::error(
                    "E5001",
                    format!(
                        "generic {} `{}` expects {} type arguments, found 0",
                        kind, name, type_param_count
                    ),
                    span,
                ));
            }
        }
    }

    pub(super) fn validate_generic_type_ref_with_args(
        &mut self,
        name: &str,
        type_args: &[AstType],
        scoped_type_params: &HashSet<String>,
        span: Span,
        reject_unknown: bool,
    ) {
        if let Some(gated) = gated_builtin_type_name(name) {
            self.diagnostics
                .push(Diagnostic::error("E0202", gated.gate_message(), span));
            return;
        }

        self.validate_generic_type_arg_refs_with_unknowns(
            type_args,
            scoped_type_params,
            span,
            reject_unknown,
        );

        if scoped_type_params.contains(name) {
            return;
        }

        let (kind, type_params, type_param_bounds) = if let Some(info) = self.structs.get(name) {
            (
                "struct",
                info.type_params.clone(),
                info.type_param_bounds.clone(),
            )
        } else if let Some(info) = self.enums.get(name) {
            (
                "enum",
                info.type_params.clone(),
                info.type_param_bounds.clone(),
            )
        } else {
            if reject_unknown && !self.imports.contains_key(name) {
                self.diagnostics.push(Diagnostic::error(
                    "E0201",
                    format!("unknown type symbol '{name}'"),
                    span,
                ));
            }
            return;
        };

        if type_params.is_empty() && !type_args.is_empty() {
            self.diagnostics.push(Diagnostic::error(
                "E5002",
                format!(
                    "non-generic {} `{}` does not accept type arguments",
                    kind, name
                ),
                span,
            ));
            return;
        }

        if type_params.len() != type_args.len() {
            self.diagnostics.push(Diagnostic::error(
                "E5001",
                format!(
                    "generic {} `{}` expects {} type arguments, found {}",
                    kind,
                    name,
                    type_params.len(),
                    type_args.len()
                ),
                span,
            ));
            return;
        }

        let substitutions: HashMap<String, Type> = type_params
            .iter()
            .zip(type_args.iter())
            .filter_map(|(param, arg)| {
                if ast_type_references_type_param(arg, scoped_type_params) {
                    None
                } else {
                    Some((param.clone(), self.resolve_type(arg)))
                }
            })
            .collect();
        self.check_generic_bounds(&type_param_bounds, &substitutions, span);
    }

    fn is_known_named_type(&self, name: &str) -> bool {
        if is_builtin_type_name(name) {
            return true;
        }
        self.structs.contains_key(name)
            || self.enums.contains_key(name)
            || self.imports.contains_key(name)
    }
}
