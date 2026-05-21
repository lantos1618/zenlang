//! Monomorphization helpers for generic callable specialization.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::error::Span;

pub(super) use super::monomorphize_types::concrete_name_matches_generic;
pub(crate) use super::monomorphize_types::type_to_ast;
use super::TypeChecker;

impl TypeChecker {
    pub(crate) fn specialize_generic_function(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_functions.get(name).cloned()?;
        if self.reject_missing_generic_substitutions(
            "function",
            name,
            &template.type_params,
            substitutions,
            span,
        ) {
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        self.specialize_generic_template_body(&mangled, &template, substitutions, None);

        Some(mangled)
    }

    pub(crate) fn specialize_generic_method(
        &mut self,
        name: &str,
        substitutions: &HashMap<String, Type>,
        span: Span,
    ) -> Option<String> {
        let template = self.generic_methods.get(name).cloned()?;
        if self.reject_missing_generic_substitutions(
            "method",
            name,
            &template.type_params,
            substitutions,
            span,
        ) {
            return None;
        }

        let mangled =
            self.generic_function_mangled_name(name, &template.type_params, substitutions);
        if self.specializations_seen.contains(&mangled) {
            return Some(mangled);
        }

        self.specializations_seen.insert(mangled.clone());
        let self_type = self.generic_method_self_type(name, substitutions);
        self.specialize_generic_template_body(&mangled, &template, substitutions, self_type);

        Some(mangled)
    }

    fn specialize_generic_template_body(
        &mut self,
        mangled: &str,
        template: &super::GenericFunctionTemplate,
        substitutions: &HashMap<String, Type>,
        self_type: Option<Type>,
    ) {
        let saved_return_type = self.current_return_type.clone();
        let saved_self_type = self.current_self_type.clone();
        let saved_defers = std::mem::take(&mut self.pending_defers);
        let dependency_state = self.install_template_dependencies(template);
        self.current_self_type = self_type;
        self.type_substitutions.push(substitutions.clone());
        match self.check_function(
            mangled,
            &template.params,
            &template.return_type,
            &template.body,
            &template.span,
        ) {
            Ok(function) => self.specialized_functions.push(function),
            Err(diagnostic) => self.diagnostics.push(diagnostic),
        }
        self.type_substitutions.pop();
        self.restore_template_dependencies(dependency_state);
        self.pending_defers = saved_defers;
        self.current_return_type = saved_return_type;
        self.current_self_type = saved_self_type;
    }
}
