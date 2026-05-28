mod generic_templates;
mod methods;

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
}
