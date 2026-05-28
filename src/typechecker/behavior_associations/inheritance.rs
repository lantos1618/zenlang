use super::*;
impl TypeChecker {
    pub(in crate::typechecker) fn validate_behavior_extends_cycles(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        for behavior in behaviors {
            let mut visiting = HashSet::new();
            let mut visited = HashSet::new();
            if self.behavior_extends_has_cycle(&behavior, &mut visiting, &mut visited) {
                let span = self
                    .behavior_extends_spans
                    .get(&behavior)
                    .copied()
                    .unwrap_or_else(Span::dummy);
                self.push_error(
                    E6008,
                    format!("behavior inheritance cycle involving `{}`", behavior),
                    span,
                );
            }
        }
    }

    fn behavior_extends_has_cycle(
        &self,
        behavior: &str,
        visiting: &mut HashSet<String>,
        visited: &mut HashSet<String>,
    ) -> bool {
        if visiting.contains(behavior) {
            return true;
        }
        if !visited.insert(behavior.to_string()) {
            return false;
        }

        visiting.insert(behavior.to_string());
        let has_cycle = self.behavior_extends.get(behavior).is_some_and(|parents| {
            parents
                .iter()
                .any(|parent| self.behavior_extends_has_cycle(&parent.behavior, visiting, visited))
        });
        visiting.remove(behavior);
        has_cycle
    }

    pub(in crate::typechecker) fn validate_behavior_method_coherence(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        let mut diagnostics = Vec::new();

        for behavior in behaviors {
            let mut seen_methods: HashMap<String, ast::BehaviorMethod> = HashMap::new();
            for method in self.behavior_methods_with_inherited_substituted(
                &behavior,
                &HashMap::new(),
                &mut HashSet::new(),
            ) {
                if let Some(previous) = seen_methods.get(&method.name) {
                    let signatures_match = previous.return_type == method.return_type
                        && previous.params.len() == method.params.len()
                        && previous
                            .params
                            .iter()
                            .zip(&method.params)
                            .all(|(left, right)| {
                                left.mutable == right.mutable && left.ty == right.ty
                            });
                    if !signatures_match {
                        diagnostics.push(Diagnostic::error_code(
                            E6009,
                            format!(
                                "conflicting behavior method `{}` inherited by `{}`",
                                method.name, behavior
                            ),
                            method.span,
                        ));
                    }
                } else {
                    seen_methods.insert(method.name.clone(), method);
                }
            }
        }

        self.diagnostics.extend(diagnostics);
    }

    pub(in crate::typechecker) fn type_implements_behavior(
        &self,
        type_name: &str,
        behavior: &str,
    ) -> bool {
        self.behavior_impls
            .iter()
            .any(|(implemented_type, implemented_behavior)| {
                implemented_type == type_name
                    && (implemented_behavior == behavior
                        || self.behavior_inherits_from(implemented_behavior, behavior))
            })
    }

    pub(in crate::typechecker) fn behavior_inherits_from(
        &self,
        behavior: &str,
        parent: &str,
    ) -> bool {
        let mut seen = HashSet::new();
        seen.insert(behavior.to_string());

        self.behavior_extends_parent_matches(behavior, &HashMap::new(), parent, &mut seen)
            || self
                .behavior_refs_by_key
                .get(behavior)
                .is_some_and(|behavior_ref| {
                    self.behavior_ref_parent_matches(behavior_ref, parent, &mut seen)
                })
    }

    fn behavior_extends_parent_matches(
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

        self.behavior_ref_parent_matches(behavior_ref, parent, seen)
    }

    fn behavior_ref_parent_matches(
        &self,
        behavior_ref: &BehaviorParentRef,
        parent: &str,
        seen: &mut HashSet<String>,
    ) -> bool {
        let substitutions =
            self.behavior_type_param_substitutions(&behavior_ref.behavior, &behavior_ref.type_args);
        self.behavior_extends_parent_matches(&behavior_ref.behavior, &substitutions, parent, seen)
    }
}
