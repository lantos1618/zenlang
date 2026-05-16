#[derive(Clone, Copy)]
struct BehaviorRefValidation {
    symbol_kind: &'static str,
    name_label: &'static str,
    ref_label: &'static str,
    name_code: &'static str,
    ref_code: &'static str,
}

#[derive(Clone, Copy)]
enum BehaviorRefRole {
    Parent,
    Impl,
    Required,
}

#[derive(Clone, Copy)]
enum BehaviorRefCheck {
    Contains,
    List,
}

impl BehaviorRefValidation {
    fn for_role(role: BehaviorRefRole, check: BehaviorRefCheck) -> Self {
        let (symbol_kind, name_label, ref_label) = Self::role_labels(role);
        let (name_code, ref_code) = Self::codes_for(role, check);
        Self {
            symbol_kind,
            name_label,
            ref_label,
            name_code,
            ref_code,
        }
    }

    fn role_labels(role: BehaviorRefRole) -> (&'static str, &'static str, &'static str) {
        match role {
            BehaviorRefRole::Parent => ("behavior", "parents", "parent refs"),
            BehaviorRefRole::Impl => ("type", "behavior impls", "behavior impl refs"),
            BehaviorRefRole::Required => ("type", "behavior requires", "behavior requires refs"),
        }
    }

    fn codes_for(role: BehaviorRefRole, check: BehaviorRefCheck) -> (&'static str, &'static str) {
        match (role, check) {
            (BehaviorRefRole::Parent, BehaviorRefCheck::Contains) => ("E0235", "E0245"),
            (BehaviorRefRole::Parent, BehaviorRefCheck::List) => ("E0240", "E0246"),
            (BehaviorRefRole::Impl, BehaviorRefCheck::Contains) => ("E0236", "E0247"),
            (BehaviorRefRole::Impl, BehaviorRefCheck::List) => ("E0238", "E0248"),
            (BehaviorRefRole::Required, BehaviorRefCheck::Contains) => ("E0237", "E0249"),
            (BehaviorRefRole::Required, BehaviorRefCheck::List) => ("E0239", "E0250"),
        }
    }

    fn contains_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn contains_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected to include '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }

    fn list_name_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.name_label
        )
    }

    fn list_ref_message(self, name: &str, actual: &str, expected: &str) -> String {
        format!(
            "resolver {} symbol '{name}' has {} '{actual}', expected '{expected}'",
            self.symbol_kind, self.ref_label
        )
    }
}

struct BehaviorRefActual<'a> {
    names: Option<&'a [String]>,
    refs: Option<&'a [BehaviorRefMetadata]>,
}

impl<'a> BehaviorRefActual<'a> {
    fn for_role(symbol: &'a Symbol, role: BehaviorRefRole) -> Self {
        let (names, refs) = Self::metadata_for_role(symbol, role);
        Self { names, refs }
    }

    fn metadata_for_role(
        symbol: &'a Symbol,
        role: BehaviorRefRole,
    ) -> (Option<&'a [String]>, Option<&'a [BehaviorRefMetadata]>) {
        match role {
            BehaviorRefRole::Parent => (
                symbol.behavior_parent_names.as_deref(),
                symbol.behavior_parent_refs.as_deref(),
            ),
            BehaviorRefRole::Impl => (
                symbol.behavior_impl_names.as_deref(),
                symbol.behavior_impl_refs.as_deref(),
            ),
            BehaviorRefRole::Required => (
                symbol.behavior_required_names.as_deref(),
                symbol.behavior_required_refs.as_deref(),
            ),
        }
    }

    fn contains_display(&self, expected: &str) -> bool {
        self.names
            .is_some_and(|names| names.iter().any(|name| name == expected))
    }

    fn contains_metadata(&self, expected: &BehaviorRefMetadata) -> bool {
        self.refs
            .is_some_and(|refs| refs.iter().any(|behavior| behavior == expected))
    }

    fn names_match(&self, expected: &[String]) -> bool {
        behavior_ref_names_match(self.names, expected)
    }

    fn refs_match(&self, expected: &[BehaviorRefMetadata]) -> bool {
        behavior_refs_match(self.refs, expected)
    }
}

#[derive(Clone)]
struct ExpectedBehaviorEdge {
    display: String,
    metadata: BehaviorRefMetadata,
}

impl ExpectedBehaviorEdge {
    fn new(behavior: &str, type_args: &[AstType]) -> Self {
        Self {
            display: behavior_ref_display(behavior, type_args),
            metadata: BehaviorRefMetadata {
                name: behavior.to_string(),
                type_args: type_args.to_vec(),
            },
        }
    }
}

struct ExpectedBehaviorEdgeMetadata {
    names: Vec<String>,
    refs: Vec<BehaviorRefMetadata>,
}

impl ExpectedBehaviorEdgeMetadata {
    fn from_edges(edges: &[ExpectedBehaviorEdge]) -> Self {
        Self {
            names: edges.iter().map(|edge| edge.display.clone()).collect(),
            refs: edges.iter().map(|edge| edge.metadata.clone()).collect(),
        }
    }
}

#[derive(Default)]
struct ExpectedBehaviorEdges {
    edges: HashMap<String, Vec<ExpectedBehaviorEdge>>,
}

impl ExpectedBehaviorEdges {
    #[cfg(test)]
    fn parents_from(program: &ast::Program) -> Self {
        let mut expected = Self::default();
        for decl in &program.declarations {
            if let Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                ..
            } = decl
            {
                expected.push(behavior, parent, parent_type_args);
            }
        }
        expected
    }

    fn push(&mut self, owner: &str, behavior: &str, type_args: &[AstType]) {
        self.edges
            .entry(owner.to_string())
            .or_default()
            .push(ExpectedBehaviorEdge::new(behavior, type_args));
    }

    #[cfg(test)]
    fn edges_for(&self, owner: &str) -> &[ExpectedBehaviorEdge] {
        self.edges.get(owner).map(Vec::as_slice).unwrap_or(&[])
    }

    fn owned_edges_for(&self, owner: &str) -> Vec<ExpectedBehaviorEdge> {
        self.edges.get(owner).cloned().unwrap_or_default()
    }
}

struct ExpectedBehaviorAssociations {
    impls: ExpectedBehaviorEdges,
    required: ExpectedBehaviorEdges,
}

#[cfg(test)]
impl ExpectedBehaviorAssociations {
    fn new(program: &ast::Program) -> Self {
        let mut expected = Self {
            impls: ExpectedBehaviorEdges::default(),
            required: ExpectedBehaviorEdges::default(),
        };
        for decl in &program.declarations {
            match decl {
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(behavior),
                    behavior_type_args,
                    ..
                } => {
                    push_expected_behavior_impl_edge(
                        &mut expected,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                Declaration::Requires {
                    type_name,
                    behavior,
                    behavior_type_args,
                    ..
                } => {
                    push_expected_behavior_required_edge(
                        &mut expected,
                        type_name,
                        behavior,
                        behavior_type_args,
                    );
                }
                _ => {}
            }
        }
        expected
    }
}
