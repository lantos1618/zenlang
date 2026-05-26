struct ResolverTypeBehaviorAssociationListTask<'a> {
    symbol: &'a Symbol,
    name: &'a str,
    impl_edges: Vec<ExpectedBehaviorEdge>,
    required_edges: Vec<ExpectedBehaviorEdge>,
    span: Span,
}

struct ResolverBehaviorParentListTask<'a> {
    symbol: &'a Symbol,
    name: &'a str,
    parent_edges: Vec<ExpectedBehaviorEdge>,
    span: Span,
}

#[derive(Default)]
struct ResolverBehaviorAssociationListTasks<'a> {
    type_associations: Vec<ResolverTypeBehaviorAssociationListTask<'a>>,
    behavior_parents: Vec<ResolverBehaviorParentListTask<'a>>,
}

#[derive(Default)]
struct ResolverExpectedSymbolSets {
    declarations: HashSet<(Namespace, String)>,
    locals: HashSet<(String, u32)>,
    validate_imports: bool,
}

#[derive(Default)]
struct ResolverContractReplayTasks<'a> {
    expected_symbols: ResolverExpectedSymbolSets,
    behavior_associations: ResolverBehaviorAssociationListTasks<'a>,
}

struct ResolverContractBehaviorAssociationSource<'a> {
    name: &'a str,
    symbol: &'a Symbol,
    span: Span,
}

struct ResolverContractReplayDeclarationTasks<'a> {
    expected_symbols: ResolverExpectedSymbolSets,
    expected_associations: ExpectedBehaviorAssociations,
    expected_parents: ExpectedBehaviorEdges,
    type_declarations: Vec<ResolverContractBehaviorAssociationSource<'a>>,
    behavior_declarations: Vec<ResolverContractBehaviorAssociationSource<'a>>,
}

impl Default for ResolverContractReplayDeclarationTasks<'_> {
    fn default() -> Self {
        Self {
            expected_symbols: ResolverExpectedSymbolSets::default(),
            expected_associations: ExpectedBehaviorAssociations {
                impls: ExpectedBehaviorEdges::default(),
                required: ExpectedBehaviorEdges::default(),
            },
            expected_parents: ExpectedBehaviorEdges::default(),
            type_declarations: Vec::new(),
            behavior_declarations: Vec::new(),
        }
    }
}

