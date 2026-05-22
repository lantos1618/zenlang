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
        let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
        if self.generic_behavior_impls.iter().any(|implementation| {
            implementation.type_name == type_name && {
                let existing = self.behavior_parent_ref(
                    &implementation.behavior,
                    &implementation.behavior_type_args,
                );
                existing.key == behavior_ref.key
            }
        }) {
            self.diagnostics.push(Diagnostic::error(
                "E6003",
                format!(
                    "duplicate implementation of behavior `{}` for generic type `{}`",
                    behavior_ref.key,
                    generic_target_display(type_name, type_args)
                ),
                span,
            ));
            return true;
        }

        if let Some(existing) =
            self.find_overlapping_generic_behavior_impl(type_name, &behavior_ref)
        {
            self.diagnostics.push(Diagnostic::error(
                "E6010",
                format!(
                    "overlapping implementations of behaviors `{}` and `{}` for generic type `{}`",
                    existing,
                    behavior_ref.key,
                    generic_target_display(type_name, type_args)
                ),
                span,
            ));
            return true;
        }

        false
    }

    pub(super) fn record_generic_behavior_impl_ref(
        &mut self,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) {
        let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
        self.behavior_refs_by_key
            .insert(behavior_ref.key.clone(), behavior_ref);
    }

    fn find_overlapping_generic_behavior_impl(
        &self,
        type_name: &str,
        behavior_ref: &BehaviorParentRef,
    ) -> Option<String> {
        self.generic_behavior_impls
            .iter()
            .filter(|implementation| implementation.type_name == type_name)
            .map(|implementation| {
                self.behavior_parent_ref(
                    &implementation.behavior,
                    &implementation.behavior_type_args,
                )
            })
            .find(|existing| {
                self.behavior_ref_inherits_from_inner(
                    existing,
                    &behavior_ref.key,
                    &mut HashSet::new(),
                ) || self.behavior_ref_inherits_from_inner(
                    behavior_ref,
                    &existing.key,
                    &mut HashSet::new(),
                )
            })
            .map(|existing| existing.key)
    }
}

fn generic_target_display(type_name: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        return type_name.to_string();
    }

    let args = type_args
        .iter()
        .map(AstType::display_name)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{type_name}<{args}>")
}
