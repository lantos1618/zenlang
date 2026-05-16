use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::AstType;
use crate::resolver::{BehaviorRefMetadata, Namespace, Symbol, SymbolTable};

use super::*;

impl TypeChecker {
    pub(crate) fn collect_resolver_behavior_parents(&mut self, symbols: &SymbolTable, name: &str) {
        let Some((parent_refs, definition_span)) =
            Self::resolver_behavior_refs(symbols, Namespace::Behavior, name, |symbol| {
                &symbol.behavior_parent_refs
            })
            .map(|(refs, symbol)| (refs, symbol.definition_span))
        else {
            return;
        };

        let parents = self.behavior_parent_refs_from_metadata(parent_refs);
        self.behavior_extends.insert(name.to_string(), parents);
        self.behavior_extends_spans
            .entry(name.to_string())
            .or_insert(definition_span);
    }

    pub(crate) fn collect_resolver_type_behavior_impls(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
    ) {
        self.behavior_impls
            .retain(|(type_name, _)| type_name != name);
        let Some((impl_refs, _)) =
            Self::resolver_behavior_refs(symbols, Namespace::Type, name, |symbol| {
                &symbol.behavior_impl_refs
            })
        else {
            return;
        };

        for behavior in impl_refs {
            let behavior_ref = self.behavior_parent_ref(&behavior.name, &behavior.type_args);
            let implementation = (name.to_string(), behavior_ref.key.clone());
            self.behavior_impls.insert(implementation);
            self.behavior_refs_by_key
                .insert(behavior_ref.key.clone(), behavior_ref);
        }
    }

    pub(crate) fn collect_resolver_type_behavior_impl_refs(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
    ) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_impl_refs,
            &mut self.resolver_behavior_impl_refs,
            &mut self.resolver_missing_behavior_impl_refs,
        );
    }

    pub(crate) fn collect_resolver_type_behavior_requires(
        &mut self,
        symbols: &SymbolTable,
        name: &str,
    ) {
        Self::collect_resolver_type_behavior_refs(
            symbols,
            name,
            |symbol| &symbol.behavior_required_refs,
            &mut self.resolver_behavior_required_refs,
            &mut self.resolver_missing_behavior_required_refs,
        );
    }

    fn collect_resolver_type_behavior_refs(
        symbols: &SymbolTable,
        name: &str,
        select_refs: impl Fn(&Symbol) -> &Option<Vec<BehaviorRefMetadata>>,
        collected_refs: &mut HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &mut HashSet<String>,
    ) {
        let Some(symbol) = symbols.lookup(Namespace::Type, name) else {
            return;
        };

        if let Some(refs) = select_refs(symbol).as_deref() {
            collected_refs.insert(name.to_string(), refs.iter().cloned().collect());
        } else {
            missing_refs.insert(name.to_string());
        }
    }

    fn resolver_behavior_refs<'a>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_refs: impl Fn(&'a Symbol) -> &'a Option<Vec<BehaviorRefMetadata>>,
    ) -> Option<(&'a [BehaviorRefMetadata], &'a Symbol)> {
        let (symbol, refs) = Self::resolver_symbol_metadata(symbols, namespace, name, |symbol| {
            select_refs(symbol).as_deref()
        })?;

        Some((refs, symbol))
    }

    pub(crate) fn resolver_symbol_metadata<'a, T: ?Sized>(
        symbols: &'a SymbolTable,
        namespace: Namespace,
        name: &str,
        select_metadata: impl Fn(&'a Symbol) -> Option<&'a T>,
    ) -> Option<(&'a Symbol, &'a T)> {
        let symbol = symbols.lookup(namespace, name)?;
        let metadata = select_metadata(symbol)?;
        Some((symbol, metadata))
    }

    fn behavior_parent_ref_from_metadata(
        &self,
        metadata: &BehaviorRefMetadata,
    ) -> BehaviorParentRef {
        self.behavior_parent_ref(&metadata.name, &metadata.type_args)
    }

    pub(crate) fn behavior_parent_refs_from_metadata(
        &self,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<BehaviorParentRef> {
        metadata
            .iter()
            .map(|parent| self.behavior_parent_ref_from_metadata(parent))
            .collect()
    }

    pub(crate) fn behavior_parent_ref(
        &self,
        behavior: &str,
        type_args: &[AstType],
    ) -> BehaviorParentRef {
        BehaviorParentRef {
            behavior: behavior.to_string(),
            type_args: type_args.to_vec(),
            key: self.behavior_reference_key(behavior, type_args),
        }
    }

    pub(crate) fn behavior_reference_key(&self, behavior: &str, type_args: &[AstType]) -> String {
        if type_args.is_empty() {
            behavior.to_string()
        } else {
            self.mangle_generic_type_name(behavior, type_args)
        }
    }

    pub(crate) fn insert_behavior_impl_ref(
        &mut self,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) {
        let behavior_ref = self.behavior_parent_ref(behavior, behavior_type_args);
        let behavior_key = behavior_ref.key.clone();
        self.behavior_impls
            .insert((type_name.to_string(), behavior_key));
        self.behavior_refs_by_key
            .insert(behavior_ref.key.clone(), behavior_ref);
    }

    #[cfg(test)]
    pub(crate) fn behavior_impl_refs_from_metadata(
        &self,
        type_name: &str,
        metadata: &[BehaviorRefMetadata],
    ) -> Vec<(String, String)> {
        metadata
            .iter()
            .map(|behavior| {
                let behavior_ref = self.behavior_parent_ref(&behavior.name, &behavior.type_args);
                (type_name.to_string(), behavior_ref.key)
            })
            .collect()
    }
}
