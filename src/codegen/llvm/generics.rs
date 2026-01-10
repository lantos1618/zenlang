use crate::ast::AstType;
use crate::type_context::TypeContext;
use crate::well_known::well_known;
use std::collections::HashMap;

/// Tracks generic type instantiations at different scopes
/// This allows nested generics like Result<Option<T>, Vec<E>>
#[derive(Debug, Clone)]
pub struct GenericTypeTracker {
    /// Stack of generic contexts, with the top being the current scope
    contexts: Vec<HashMap<String, AstType>>,
}

impl Default for GenericTypeTracker {
    fn default() -> Self {
        Self {
            contexts: vec![HashMap::new()],
        }
    }
}

impl GenericTypeTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a new generic context scope
    pub fn push_scope(&mut self) {
        self.contexts.push(HashMap::new());
    }

    /// Pop the current generic context scope
    pub fn pop_scope(&mut self) {
        if self.contexts.len() > 1 {
            self.contexts.pop();
        }
    }

    /// Insert a type mapping in the current scope
    pub fn insert(&mut self, key: String, type_: AstType) {
        if let Some(current) = self.contexts.last_mut() {
            current.insert(key, type_);
        }
    }

    /// Get a type mapping, searching from current scope upwards
    pub fn get(&self, key: &str) -> Option<&AstType> {
        for context in self.contexts.iter().rev() {
            if let Some(type_) = context.get(key) {
                return Some(type_);
            }
        }
        None
    }

    /// Extract and track generic types from a complex type
    /// Uses TypeContext to determine collection semantics instead of hardcoded type names.
    pub fn track_generic_type(&mut self, type_: &AstType, prefix: &str) {
        // Delegate to the version with TypeContext, using None for backward compatibility
        self.track_generic_type_with_ctx(type_, prefix, None);
    }

    /// Extract and track generic types from a complex type with TypeContext
    pub fn track_generic_type_with_ctx(&mut self, type_: &AstType, prefix: &str, type_ctx: Option<&TypeContext>) {
        match type_ {
            AstType::Generic { name, type_args } => {
                // Track the main generic type
                self.insert(format!("{}_type", prefix), type_.clone());

                // Track the generic name for easy lookup
                self.insert(
                    format!("{}_name", prefix),
                    AstType::EnumType { name: name.clone() },
                );

                // Track each type argument recursively
                let wk = well_known();
                if wk.is_result(name) && type_args.len() == 2 {
                    self.insert(format!("{}_Ok_Type", prefix), type_args[0].clone());
                    self.insert(format!("{}_Err_Type", prefix), type_args[1].clone());

                    // Recursively track nested generics
                    self.track_generic_type_with_ctx(&type_args[0], &format!("{}_Ok", prefix), type_ctx);
                    self.track_generic_type_with_ctx(&type_args[1], &format!("{}_Err", prefix), type_ctx);
                } else if wk.is_option(name) && type_args.len() == 1 {
                    self.insert(format!("{}_Some_Type", prefix), type_args[0].clone());

                    // Recursively track nested generics
                    self.track_generic_type_with_ctx(&type_args[0], &format!("{}_Some", prefix), type_ctx);
                } else if crate::type_context::is_single_element_collection(name) {
                    // Single element collection - track element type
                    if !type_args.is_empty() {
                        self.insert(format!("{}_Element_Type", prefix), type_args[0].clone());
                        self.track_generic_type_with_ctx(&type_args[0], &format!("{}_Element", prefix), type_ctx);
                    }
                } else if crate::type_context::is_key_value_collection(name) {
                    // Key-value collection - track key and value types
                    if type_args.len() >= 2 {
                        self.insert(format!("{}_Key_Type", prefix), type_args[0].clone());
                        self.insert(format!("{}_Value_Type", prefix), type_args[1].clone());
                        self.track_generic_type_with_ctx(&type_args[0], &format!("{}_Key", prefix), type_ctx);
                        self.track_generic_type_with_ctx(&type_args[1], &format!("{}_Value", prefix), type_ctx);
                    }
                } else {
                    // For other generic types, track type arguments by index
                    for (i, arg) in type_args.iter().enumerate() {
                        self.insert(format!("{}_arg{}_Type", prefix, i), arg.clone());
                        self.track_generic_type_with_ctx(arg, &format!("{}_arg{}", prefix, i), type_ctx);
                    }
                }
            }
            AstType::EnumType { name: _ } | AstType::Enum { name: _, .. } => {
                // Track custom enum types specially
                self.insert(format!("{}_type", prefix), type_.clone());
                self.insert(format!("{}_enum_name", prefix), type_.clone());
            }
            _ => {
                // For non-generic types, just track them directly
                self.insert(format!("{}_type", prefix), type_.clone());
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_generic_tracking() {
        let mut tracker = GenericTypeTracker::new();
        let wk = well_known();

        // Create a nested generic type: Result<Option<i32>, String>
        let nested_type = AstType::Generic {
            name: wk.result_name().to_string(),
            type_args: vec![
                AstType::Generic {
                    name: wk.option_name().to_string(),
                    type_args: vec![AstType::I32],
                },
                crate::ast::resolve_string_struct_type(),
            ],
        };

        tracker.track_generic_type(&nested_type, "test");

        // Check that all nested types are tracked
        assert!(tracker.get("test_Ok_Type").is_some());
        assert!(tracker.get("test_Err_Type").is_some());
        assert!(tracker.get("test_Ok_Some_Type").is_some());
        assert_eq!(tracker.get("test_Ok_Some_Type"), Some(&AstType::I32));
        assert_eq!(
            tracker.get("test_Err_Type"),
            Some(&crate::ast::resolve_string_struct_type())
        );
    }
}
