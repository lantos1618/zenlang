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
