impl SymbolTable {
    pub(super) fn record_behavior_parent(
        &mut self,
        behavior: &str,
        parent_ref: BehaviorRefMetadata,
    ) -> bool {
        self.record_behavior_edge(
            Namespace::Behavior,
            behavior,
            parent_ref,
            |symbol| &mut symbol.behavior_parent_refs,
        )
    }

    pub(super) fn record_behavior_impl(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        self.record_behavior_edge(
            Namespace::Type,
            type_name,
            behavior_ref,
            |symbol| &mut symbol.behavior_impl_refs,
        )
    }

    pub(super) fn record_behavior_required(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        self.record_behavior_edge(
            Namespace::Type,
            type_name,
            behavior_ref,
            |symbol| &mut symbol.behavior_required_refs,
        )
    }

    fn record_behavior_edge(
        &mut self,
        namespace: Namespace,
        symbol_name: &str,
        behavior_ref: BehaviorRefMetadata,
        behavior_refs: impl FnOnce(&mut Symbol) -> &mut Option<Vec<BehaviorRefMetadata>>,
    ) -> bool {
        let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == symbol_name)
        else {
            return true;
        };

        let refs = behavior_refs(symbol).get_or_insert_with(Vec::new);
        if refs.contains(&behavior_ref) {
            return false;
        }
        refs.push(behavior_ref);
        true
    }
}
