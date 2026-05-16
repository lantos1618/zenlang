use std::collections::{HashMap, HashSet, VecDeque};

use crate::ast::{AstType, Declaration};
use crate::error::Span;
use crate::resolver::{BehaviorRefMetadata, Namespace, Symbol, SymbolTable};

use super::*;

impl TypeChecker {
    pub(crate) fn resolver_symbol_name_for(
        symbols: &SymbolTable,
        namespace: Namespace,
        name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(namespace, name)
            .or_else(|| Self::resolver_symbol_by_span(symbols, namespace, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| name.to_string())
    }

    pub(crate) fn resolver_method_signature_name_for(
        symbols: &SymbolTable,
        ast_key: &str,
        type_name: &str,
        span: Span,
    ) -> String {
        symbols
            .lookup(Namespace::Value, ast_key)
            .or_else(|| {
                let prefix = format!("{type_name}.");
                Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
                    symbol.name.starts_with(&prefix)
                })
            })
            .or_else(|| Self::resolver_method_signature_symbol_by_span(symbols, span))
            .map(|symbol| symbol.name.clone())
            .unwrap_or_else(|| ast_key.to_string())
    }

    pub(crate) fn resolver_symbol_by_span(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
    ) -> Option<&Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, namespace, span, |_| true)
    }

    pub(crate) fn resolver_method_signature_symbol_by_span(
        symbols: &SymbolTable,
        span: Span,
    ) -> Option<&Symbol> {
        Self::resolver_symbol_by_span_matching(symbols, Namespace::Value, span, |symbol| {
            is_method_signature_key(&symbol.name)
        })
    }

    pub(crate) fn resolver_symbol_by_span_matching(
        symbols: &SymbolTable,
        namespace: Namespace,
        span: Span,
        matches: impl Fn(&Symbol) -> bool,
    ) -> Option<&Symbol> {
        symbols.symbols().iter().find(|symbol| {
            symbol.namespace == namespace && symbol.definition_span == span && matches(symbol)
        })
    }

    pub(crate) fn resolver_impl_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        methods: &[Declaration],
        behavior_ref: Option<(&str, &[AstType])>,
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(type_name) = methods.iter().find_map(|method| {
            let Declaration::Function { span, .. } = method else {
                return None;
            };
            Self::resolver_method_signature_symbol_by_span(symbols, *span)
                .and_then(|symbol| method_signature_receiver_name(&symbol.name).map(str::to_string))
        }) {
            return type_name;
        }

        if let Some((behavior, behavior_type_args)) = behavior_ref {
            if let Some(candidate) = self.resolver_behavior_ref_owner_for(
                &self.resolver_behavior_impl_refs,
                &self.resolver_missing_behavior_impl_refs,
                behavior,
                behavior_type_args,
            ) {
                return candidate;
            }
        }

        type_name.to_string()
    }

    pub(crate) fn resolver_required_type_name_for(
        &self,
        symbols: &SymbolTable,
        type_name: &str,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> String {
        if symbols.lookup(Namespace::Type, type_name).is_some() {
            return type_name.to_string();
        }

        if let Some(candidate) = self.resolver_behavior_ref_owner_for(
            &self.resolver_behavior_required_refs,
            &self.resolver_missing_behavior_required_refs,
            behavior,
            behavior_type_args,
        ) {
            return candidate;
        }

        type_name.to_string()
    }

    pub(crate) fn resolver_behavior_ref_owner_for(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        missing_refs: &HashSet<String>,
        behavior: &str,
        behavior_type_args: &[AstType],
    ) -> Option<String> {
        let behavior_key = self.behavior_reference_key(behavior, behavior_type_args);
        self.unique_behavior_ref_owner_for_key(refs_by_type, &behavior_key)
            .or_else(|| self.unique_behavior_ref_owner(refs_by_type, |_| true))
            .or_else(|| Self::unique_owned_candidate(missing_refs.iter().cloned()))
    }

    fn unique_behavior_ref_owner_for_key(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        behavior_key: &str,
    ) -> Option<String> {
        self.unique_behavior_ref_owner(refs_by_type, |reference| {
            self.behavior_reference_matches_key(reference, behavior_key)
        })
    }

    fn behavior_reference_matches_key(
        &self,
        reference: &BehaviorRefMetadata,
        behavior_key: &str,
    ) -> bool {
        self.behavior_reference_key(&reference.name, &reference.type_args) == behavior_key
    }

    fn unique_behavior_ref_owner(
        &self,
        refs_by_type: &HashMap<String, VecDeque<BehaviorRefMetadata>>,
        matches: impl Fn(&BehaviorRefMetadata) -> bool,
    ) -> Option<String> {
        Self::unique_owned_candidate(refs_by_type.iter().filter_map(|(candidate_type, refs)| {
            refs.iter().any(&matches).then_some(candidate_type.clone())
        }))
    }

    fn unique_owned_candidate(mut candidates: impl Iterator<Item = String>) -> Option<String> {
        let candidate = candidates.next()?;
        candidates.next().is_none().then_some(candidate)
    }
}
