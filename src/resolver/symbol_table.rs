use std::collections::HashMap;

use crate::ast::AstType;
use crate::error::Span;

include!("symbol_table/core.rs");

impl SymbolTable {
    pub fn lookup(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        let id = self.by_name.get(&(namespace, name.to_string()))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn lookup_variant(&self, owner_name: &str, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|symbol| {
            symbol.namespace == Namespace::Variant
                && symbol.name == name
                && symbol.variant_owner_name.as_deref() == Some(owner_name)
        })
    }

    pub fn lookup_scoped(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
    }

    pub fn lookup_in_scope(
        &self,
        namespace: Namespace,
        name: &str,
        scope_id: u32,
    ) -> Option<&Symbol> {
        let id = self
            .by_scoped_name
            .get(&(namespace, name.to_string(), scope_id))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }
}

include!("symbol_table/definition_metadata.rs");
include!("symbol_table/definitions.rs");
include!("symbol_table/storage.rs");
include!("symbol_table/behavior_edges.rs");
