use super::*;

mod inheritance;

impl TypeChecker {
    pub(crate) fn behavior_parent_ref(
        &self,
        behavior: &str,
        type_args: &[AstType],
    ) -> BehaviorParentRef {
        BehaviorParentRef {
            behavior: behavior.to_string(),
            type_args: type_args.to_vec(),
            key: self.behavior_reference_key(behavior, type_args),
        }
    }

    pub(crate) fn behavior_reference_key(&self, behavior: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            behavior.to_string()
        } else {
            self.mangle_generic_type_name(behavior, type_args)
        }
    }

    pub(super) fn check_behavior_requires(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        if !self.structs.contains_key(type_name) && !self.enums.contains_key(type_name) {
            self.push_error(E6005, format!("undefined type `{}`", type_name), span);
            return;
        }

        if self.reject_unspecialized_generic_type(type_name, span) {
            return;
        }

        if self
            .behavior_type_arg_substitutions(behavior, behavior_type_args, &HashSet::new(), span)
            .is_none()
        {
            return;
        }
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if !self.type_implements_behavior(type_name, &behavior_key) {
            self.push_error(
                E6007,
                format!("type `{type_name}` does not implement required behavior `{behavior_key}`"),
                span,
            );
        }
    }

    pub(super) fn check_behavior_extends(
        &mut self,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
    ) {
        if !self.behaviors.contains_key(behavior) {
            self.push_error(E6006, format!("undefined behavior `{}`", behavior), span);
            return;
        }

        let scoped_type_params: HashSet<String> = self
            .behaviors
            .get(behavior)
            .map(|info| info.type_params.iter().cloned().collect())
            .unwrap_or_default();
        if self
            .behavior_type_arg_substitutions(parent, parent_type_args, &scoped_type_params, span)
            .is_none()
        {
            return;
        }

        let parent_ref = self.behavior_parent_ref(parent, parent_type_args);
        let parent_display = behavior_ref_display(parent, parent_type_args);
        let parents = self
            .behavior_extends
            .entry(behavior.to_string())
            .or_default();
        if parents
            .iter()
            .any(|existing| existing.key == parent_ref.key)
        {
            self.push_error(
                E6011,
                format!("duplicate behavior inheritance `{behavior}.extends({parent_display})`"),
                span,
            );
            return;
        }

        parents.push(parent_ref);
        self.behavior_extends_spans
            .entry(behavior.to_string())
            .or_insert(span);
    }
}
