use super::*;

impl SymbolTable {
    #[cfg(test)]
    pub(crate) fn set_behavior_method_signatures_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_method_signatures: Option<Vec<MethodSignatureMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_method_signatures = behavior_method_signatures;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_method_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_method_types: Option<Vec<BehaviorMethodTypeMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_method_types = behavior_method_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_parent_names = behavior_parent_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_parent_refs = behavior_parent_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_impl_names = behavior_impl_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_impl_refs = behavior_impl_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_required_names = behavior_required_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.behavior_required_refs = behavior_required_refs;
        }
    }
}
