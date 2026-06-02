use super::*;

impl TypeChecker {
    pub(super) fn check_generic_behavior_impl_template(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
    ) {
        let type_params = named_type_arg_names(type_args);
        if type_params.len() != type_args.len() {
            self.push_error(
                E5001,
                format!(
                    "generic behavior implementation target `{type_name}` must use named type parameters"
                ),
                span,
            );
            return;
        }

        let Some(expected_type_arg_count) = self.type_params_for_type(type_name).map(<[_]>::len)
        else {
            return;
        };
        if !self.validate_type_arg_arity(
            "type",
            type_name,
            expected_type_arg_count,
            type_args,
            span,
        ) {
            return;
        }

        let scoped: HashSet<String> = type_params.iter().cloned().collect();
        let Some(behavior_substitutions) =
            self.behavior_type_arg_substitutions(behavior, behavior_type_args, &scoped, span)
        else {
            return;
        };
        if self.reject_generic_behavior_impl_coherence_conflict(
            type_name,
            type_args,
            behavior,
            behavior_type_args,
            span,
        ) {
            return;
        }
        let required_methods = self.behavior_methods_with_inherited_substituted(
            behavior,
            &behavior_substitutions,
            &mut HashSet::new(),
        );

        let behavior_display = behavior_ref_display(behavior, behavior_type_args);
        self.check_behavior_impl_methods(
            type_name,
            &behavior_display,
            &required_methods,
            methods,
            span,
            type_args,
        );
        let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
        self.behavior_refs_by_key
            .insert(behavior_ref.key.clone(), behavior_ref);
        self.generic_behavior_impls
            .push(GenericBehaviorImplTemplate {
                type_name: type_name.to_string(),
                type_params,
                behavior: behavior.to_string(),
                behavior_type_args: behavior_type_args.to_vec(),
            });
    }

    pub(super) fn reject_generic_behavior_impl_coherence_conflict(
        &mut self,
        type_name: &str,
        type_args: &[AstType],
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) -> bool {
        let behavior_ref = self.generic_behavior_impl_behavior_ref(
            type_name,
            &named_type_arg_names(type_args),
            behavior,
            behavior_type_args,
        );
        let target = behavior_ref_display(type_name, type_args);
        let behavior_key = &behavior_ref.key;
        let mut overlapping_impl = None;

        for implementation in self
            .generic_behavior_impls
            .iter()
            .filter(|implementation| implementation.type_name == type_name)
        {
            let existing = self.generic_behavior_impl_behavior_ref(
                type_name,
                &implementation.type_params,
                &implementation.behavior,
                &implementation.behavior_type_args,
            );
            if existing.key == behavior_ref.key {
                self.push_error(
                    E6003,
                    format!("duplicate implementation of behavior `{behavior_key}` for generic type `{target}`"),
                    span,
                );
                return true;
            }
            if overlapping_impl.is_none()
                && (self.behavior_ref_inherits_from_inner(
                    &existing,
                    &behavior_ref.key,
                    &mut HashSet::new(),
                ) || self.behavior_ref_inherits_from_inner(
                    &behavior_ref,
                    &existing.key,
                    &mut HashSet::new(),
                ))
            {
                overlapping_impl = Some(existing.key);
            }
        }
        if let Some(existing) = overlapping_impl {
            self.push_error(
                E6010,
                format!("overlapping implementations of behaviors `{existing}` and `{behavior_key}` for generic type `{target}`"),
                span,
            );
            return true;
        }

        false
    }

    fn generic_behavior_impl_behavior_ref(
        &self,
        type_name: &str,
        type_params: &[String],
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> BehaviorParentRef {
        let canonical_params = self.type_params_for_type(type_name).unwrap_or_default();
        let substitutions: HashMap<String, AstType> = type_params
            .iter()
            .enumerate()
            .map(|(index, name)| {
                let canonical = canonical_params.get(index).unwrap_or(name).clone();
                (name.clone(), AstType::Named(canonical))
            })
            .collect();
        let type_args = behavior_type_args
            .iter()
            .map(|arg| substitute_behavior_ast_type(arg, &substitutions))
            .collect::<Vec<_>>();
        self.behavior_parent_ref(behavior, &type_args)
    }
}
