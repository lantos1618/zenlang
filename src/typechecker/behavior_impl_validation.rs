use super::*;

impl TypeChecker {
    pub(super) fn check_behavior_impl(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
        span: Span,
        symbols: Option<&SymbolTable>,
    ) {
        let resolver_impl_ref = self.resolver_impl_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_impl_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_impl_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_impl_ref.as_ref(), behavior, behavior_type_args);

        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.diagnostics.push(Diagnostic::error(
                "E6005",
                format!("undefined type `{}`", type_name),
                span,
            ));
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

        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior_key.clone()))
        {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for type `{}`",
                    behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        if let Some(existing) = self.find_overlapping_behavior_impl(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6010",
                format!(
                    "overlapping implementations of behaviors `{}` and `{}` for type `{}`",
                    existing, behavior_key, type_name
                ),
                span,
            ));
            return;
        }

        self.behavior_impls
            .insert((type_name.to_string(), behavior_key.clone()));
        self.behavior_refs_by_key.insert(
            behavior_key.clone(),
            self.behavior_parent_ref(behavior, behavior_type_args),
        );
        let required_methods =
            self.behavior_methods_for_impl(behavior, &behavior_substitutions, &mut HashSet::new());
        let mut unmatched_required: VecDeque<String> = required_methods
            .iter()
            .map(|required| required.name.clone())
            .collect();
        let effective_methods = self.effective_behavior_impl_methods(
            symbols,
            type_name,
            methods,
            &mut unmatched_required,
        );

        self.validate_behavior_impl_methods(
            type_name,
            &behavior_key,
            &required_methods,
            &effective_methods,
            span,
        );
    }
}
