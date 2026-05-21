use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

use super::TypeChecker;

impl TypeChecker {
    pub(in crate::typechecker) fn reject_missing_generic_substitutions(
        &mut self,
        kind: &str,
        name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> bool {
        let missing: Vec<&str> = type_params
            .iter()
            .map(String::as_str)
            .filter(|param| !substitutions.contains_key(*param))
            .collect();
        if missing.is_empty() {
            return false;
        }

        self.diagnostics.push(Diagnostic::error(
            "E5000",
            format!(
                "cannot infer type argument{} {} for generic {} `{}`",
                if missing.len() == 1 {
                    ""
                } else {
                    "s"
                },
                missing
                    .iter()
                    .map(|name| format!("`{name}`"))
                    .collect::<Vec<_>>()
                    .join(", "),
                kind,
                name
            ),
            span,
        ));
        true
    }

    pub(crate) fn type_param_substitutions(
        &mut self,
        type_params: &[String],
        type_args: &[AstType],
        kind: &str,
        name: &str,
        span: Span,
    ) -> HashMap<String, Type> {
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
        }

        type_params
            .iter()
            .zip(type_args.iter())
            .map(|(param, arg)| (param.clone(), self.resolve_type(arg)))
            .collect()
    }
}
