#[derive(Default)]
struct ResolverScopeCursor {
    next_scope_id: u32,
}

impl ResolverScopeCursor {
    fn new_scope(&mut self) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::new(self.next_scope_id)
    }

    fn child_scope(&mut self, parent: &ResolverLocalScope) -> ResolverLocalScope {
        self.next_scope_id += 1;
        ResolverLocalScope::with_parent(self.next_scope_id, parent)
    }
}

#[derive(Clone)]
struct ResolverLocalScope {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ResolverLocalScope {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ResolverLocalScope) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}
