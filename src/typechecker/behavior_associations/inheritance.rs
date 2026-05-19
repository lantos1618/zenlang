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
                self.diagnostics.push(Diagnostic::error(
                    "E6008",
                    format!("behavior inheritance cycle involving `{}`", behavior),
                    span,
                ));
            }
        }
    }

    pub(in crate::typechecker) fn behavior_extends_has_cycle(
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
                .any(|parent| self.behavior_extends_has_cycle(&parent.key, visiting, visited))
        });
        visiting.remove(behavior);
        has_cycle
    }

    pub(in crate::typechecker) fn validate_behavior_method_coherence(&mut self) {
        let behaviors: Vec<String> = self.behavior_extends.keys().cloned().collect();
        let mut diagnostics = Vec::new();

        for behavior in behaviors {
            let mut seen_behaviors = HashSet::new();
            let mut seen_methods = HashMap::new();
            self.collect_behavior_method_coherence_errors(
                &behavior,
                &behavior,
                &HashMap::new(),
                &mut seen_behaviors,
                &mut seen_methods,
                &mut diagnostics,
            );
        }

        self.diagnostics.extend(diagnostics);
    }

    pub(in crate::typechecker) fn collect_behavior_method_coherence_errors(
        &self,
        behavior: &str,
        root_behavior: &str,
        substitutions: &HashMap<String, AstType>,
        seen_behaviors: &mut HashSet<String>,
        seen_methods: &mut HashMap<String, ast::BehaviorMethod>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if !self.mark_behavior_seen(behavior, substitutions, seen_behaviors) {
            return;
        }

        if let Some(parents) = self.behavior_extends.get(behavior) {
            for parent in parents {
                let parent_substitutions =
                    self.behavior_parent_type_param_substitutions(parent, substitutions);
                self.collect_behavior_method_coherence_errors(
                    &parent.behavior,
                    root_behavior,
                    &parent_substitutions,
                    seen_behaviors,
                    seen_methods,
                    diagnostics,
                );
            }
        }

        if let Some(info) = self.behaviors.get(behavior) {
            for method in &info.methods {
                let method = substituted_behavior_method_signature(method, substitutions);

                if let Some(previous) = seen_methods.get(&method.name) {
                    if !behavior_method_signatures_match(previous, &method) {
                        diagnostics.push(Diagnostic::error(
                            "E6009",
                            format!(
                                "conflicting behavior method `{}` inherited by `{}`",
                                method.name, root_behavior
                            ),
                            method.span,
                        ));
                    }
                } else {
                    seen_methods.insert(method.name.clone(), method);
                }
            }
        }
    }

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
