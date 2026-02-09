//! Type alias resolution - Centralized handling of type aliases
//!
//! This module provides a centralized registry for type aliases, eliminating
//! the scattered alias handling found throughout the codebase.
//!
//! # Features
//!
//! - Chain resolution: A -> B -> C resolves to C
//! - Cycle detection: Prevents infinite loops in alias chains
//! - Caching: Resolved aliases are cached for performance
//! - Normalization: Replace all aliases with canonical types

use crate::ast::AstType;
use std::collections::HashMap;

/// Centralized registry for type aliases
#[derive(Debug, Clone, Default)]
pub struct TypeAliasRegistry {
    aliases: HashMap<String, AstType>,
    resolved_cache: HashMap<String, AstType>,
}

impl TypeAliasRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            aliases: HashMap::new(),
            resolved_cache: HashMap::new(),
        }
    }

    /// Register a type alias
    ///
    /// # Example
    /// ```
    /// registry.register("Int", AstType::I32);
    /// registry.register("StringPtr", AstType::ptr(AstType::StaticString));
    /// ```
    pub fn register(&mut self, name: &str, target: AstType) {
        self.aliases.insert(name.to_string(), target);
        // Invalidate cache since we added a new alias
        self.resolved_cache.clear();
    }

    /// Look up a type alias without resolving chains
    pub fn get(&self, name: &str) -> Option<&AstType> {
        self.aliases.get(name)
    }

    /// Check if a name is a registered alias
    pub fn is_alias(&self, name: &str) -> bool {
        self.aliases.contains_key(name)
    }

    /// Resolve a type alias, following chains
    ///
    /// If A -> B -> I32, then resolve("A") returns I32
    /// Uses caching for performance on repeated lookups
    pub fn resolve(&mut self, name: &str) -> Option<AstType> {
        // Check cache first
        if let Some(cached) = self.resolved_cache.get(name) {
            return Some(cached.clone());
        }

        // Resolve with cycle detection
        let mut visited = std::collections::HashSet::new();
        let result = self.resolve_with_cycle_detection(name, &mut visited);

        // Cache the result
        if let Some(ref ty) = result {
            self.resolved_cache.insert(name.to_string(), ty.clone());
        }

        result
    }

    fn resolve_with_cycle_detection(
        &self,
        name: &str,
        visited: &mut std::collections::HashSet<String>,
    ) -> Option<AstType> {
        // Cycle detection
        if visited.contains(name) {
            // Cycle detected - return the type as-is to break the cycle
            return Some(AstType::Generic {
                name: name.to_string(),
                type_args: vec![],
            });
        }

        let target = self.aliases.get(name)?;

        // Check if target is itself an alias
        if let AstType::Generic {
            name: inner_name,
            type_args,
        } = target
        {
            if type_args.is_empty() && self.aliases.contains_key(inner_name) {
                visited.insert(name.to_string());
                return self.resolve_with_cycle_detection(inner_name, visited);
            }
        }

        Some(target.clone())
    }

    /// Normalize a type, replacing all aliases with canonical types
    ///
    /// This recursively resolves all aliases in a type
    pub fn normalize(&mut self, ty: &AstType) -> AstType {
        match ty {
            // Check if this is an alias
            AstType::Generic { name, type_args } if type_args.is_empty() => {
                if let Some(resolved) = self.resolve(name) {
                    // Recursively normalize the resolved type
                    self.normalize(&resolved)
                } else {
                    ty.clone()
                }
            }

            // Recursively normalize composite types
            AstType::Slice(inner) => AstType::Slice(Box::new(self.normalize(inner))),
            AstType::FixedArray { element_type, size } => AstType::FixedArray {
                element_type: Box::new(self.normalize(element_type)),
                size: *size,
            },
            AstType::Function { args, return_type } => AstType::Function {
                args: args.iter().map(|t| self.normalize(t)).collect(),
                return_type: Box::new(self.normalize(return_type)),
            },
            AstType::FunctionPointer {
                param_types,
                return_type,
            } => AstType::FunctionPointer {
                param_types: param_types.iter().map(|t| self.normalize(t)).collect(),
                return_type: Box::new(self.normalize(return_type)),
            },
            AstType::Struct { name, fields } => AstType::Struct {
                name: name.clone(),
                fields: fields
                    .iter()
                    .map(|(n, t)| (n.clone(), self.normalize(t)))
                    .collect(),
            },
            AstType::Enum { name, variants } => AstType::Enum {
                name: name.clone(),
                variants: variants.clone(),
            },
            AstType::Ref(inner) => AstType::Ref(Box::new(self.normalize(inner))),
            AstType::Range {
                start_type,
                end_type,
                inclusive,
            } => AstType::Range {
                start_type: Box::new(self.normalize(start_type)),
                end_type: Box::new(self.normalize(end_type)),
                inclusive: *inclusive,
            },
            AstType::Generic { name, type_args } => AstType::Generic {
                name: name.clone(),
                type_args: type_args.iter().map(|t| self.normalize(t)).collect(),
            },

            // Primitive types - no normalization needed
            _ => ty.clone(),
        }
    }

    /// Get the canonical name for a type
    ///
    /// If the type is an alias, returns the underlying type name
    pub fn canonical_name(&mut self, name: &str) -> String {
        if let Some(resolved) = self.resolve(name) {
            if let Some(type_name) = resolved.get_type_name() {
                return type_name;
            }
        }
        name.to_string()
    }

    /// Merge another registry into this one
    pub fn merge(&mut self, other: &TypeAliasRegistry) {
        for (name, target) in &other.aliases {
            self.aliases.insert(name.clone(), target.clone());
        }
        self.resolved_cache.clear();
    }

    /// Get all registered aliases
    pub fn all_aliases(&self) -> &HashMap<String, AstType> {
        &self.aliases
    }

    /// Clear all aliases
    pub fn clear(&mut self) {
        self.aliases.clear();
        self.resolved_cache.clear();
    }

    /// Number of registered aliases
    pub fn len(&self) -> usize {
        self.aliases.len()
    }

    /// Check if registry is empty
    pub fn is_empty(&self) -> bool {
        self.aliases.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_resolution() {
        let mut registry = TypeAliasRegistry::new();

        registry.register("Int", AstType::I32);

        assert_eq!(registry.resolve("Int"), Some(AstType::I32));
        assert_eq!(registry.resolve("Unknown"), None);
    }

    #[test]
    fn test_chain_resolution() {
        let mut registry = TypeAliasRegistry::new();

        registry.register(
            "A",
            AstType::Generic {
                name: "B".to_string(),
                type_args: vec![],
            },
        );
        registry.register("B", AstType::I32);

        assert_eq!(registry.resolve("A"), Some(AstType::I32));
    }

    #[test]
    fn test_cycle_detection() {
        let mut registry = TypeAliasRegistry::new();

        registry.register(
            "A",
            AstType::Generic {
                name: "B".to_string(),
                type_args: vec![],
            },
        );
        registry.register(
            "B",
            AstType::Generic {
                name: "A".to_string(),
                type_args: vec![],
            },
        );

        // Should break cycle and return something
        let result = registry.resolve("A");
        assert!(result.is_some());
    }

    #[test]
    fn test_normalize() {
        let mut registry = TypeAliasRegistry::new();

        registry.register("Int", AstType::I32);

        let input = AstType::Slice(Box::new(AstType::Generic {
            name: "Int".to_string(),
            type_args: vec![],
        }));

        let normalized = registry.normalize(&input);

        assert_eq!(normalized, AstType::Slice(Box::new(AstType::I32)));
    }
}
