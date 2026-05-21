use super::*;

impl TypeChecker {
    pub(in crate::typechecker) fn type_implements_behavior(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> bool {
        if self
            .behavior_impls
            .contains(&(type_name.to_string(), behavior.to_string()))
        {
            return true;
        }

        self.behavior_impls
            .iter()
            .any(|(implemented_type, implemented_behavior)| {
                implemented_type == type_name
                    && self.behavior_inherits_from(implemented_behavior, behavior)
            })
    }

    pub(in crate::typechecker) fn behavior_inherits_from(
        &self,
        behavior: &str,
        parent: &str,
    ) -> bool {
        self.behavior_inherits_from_inner(behavior, parent, &mut HashSet::new())
    }

    pub(in crate::typechecker) fn behavior_inherits_from_inner(
        &self,
        behavior: &str,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(behavior.to_string()) {
            return false;
        }

        if self.behavior_extends_parent_matches(behavior, &HashMap::new(), parent, seen) {
            return true;
        }

        self.behavior_refs_by_key
            .get(behavior)
            .is_some_and(|behavior_ref| {
                let substitutions = self.behavior_type_param_substitutions(
                    &behavior_ref.behavior,
                    &behavior_ref.type_args,
                );
                self.behavior_extends_parent_matches(
                    &behavior_ref.behavior,
                    &substitutions,
                    parent,
                    seen,
                )
            })
    }

    pub(in crate::typechecker) fn behavior_extends_parent_matches(
        &self,
        behavior: &str,
        substitutions: &HashMap<String, AstType>,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents.iter().any(|candidate| {
                let candidate_args: Vec<AstType> = candidate
                    .type_args
                    .iter()
                    .map(|type_arg| substitute_behavior_ast_type(type_arg, substitutions))
                    .collect();
                let candidate_ref = self.behavior_parent_ref(&candidate.behavior, &candidate_args);
                candidate_ref.key == parent
                    || self.behavior_ref_inherits_from_inner(&candidate_ref, parent, seen)
            })
        })
    }

    pub(in crate::typechecker) fn behavior_ref_inherits_from_inner(
        &self,
        behavior_ref: &BehaviorParentRef,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        if !seen.insert(behavior_ref.key.clone()) {
            return false;
        }

        let substitutions =
            self.behavior_type_param_substitutions(&behavior_ref.behavior, &behavior_ref.type_args);
        self.behavior_extends_parent_matches(&behavior_ref.behavior, &substitutions, parent, seen)
    }
}
