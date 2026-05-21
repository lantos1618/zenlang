use super::*;

impl SymbolTable {
    #[cfg(test)]
    pub(crate) fn set_type_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.type_parameter_count = type_parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.type_parameter_names = type_parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_bounds_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.type_parameter_bounds = type_parameter_bounds;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_bound_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_bound_refs: Option<Vec<TypeParameterBoundRefMetadata>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.type_parameter_bound_refs = type_parameter_bound_refs;
        }
    }
}
