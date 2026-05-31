use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;
use crate::error::Span;

use super::monomorphize_names::reserve_specialization_name;
use super::super::{quoted_list, GenericFunctionTemplate, TypeChecker};

impl TypeChecker {
    pub(crate) fn specialize_generic_function(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_functions.get(name).cloned()?;
        self.specialize_generic_callable("function", name, template, substitutions, span, None)
    }

    pub(crate) fn specialize_generic_method(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_methods.get(name).cloned()?;
        let self_type = self.generic_method_self_type(name, substitutions);
        self.specialize_generic_callable("method", name, template, substitutions, span, self_type)
    }

    fn specialize_generic_callable(
        &mut self,
        kind: &str,
        name: &str,
        template: GenericFunctionTemplate,
        substitutions: &HashMap<String, Type>,
        span: Span,
        self_type: Option<Type>,
    ) -> Option<String> {
        let missing: Vec<&str> = template
            .type_params
            .iter()
            .map(String::as_str)
            .filter(|param| !substitutions.contains_key(*param))
            .collect();
        if !missing.is_empty() {
            self.push_error(
                crate::error::CompilerDiagnosticCode::E5000,
                format!(
                    "cannot infer type argument{} {} for generic {} `{}`",
                    if missing.len() == 1 {
                        ""
                    } else {
                        "s"
                    },
                    quoted_list(&missing),
                    kind,
                    name
                ),
                span,
            );
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        let specialization_key = self.generic_specialization_key(
            kind,
            template.dependencies.specialization_scope.as_deref(),
            &mangled,
        );
        if let Some(existing) = self.specializations_seen.get(&specialization_key) {
            return Some(existing.clone());
        }

        let mangled = reserve_specialization_name(
            &mut self.specializations_seen,
            &mut self.specialization_name_owners,
            &specialization_key,
            &mangled,
            template.dependencies.specialization_scope.as_deref(),
        );
        self.specialize_generic_template_body(&mangled, &template, substitutions, self_type);

        Some(mangled)
    }

    fn specialize_generic_template_body(
        &mut self,
        mangled: &str,
        template: &GenericFunctionTemplate,
        substitutions: &HashMap<String, Type>,
        self_type: Option<Type>,
    ) {
        let dependency_state = self.install_template_dependencies(template);
        let saved_self_type = std::mem::replace(&mut self.current_self_type, self_type);
        self.type_substitutions.push(substitutions.clone());
        let checked = self.check_function(
            mangled,
            &template.params,
            &template.return_type,
            &template.body,
            // Generic async functions are out of scope for milestone 1.
            false,
            &template.span,
        );
        self.type_substitutions.pop();
        match checked {
            Ok(function) => self.specialized_functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        self.restore_template_dependencies(dependency_state);
        self.current_self_type = saved_self_type;
    }

    pub(crate) fn type_param_substitutions(
        &mut self,
        type_params: &[String],
        type_args: &[AstType],
        kind: &str,
        name: &str,
        span: Span,
    ) -> HashMap<String, Type> {
        let type_args = self.fill_type_arg_defaults(name, type_args);
        if type_params.len() != type_args.len() {
            self.report_generic_type_arg_arity(
                kind,
                name,
                type_params.len(),
                type_args.len(),
                span,
            );
        }

        self.type_arg_substitutions(type_params, &type_args)
    }
}
