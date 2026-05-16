use super::*;

impl TypeChecker {
    pub(super) fn check_behavior_requires(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
        span: Span,
    ) {
        let resolver_required_ref = self.resolver_required_ref_for(type_name, behavior);
        if self.should_skip_missing_resolver_behavior_ref(
            resolver_required_ref.as_ref(),
            type_name,
            &self.resolver_missing_behavior_required_refs,
        ) {
            return;
        }
        let (behavior, behavior_type_args) =
            Self::behavior_ref_parts(resolver_required_ref.as_ref(), behavior, behavior_type_args);

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

        let Some(_) = self.behavior_type_arg_substitutions(
            behavior,
            behavior_type_args,
            &HashSet::new(),
            span,
        ) else {
            return;
        };
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);

        if !self.type_implements_behavior(type_name, &behavior_key) {
            self.diagnostics.push(Diagnostic::error(
                "E6007",
                format!(
                    "type `{}` does not implement required behavior `{}`",
                    type_name, behavior_key
                ),
                span,
            ));
        }
    }

    pub(super) fn resolver_required_ref_for(
        &mut self,
        type_name: &str,
        behavior: &str,
    ) -> Option<BehaviorRefMetadata> {
        self.resolver_behavior_ref_for(BehaviorRefRole::Required, type_name, behavior)
    }

    pub(super) fn check_behavior_extends(
        &mut self,
        behavior: &str,
        parent: &str,
        parent_type_args: &[AstType],
        span: Span,
    ) {
        if !self.behaviors.contains_key(behavior) {
            self.diagnostics.push(Diagnostic::error(
                "E6006",
                format!("undefined behavior `{}`", behavior),
                span,
            ));
            return;
        }

        let scoped_type_params: HashSet<String> = self
            .behaviors
            .get(behavior)
            .map(|info| info.type_params.iter().cloned().collect())
            .unwrap_or_default();
        let Some(_) = self.behavior_type_arg_substitutions(
            parent,
            parent_type_args,
            &scoped_type_params,
            span,
        ) else {
            return;
        };

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
            self.diagnostics.push(Diagnostic::error(
                "E6011",
                format!("duplicate behavior inheritance `{behavior}.extends({parent_display})`"),
                span,
            ));
            return;
        }

        parents.push(parent_ref);
        self.behavior_extends_spans
            .entry(behavior.to_string())
            .or_insert(span);
    }

    pub(super) fn validate_behavior_extends_cycles(&mut self) {
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

    pub(super) fn behavior_extends_has_cycle(
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

    pub(super) fn validate_behavior_method_coherence(&mut self) {
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

    pub(super) fn collect_behavior_method_coherence_errors(
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

    pub(super) fn type_implements_behavior(&self, type_name: &str, behavior: &str) -> bool {
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

    pub(super) fn behavior_inherits_from(&self, behavior: &str, parent: &str) -> bool {
        self.behavior_inherits_from_inner(behavior, parent, &mut HashSet::new())
    }

    pub(super) fn behavior_inherits_from_inner(
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

    pub(super) fn behavior_extends_parent_matches(
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

    pub(super) fn behavior_ref_inherits_from_inner(
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
