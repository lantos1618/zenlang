//! Monomorphization helpers for generic callable specialization.

use std::collections::HashMap;

use crate::ast::typed::Type;
use crate::ast::AstType;
use crate::error::{Diagnostic, Span};

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

    fn reject_missing_generic_substitutions(
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

    pub(crate) fn generic_method_self_type(
        &mut self,
        method_name: &str,
        substitutions: &HashMap<String, Type>,
    ) -> Option<Type> {
        let receiver_name = super::method_signature_receiver_name(method_name)?;
        if let Some(info) = self.structs.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        if let Some(info) = self.enums.get(receiver_name).cloned() {
            return Some(self.generic_receiver_self_type(
                receiver_name,
                &info.type_params,
                substitutions,
            ));
        }
        Some(self.resolve_type(&AstType::Named(receiver_name.to_string())))
    }

    fn generic_receiver_self_type(
        &mut self,
        receiver_name: &str,
        type_params: &[String],
        substitutions: &HashMap<String, Type>,
    ) -> Type {
        if type_params.is_empty() {
            return self.resolve_type(&AstType::Named(receiver_name.to_string()));
        }

        let type_args: Vec<AstType> = type_params
            .iter()
            .filter_map(|param| substitutions.get(param).map(|ty| self.type_to_ast_ref(ty)))
            .collect();
        if type_args.len() == type_params.len() {
            self.resolve_type(&AstType::Generic {
                name: receiver_name.to_string(),
                type_args,
            })
        } else {
            Type::Unknown
        }
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
