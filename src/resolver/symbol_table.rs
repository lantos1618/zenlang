use std::collections::HashMap;

use crate::ast::{type_param_names, AstType};
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

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    /// Mark every exportable symbol with `name` (value/type/behavior) public, as
    /// named in an `@export({ ... })` manifest. Returns true if at least one
    /// matching symbol was found.
    pub fn mark_public(&mut self, name: &str) -> bool {
        let mut found = false;
        for symbol in &mut self.symbols {
            if symbol.name == name
                && matches!(
                    symbol.namespace,
                    Namespace::Value | Namespace::Type | Namespace::Behavior
                )
            {
                symbol.is_public = true;
                found = true;
            }
        }
        found
    }
}

include!("symbol_table/definition_metadata.rs");
include!("symbol_table/definitions.rs");
include!("symbol_table/storage.rs");
include!("symbol_table/behavior_edges.rs");
