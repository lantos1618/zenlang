use super::metadata_helpers::behavior_ref_display;

impl SymbolTable {
    pub(super) fn record_behavior_parent(
        &mut self,
        behavior: &str,
        parent_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Behavior && symbol.name == behavior)
        {
            let parent = behavior_ref_display(&parent_ref.name, &parent_ref.type_args);
            let parents = symbol.behavior_parent_names.get_or_insert_with(Vec::new);
            if parents.iter().any(|recorded| recorded == &parent) {
                return false;
            }
            parents.push(parent);
            symbol
                .behavior_parent_refs
                .get_or_insert_with(Vec::new)
                .push(parent_ref);
        }
        true
    }

    pub(super) fn record_behavior_impl(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Type && symbol.name == type_name)
        {
            let behavior = behavior_ref_display(&behavior_ref.name, &behavior_ref.type_args);
            let impls = symbol.behavior_impl_names.get_or_insert_with(Vec::new);
            if impls.iter().any(|recorded| recorded == &behavior) {
                return false;
            }
            impls.push(behavior);
            symbol
                .behavior_impl_refs
                .get_or_insert_with(Vec::new)
                .push(behavior_ref);
        }
        true
    }

    pub(super) fn record_behavior_required(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Type && symbol.name == type_name)
        {
            let behavior = behavior_ref_display(&behavior_ref.name, &behavior_ref.type_args);
            let required = symbol.behavior_required_names.get_or_insert_with(Vec::new);
            if required.iter().any(|recorded| recorded == &behavior) {
                return false;
            }
            required.push(behavior);
            symbol
                .behavior_required_refs
                .get_or_insert_with(Vec::new)
                .push(behavior_ref);
        }
        true
    }
}
