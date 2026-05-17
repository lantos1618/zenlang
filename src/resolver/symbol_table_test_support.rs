use super::*;
use crate::ast::AstType;

mod aggregate_metadata;
mod behavior_metadata;

impl SymbolTable {
    #[cfg(test)]
    pub(super) fn find_symbol_mut_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
    ) -> Option<&mut Symbol> {
        self.symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
    }

    #[cfg(test)]
    pub(crate) fn remove_for_test(&mut self, namespace: Namespace, name: &str) {
        self.symbols
            .retain(|symbol| symbol.namespace != namespace || symbol.name != name);
        self.by_name.clear();
        self.by_scoped_name.clear();
        for (idx, symbol) in self.symbols.iter_mut().enumerate() {
            symbol.id = SymbolId(idx as u32);
            if symbol.namespace != Namespace::Local {
                self.by_name
                    .insert((symbol.namespace, symbol.name.clone()), symbol.id);
            }
            self.by_scoped_name.insert(
                (symbol.namespace, symbol.name.clone(), symbol.scope_id),
                symbol.id,
            );
        }
    }

    #[cfg(test)]
    pub(crate) fn set_public_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.is_public = is_public;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_import_source_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        import_source: Option<String>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.import_source = import_source;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_for_test(&mut self, name: &str, is_mutable: Option<bool>) {
        if let Some(symbol) = self.find_symbol_mut_for_test(Namespace::Local, name) {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_in_scope_for_test(
        &mut self,
        name: &str,
        scope_id: u32,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self.symbols.iter_mut().find(|symbol| {
            symbol.namespace == Namespace::Local
                && symbol.name == name
                && symbol.scope_id == scope_id
        }) {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_mutability_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_count = parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_type_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_type_names = parameter_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_types_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_types: Option<Vec<AstType>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_types = parameter_types;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.parameter_names = parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type_name: Option<String>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.return_type_name = return_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type: Option<AstType>,
    ) {
        if let Some(symbol) = self.find_symbol_mut_for_test(namespace, name) {
            symbol.return_type = return_type;
        }
    }

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
