//! Unified type storage - Single Source of Truth for all type information
//!
//! This module eliminates duplication between TypeChecker, TypeContext, and TypeEnvironment
//! by providing a single storage location for all type information.
//!
//! # Architecture
//!
//! ```
//! TypeStore (single source of truth)
//!     ├── TypeChecker (mutably borrows for type checking)
//!     ├── TypeContext (immutably borrows for codegen)
//!     └── TypeEnvironment (queries for generic resolution)
//! ```
//!
//! # Benefits
//!
//! - No duplicate data structures
//! - Shared type information
//! - Consistent views across modules
//! - Reduced memory usage

use crate::ast::{AstType, Declaration, EnumDefinition, Function, StructDefinition};
use crate::name_utils;
use crate::typechecker::{EnumInfo, FunctionSignature, MethodSignature, StructInfo};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Unified storage for all type information
///
/// This is the single source of truth for:
/// - Struct definitions
/// - Enum definitions  
/// - Function signatures
/// - Type aliases
/// - Methods
/// - Variables
#[derive(Debug, Clone)]
pub struct TypeStore {
    // Core type definitions
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FunctionSignature>,

    // Type aliases: "AliasName" -> actual type
    type_aliases: HashMap<String, AstType>,

    // Methods: "TypeName::method" -> signature
    methods: HashMap<String, MethodSignature>,

    // Stdlib function signatures: "module::function"
    stdlib_functions: HashMap<String, FunctionSignature>,

    // Variable types per function: "function_name::var_name" -> type
    variables: HashMap<String, AstType>,

    // Generic definitions (for monomorphization)
    generic_functions: HashMap<String, Function>,
    generic_structs: HashMap<String, StructDefinition>,
    generic_enums: HashMap<String, EnumDefinition>,
}

impl TypeStore {
    /// Create a new empty TypeStore
    pub fn new() -> Self {
        Self {
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            type_aliases: HashMap::new(),
            methods: HashMap::new(),
            stdlib_functions: HashMap::new(),
            variables: HashMap::new(),
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            generic_enums: HashMap::new(),
        }
    }

    // ============================================================================
    // Struct Operations
    // ============================================================================

    pub fn register_struct(&mut self, name: &str, info: StructInfo) {
        self.structs.insert(name.to_string(), info);
    }

    pub fn get_struct(&self, name: &str) -> Option<&StructInfo> {
        self.structs.get(name)
    }

    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    pub fn get_all_structs(&self) -> &HashMap<String, StructInfo> {
        &self.structs
    }

    // ============================================================================
    // Enum Operations
    // ============================================================================

    pub fn register_enum(&mut self, name: &str, info: EnumInfo) {
        self.enums.insert(name.to_string(), info);
    }

    pub fn get_enum(&self, name: &str) -> Option<&EnumInfo> {
        self.enums.get(name)
    }

    pub fn has_enum(&self, name: &str) -> bool {
        self.enums.contains_key(name)
    }

    pub fn get_all_enums(&self) -> &HashMap<String, EnumInfo> {
        &self.enums
    }

    // ============================================================================
    // Function Operations
    // ============================================================================

    pub fn register_function(&mut self, name: &str, signature: FunctionSignature) {
        self.functions.insert(name.to_string(), signature);
    }

    pub fn get_function(&self, name: &str) -> Option<&FunctionSignature> {
        self.functions.get(name)
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn get_all_functions(&self) -> &HashMap<String, FunctionSignature> {
        &self.functions
    }

    // ============================================================================
    // Type Alias Operations
    // ============================================================================

    pub fn register_type_alias(&mut self, name: &str, target: AstType) {
        self.type_aliases.insert(name.to_string(), target);
    }

    pub fn get_type_alias(&self, name: &str) -> Option<&AstType> {
        self.type_aliases.get(name)
    }

    pub fn has_type_alias(&self, name: &str) -> bool {
        self.type_aliases.contains_key(name)
    }

    /// Get all type aliases
    pub fn get_all_aliases(&self) -> &HashMap<String, AstType> {
        &self.type_aliases
    }

    pub fn resolve_type(&self, ty: &AstType) -> AstType {
        // If it's a generic type with no args, check if it's an alias
        if let AstType::Generic { name, type_args } = ty {
            if type_args.is_empty() {
                if let Some(resolved) = self.get_type_alias(name) {
                    return resolved.clone();
                }
            }
        }
        ty.clone()
    }

    // ============================================================================
    // Method Operations
    // ============================================================================

    pub fn register_method(
        &mut self,
        type_name: &str,
        method_name: &str,
        signature: MethodSignature,
    ) {
        let key = name_utils::method_key(type_name, method_name);
        self.methods.insert(key, signature);
    }

    pub fn get_method(&self, type_name: &str, method_name: &str) -> Option<&MethodSignature> {
        let key = name_utils::method_key(type_name, method_name);
        self.methods.get(&key)
    }

    pub fn find_method_for_type(
        &self,
        type_name: &str,
        method_name: &str,
    ) -> Option<&MethodSignature> {
        // Try exact match first
        if let Some(method) = self.get_method(type_name, method_name) {
            return Some(method);
        }

        // Try stripping generic parameters (e.g., "Vec<T>" -> "Vec")
        let base_name = name_utils::strip_generics(type_name);
        self.get_method(base_name, method_name)
    }

    // ============================================================================
    // Variable Operations
    // ============================================================================

    pub fn register_variable(&mut self, function_name: &str, var_name: &str, ty: AstType) {
        let key = name_utils::scoped_var_key(function_name, var_name);
        self.variables.insert(key, ty);
    }

    pub fn get_variable(&self, function_name: &str, var_name: &str) -> Option<&AstType> {
        let key = name_utils::scoped_var_key(function_name, var_name);
        self.variables.get(&key)
    }

    pub fn get_all_variables(&self) -> &HashMap<String, AstType> {
        &self.variables
    }

    // ============================================================================
    // Generic Definition Operations
    // ============================================================================

    pub fn register_generic_function(&mut self, name: &str, func: Function) {
        self.generic_functions.insert(name.to_string(), func);
    }

    pub fn get_generic_function(&self, name: &str) -> Option<&Function> {
        self.generic_functions.get(name)
    }

    pub fn register_generic_struct(&mut self, name: &str, struct_def: StructDefinition) {
        self.generic_structs.insert(name.to_string(), struct_def);
    }

    pub fn get_generic_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.generic_structs.get(name)
    }

