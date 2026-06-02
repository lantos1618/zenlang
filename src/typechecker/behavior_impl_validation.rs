mod generic_templates;

use super::*;

impl TypeChecker {
    pub(super) fn check_behavior_impl(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.push_error(E6005, format!("undefined type `{}`", type_name), span);
            return;
        }

        if !type_args.is_empty() {
            self.check_generic_behavior_impl_template(
                type_name,
                type_args,
                behavior,
                behavior_type_args,
                methods,
                span,
            );
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        let Some(behavior_substitutions) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        let mut overlapping_impl = None;
        for (_, implemented_behavior) in self
            .behavior_impls
            .iter()
            .filter(|(implemented_type, _)| implemented_type == type_name)
        {
            if implemented_behavior == &behavior_key {
                self.push_error(
                    E6003,
                    format!("duplicate implementation of behavior `{behavior_key}` for type `{type_name}`"),
                    span,
                );
                return;
            }
            if overlapping_impl.is_none()
                && (self.behavior_inherits_from(implemented_behavior, &behavior_key)
                    || self.behavior_inherits_from(&behavior_key, implemented_behavior))
            {
                overlapping_impl = Some(implemented_behavior.clone());
            }
        }
        if let Some(existing) = overlapping_impl {
            self.push_error(
                E6010,
                format!("overlapping implementations of behaviors `{existing}` and `{behavior_key}` for type `{type_name}`"),
                span,
            );
            return;
        }

        self.behavior_impls
            .insert((type_name.to_string(), behavior_key.clone()));
        self.behavior_refs_by_key.insert(
            behavior_key.clone(),
            self.behavior_parent_ref(behavior, behavior_type_args),
        );
        let required_methods = self.behavior_methods_with_inherited_substituted(
            behavior,
            &behavior_substitutions,
            &mut HashSet::new(),
        );

        self.check_behavior_impl_methods(
            type_name,
            &behavior_key,
            &required_methods,
            methods,
            span,
            &[],
        );
    }

    pub(super) fn check_behavior_impl_methods(
        &mut self,
        type_name: &str,
        behavior_key: &str,
        required_methods: &[crate::ast::BehaviorMethod],
        methods: &[Declaration],
        span: Span,
        target_type_args: &[AstType],
    ) {
        let actual_methods = methods
            .iter()
            .filter_map(Declaration::as_callable)
            .collect::<Vec<_>>();

        for method in &actual_methods {
            if !required_methods
                .iter()
                .any(|required| required.name == method.name)
            {
                self.push_error(
                    E6005,
                    format!(
                        "method `{}` is not declared by behavior `{behavior_key}`",
                        method.name
                    ),
                    method.span,
                );
            }
        }

        let target_label = if target_type_args.is_empty() {
            "type"
        } else {
            "generic type"
        };
        for required in required_methods {
            let method_name = &required.name;
            let Some(actual) = actual_methods
                .iter()
                .find(|method| method.name == required.name)
            else {
                if required.default_body.is_some() {
                    continue;
                }
                self.push_error(
                    E6001,
                    format!("{target_label} `{type_name}` implementation of `{behavior_key}` is missing required method `{method_name}`"),
                    span,
                );
                continue;
            };

            let actual_return = actual.return_type.clone().unwrap_or(AstType::Void);

            if actual.params.len() != required.params.len() {
                let expected_count = required.params.len();
                let actual_count = actual.params.len();
                self.push_error(
                    E6002,
                    format!("method `{method_name}` for behavior `{behavior_key}` expects {expected_count} parameters, found {actual_count}"),
                    actual.span,
                );
                continue;
            }

            for (idx, (expected, actual_param)) in
                required.params.iter().zip(actual.params).enumerate()
            {
                let expected_ty =
                    concrete_self_ast_type_for_target(&expected.ty, type_name, target_type_args);
                let actual_ty = concrete_self_ast_type_for_target(
                    &actual_param.ty,
                    type_name,
                    target_type_args,
                );
                if expected_ty != actual_ty {
                    let position = idx + 1;
                    let expected_display = expected_ty.display_name();
                    let actual_display = actual_param.ty.display_name();
                    self.push_error(
                        E6002,
                        format!("parameter {position} for method `{method_name}` in behavior `{behavior_key}` expects `{expected_display}`, found `{actual_display}`"),
                        actual_param.span,
                    );
                }
            }

            let expected_return = required.return_type.as_ref().unwrap_or(&AstType::Void);
            let expected_return =
                concrete_self_ast_type_for_target(expected_return, type_name, target_type_args);
            let actual_return =
                concrete_self_ast_type_for_target(&actual_return, type_name, target_type_args);
            if expected_return != actual_return {
                let expected_display = expected_return.display_name();
                let actual_display = actual_return.display_name();
                self.push_error(
                    E6002,
                    format!("method `{method_name}` for behavior `{behavior_key}` expects return `{expected_display}`, found `{actual_display}`"),
                    actual.span,
                );
            }
        }
    }
}
