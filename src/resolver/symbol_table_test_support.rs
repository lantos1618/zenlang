use super::*;

mod aggregate_metadata;
mod behavior_metadata;
mod type_parameters;
mod value_metadata;

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
}
