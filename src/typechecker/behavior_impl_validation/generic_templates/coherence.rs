use super::*;

impl TypeChecker {
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