    pub fn register_generic_enum(&mut self, name: &str, enum_def: EnumDefinition) {
        self.generic_enums.insert(name.to_string(), enum_def);
    }

    pub fn get_generic_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.generic_enums.get(name)
    }

    // ============================================================================
    // Stdlib Integration
    // ============================================================================

    pub fn register_stdlib_function(
        &mut self,
        module: &str,
        func_name: &str,
        signature: FunctionSignature,
    ) {
        let key = name_utils::stdlib_func_key(module, func_name);
        self.stdlib_functions.insert(key, signature);
    }

    pub fn get_stdlib_function(&self, module: &str, func_name: &str) -> Option<&FunctionSignature> {
        let key = name_utils::stdlib_func_key(module, func_name);
        self.stdlib_functions.get(&key)
    }

    /// Load type information from stdlib modules
    pub fn load_stdlib_types(&mut self, modules: &HashMap<String, crate::ast::Program>) {
        for (module_name, program) in modules {
            for decl in &program.declarations {
                match decl {
                    Declaration::Function(func) => {
                        let params: Vec<(String, AstType)> = func
                            .args
                            .iter()
                            .map(|(name, ty)| (name.clone(), ty.clone()))
                            .collect();

                        let signature = FunctionSignature {
                            params,
                            return_type: func.return_type.clone(),
                            is_external: false,
                        };

                        self.register_stdlib_function(module_name, &func.name, signature);
                    }
                    Declaration::Struct(struct_def) => {
                        let fields: Vec<(String, AstType)> = struct_def
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), f.type_.clone()))
                            .collect();

                        self.register_struct(&struct_def.name, StructInfo::new(fields));
                    }
                    Declaration::Enum(enum_def) => {
                        let variants: Vec<(String, Option<AstType>)> = enum_def
                            .variants
                            .iter()
                            .map(|v| (v.name.clone(), v.payload.clone()))
                            .collect();

                        self.register_enum(&enum_def.name, EnumInfo { variants });
                    }
                    Declaration::TypeAlias(alias) => {
                        self.register_type_alias(&alias.name, alias.target_type.clone());
                    }
                    _ => {}
                }
            }
        }
    }

    // ============================================================================
    // Utility Methods
    // ============================================================================

    /// Check if a name refers to a known type (struct, enum, or alias)
    pub fn is_known_type(&self, name: &str) -> bool {
        self.has_struct(name) || self.has_enum(name) || self.has_type_alias(name)
    }

    /// Get the type kind for a name (for error messages)
    pub fn type_kind(&self, name: &str) -> Option<&'static str> {
        if self.has_struct(name) {
            Some("struct")
        } else if self.has_enum(name) {
            Some("enum")
        } else if self.has_type_alias(name) {
            Some("type alias")
        } else {
            None
        }
    }

    /// Clear all stored types (for testing)
    pub fn clear(&mut self) {
        self.structs.clear();
        self.enums.clear();
        self.functions.clear();
        self.type_aliases.clear();
        self.methods.clear();
        self.stdlib_functions.clear();
        self.variables.clear();
        self.generic_functions.clear();
        self.generic_structs.clear();
        self.generic_enums.clear();
    }
}

impl Default for TypeStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe reference to a TypeStore
pub type TypeStoreRef = Rc<RefCell<TypeStore>>;

/// Create a new TypeStore wrapped in Rc<RefCell>
pub fn new_type_store() -> TypeStoreRef {
    Rc::new(RefCell::new(TypeStore::new()))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_struct_registration() {
        let mut store = TypeStore::new();
        let info = StructInfo::new(vec![("x".to_string(), AstType::I32)]);

        store.register_struct("Point", info.clone());

        assert!(store.has_struct("Point"));
        assert_eq!(store.get_struct("Point").unwrap().fields.len(), 1);
    }

    #[test]
    fn test_type_alias_resolution() {
        let mut store = TypeStore::new();

        store.register_type_alias("Int", AstType::I32);

        let generic = AstType::Generic {
            name: "Int".to_string(),
            type_args: vec![],
        };

        let resolved = store.resolve_type(&generic);
        assert_eq!(resolved, AstType::I32);
    }

    #[test]
    fn test_method_lookup() {
        let mut store = TypeStore::new();
        let sig = MethodSignature {
            receiver_type: "Vec".to_string(),
            method_name: "len".to_string(),
            params: vec![],
            return_type: AstType::Usize,
            is_static: false,
        };

        store.register_method("Vec", "len", sig);

        assert!(store.get_method("Vec", "len").is_some());
        assert!(store.find_method_for_type("Vec<T>", "len").is_some());
    }
}
