//! TypeContext - Shared type information between compiler phases
//!
//! This module provides a bridge between the typechecker and codegen,
//! eliminating the need for duplicate type inference code.
//!
//! Pipeline: Parser → Typechecker → TypeContext → Codegen
//!
//! The typechecker populates TypeContext with:
//! - Function signatures (params, return types)
//! - Struct definitions (fields and their types)
//! - Enum definitions (variants and payloads)
//! - Method signatures from impl blocks
//! - Variable types per scope
//!
//! Codegen consumes TypeContext instead of re-inferring types.

use crate::ast::AstType;
use std::collections::HashMap;

/// Shared type context that flows from typechecker to codegen
#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    /// Function signatures: name -> (params, return_type)
    pub functions: HashMap<String, FunctionType>,

    /// Struct definitions: name -> fields
    pub structs: HashMap<String, Vec<(String, AstType)>>,

    /// Enum definitions: name -> variants
    pub enums: HashMap<String, Vec<(String, Option<AstType>)>>,

    /// Method signatures: "TypeName.method" -> return_type
    pub methods: HashMap<String, AstType>,

    /// Method parameter types: "TypeName.method" -> parameter list
    pub method_params: HashMap<String, Vec<(String, AstType)>>,

    /// Behavior implementations: type_name -> list of behavior names
    pub behavior_impls: HashMap<String, Vec<String>>,

    /// Constructor return types: "TypeName.new" or "TypeName<T>.new" -> return_type
    /// This helps codegen avoid re-inferring constructor results
    pub constructors: HashMap<String, AstType>,

    /// Variable types by scope: "function_name::var_name" -> type
    /// Populated during typechecking to avoid re-inference in codegen
    pub variables: HashMap<String, AstType>,

    /// Type aliases: "CompletionFn" -> function type
    /// Used to resolve type alias names to their underlying types
    pub type_aliases: HashMap<String, AstType>,
}

#[derive(Debug, Clone)]
pub struct FunctionType {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_external: bool,
}

impl TypeContext {
    pub fn new() -> Self {
        Self::default()
    }

    // ========================================================================
    // Registration methods (called by typechecker)
    // ========================================================================

    pub fn register_function(
        &mut self,
        name: String,
        params: Vec<(String, AstType)>,
        return_type: AstType,
        is_external: bool,
    ) {
        self.functions.insert(
            name,
            FunctionType {
                params,
                return_type,
                is_external,
            },
        );
    }

    pub fn register_struct(&mut self, name: String, fields: Vec<(String, AstType)>) {
        self.structs.insert(name, fields);
    }

    pub fn register_enum(&mut self, name: String, variants: Vec<(String, Option<AstType>)>) {
        self.enums.insert(name, variants);
    }

    pub fn register_method(&mut self, type_name: &str, method_name: &str, return_type: AstType) {
        let key = format!("{}.{}", type_name, method_name);
        self.methods.insert(key, return_type);
    }

    pub fn register_method_with_params(
        &mut self,
        type_name: &str,
        method_name: &str,
        params: Vec<(String, AstType)>,
        return_type: AstType,
    ) {
        let key = format!("{}.{}", type_name, method_name);
        self.methods.insert(key.clone(), return_type);
        self.method_params.insert(key, params);
    }

    pub fn register_behavior_impl(&mut self, type_name: &str, behavior_name: &str) {
        self.behavior_impls
            .entry(type_name.to_string())
            .or_default()
            .push(behavior_name.to_string());
    }

    /// Register a constructor return type (e.g., "Vec<i32>.new" -> Vec<i32>)
    pub fn register_constructor(
        &mut self,
        type_name: &str,
        method_name: &str,
        return_type: AstType,
    ) {
        let key = format!("{}.{}", type_name, method_name);
        self.constructors.insert(key, return_type);
    }

    /// Register a variable's type within a scope
    pub fn register_variable(&mut self, scope: &str, var_name: &str, var_type: AstType) {
        let key = format!("{}::{}", scope, var_name);
        self.variables.insert(key, var_type);
    }

    // ========================================================================
    // Query methods (called by codegen)
    // ========================================================================

    pub fn get_function_return_type(&self, name: &str) -> Option<AstType> {
        self.functions.get(name).map(|f| f.return_type.clone())
    }

    pub fn get_struct_fields(&self, name: &str) -> Option<&Vec<(String, AstType)>> {
        self.structs.get(name)
    }

    pub fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<AstType> {
        self.structs
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _)| n == field_name))
            .map(|(_, t)| t.clone())
    }

    pub fn get_enum_variants(&self, name: &str) -> Option<&Vec<(String, Option<AstType>)>> {
        self.enums.get(name)
    }

    pub fn get_method_return_type(&self, type_name: &str, method_name: &str) -> Option<AstType> {
        let key = format!("{}.{}", type_name, method_name);
        self.methods.get(&key).cloned()
    }

    pub fn get_method_params(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&Vec<(String, AstType)>> {
        let key = format!("{}.{}", type_name, method_name);
        self.method_params.get(&key)
    }

    pub fn get_function_params(&self, name: &str) -> Option<&Vec<(String, AstType)>> {
        self.functions.get(name).map(|f| &f.params)
    }

    pub fn get_enum_variant_type(
        &self,
        enum_name: &str,
        variant_name: &str,
    ) -> Option<Option<AstType>> {
        self.enums
            .get(enum_name)
            .and_then(|variants| variants.iter().find(|(n, _)| n == variant_name))
            .map(|(_, payload)| payload.clone())
    }

    pub fn type_implements_behavior(&self, type_name: &str, behavior_name: &str) -> bool {
        self.behavior_impls
            .get(type_name)
            .map(|impls| impls.iter().any(|b| b == behavior_name))
            .unwrap_or(false)
    }

    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }

    pub fn has_method(&self, type_name: &str, method_name: &str) -> bool {
        let key = format!("{}.{}", type_name, method_name);
        self.methods.contains_key(&key)
    }

    /// Get constructor return type
    pub fn get_constructor_type(&self, type_name: &str, method_name: &str) -> Option<AstType> {
        let key = format!("{}.{}", type_name, method_name);
        self.constructors.get(&key).cloned()
    }

    /// Get variable type within a scope
    pub fn get_variable_type(&self, scope: &str, var_name: &str) -> Option<AstType> {
        let key = format!("{}::{}", scope, var_name);
        self.variables.get(&key).cloned()
    }

    /// Check if a constructor exists
    pub fn has_constructor(&self, type_name: &str, method_name: &str) -> bool {
        let key = format!("{}.{}", type_name, method_name);
        self.constructors.contains_key(&key)
    }
}
