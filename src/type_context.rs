//! TypeContext - Shared type information that flows through the compilation pipeline
//!
//! This module provides the infrastructure for passing resolved type information
//! from the typechecker to later phases (monomorphization, codegen), eliminating
//! the need for codegen to re-infer types.
//!
//! Pipeline flow:
//!   Parser -> TypeChecker -> TypeContext -> Monomorphizer -> Codegen
//!                              (populated)                    (consumed)
//!
//! ## Intrinsic Layouts
//!
//! The compiler needs to know memory layouts for certain stdlib types to generate
//! correct code. Instead of hardcoding type names like "Array" or "HashMap",
//! stdlib types declare their layout via `@compiler_intrinsic(layout = "...")`.
//! The typechecker populates `intrinsic_layouts` which codegen queries.
//!
//! This enables:
//! - Self-hosting: Zen compiler written in Zen uses same pattern
//! - Extensibility: New collection types don't require compiler changes
//! - Single source of truth: Type info comes from .zen files

use crate::ast::AstType;
use crate::error::Span;
use std::collections::HashMap;

// ============================================================================
// Collection Type Categories (TEMPORARY - should come from stdlib annotations)
// ============================================================================
// These lists exist as a stepping stone toward fully stdlib-driven type discovery.
// Eventually, stdlib types will use @collection(kind="...") annotations and
// the typechecker will populate these from parsing .zen files.

/// Key-value collection types: return type_args[1] for get/remove
pub const KEY_VALUE_COLLECTIONS: &[&str] = &["HashMap", "BTreeMap"];

/// Single-element collection types: return type_args[0] for get/pop
pub const SINGLE_ELEMENT_COLLECTIONS: &[&str] = &[
    "Vec", "Array", "HashSet", "Set", "Stack", "Queue", "LinkedList", "Range",
];

/// Check if a type name is a key-value collection
pub fn is_key_value_collection(name: &str) -> bool {
    KEY_VALUE_COLLECTIONS.contains(&name)
}

/// Check if a type name is a single-element collection
pub fn is_single_element_collection(name: &str) -> bool {
    SINGLE_ELEMENT_COLLECTIONS.contains(&name)
}

// ============================================================================
// Intrinsic Layouts - Compiler-generated types only
// ============================================================================

/// Truly intrinsic layouts that require compiler knowledge.
///
/// Only Closure is included here because closures are compiler-generated types
/// with a specific ABI ({ fn_ptr, captures_ptr }).
///
/// All other types (Array, HashMap, HashSet, String, Vec, Range) are regular
/// structs defined in stdlib - their layouts come from TypeContext.structs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IntrinsicLayout {
    /// Closure: { fn_ptr, captures_ptr }
    /// Closures are compiler-generated types with special ABI.
    Closure,
}

impl IntrinsicLayout {
    /// Parse layout name from @compiler_intrinsic attribute
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "closure" => Some(Self::Closure),
            _ => None,
        }
    }
}

/// Information about a method on a type (for method return type lookups)
#[derive(Clone, Debug)]
pub struct MethodTypeInfo {
    pub receiver_type: String,
    pub method_name: String,
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_static: bool,
}

/// Information about a function's signature
#[derive(Clone, Debug)]
pub struct FunctionTypeInfo {
    pub params: Vec<(String, AstType)>,
    pub return_type: AstType,
    pub is_external: bool,
}

/// Information about a struct type
#[derive(Clone, Debug)]
pub struct StructTypeInfo {
    pub fields: Vec<(String, AstType)>,
}

/// Information about an enum type
#[derive(Clone, Debug)]
pub struct EnumTypeInfo {
    pub variants: Vec<(String, Option<AstType>)>,
}

/// Information about a variable
#[derive(Clone, Debug)]
pub struct VariableTypeInfo {
    pub type_: AstType,
    pub is_mutable: bool,
}

/// TypeContext holds all resolved type information from semantic analysis.
///
/// This is the "Typed AST" concept - instead of annotating each AST node,
/// we maintain a side table of type information indexed by name/location.
#[derive(Clone, Debug, Default)]
pub struct TypeContext {
    /// Function signatures: name -> (params, return_type)
    pub functions: HashMap<String, FunctionTypeInfo>,

    /// Struct definitions: name -> fields
    pub structs: HashMap<String, StructTypeInfo>,

    /// Enum definitions: name -> variants
    pub enums: HashMap<String, EnumTypeInfo>,

    /// Global/module-level variables
    pub globals: HashMap<String, VariableTypeInfo>,

    /// Method return types: "Type.method" -> return_type
    pub method_returns: HashMap<String, AstType>,

    /// Method signatures: "Type::method" -> full signature
    pub methods: HashMap<String, MethodTypeInfo>,

    /// Expression types: (start, end) -> type
    /// This maps source spans to the type of the expression at that location.
    /// Populated by the typechecker, consumed by codegen.
    pub expr_types: HashMap<(usize, usize), AstType>,

    /// Variable types in scope: name -> type
    /// Tracks local variable types within functions.
    pub var_types: HashMap<String, AstType>,

    /// Intrinsic layouts: type_name -> layout
    /// Maps stdlib type names to their memory layout patterns.
    /// Populated when typechecker sees @compiler_intrinsic attribute.
    /// Used by codegen instead of hardcoded type name checks.
    pub intrinsic_layouts: HashMap<String, IntrinsicLayout>,
}

impl TypeContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a function signature
    pub fn register_function(&mut self, name: String, params: Vec<(String, AstType)>, return_type: AstType, is_external: bool) {
        self.functions.insert(name.clone(), FunctionTypeInfo {
            params,
            return_type: return_type.clone(),
            is_external,
        });
        // Also register in method_returns for easy lookup
        self.method_returns.insert(name, return_type);
    }

    /// Register a struct type
    pub fn register_struct(&mut self, name: String, fields: Vec<(String, AstType)>) {
        self.structs.insert(name, StructTypeInfo { fields });
    }

    /// Register an enum type
    pub fn register_enum(&mut self, name: String, variants: Vec<(String, Option<AstType>)>) {
        self.enums.insert(name, EnumTypeInfo { variants });
    }

    /// Get the return type of a function
    pub fn get_function_return_type(&self, name: &str) -> Option<&AstType> {
        self.functions.get(name).map(|f| &f.return_type)
    }

    /// Get the type of a struct field
    pub fn get_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<&AstType> {
        self.structs.get(struct_name).and_then(|s| {
            s.fields.iter().find(|(n, _)| n == field_name).map(|(_, t)| t)
        })
    }

    /// Get all fields of a struct
    pub fn get_struct_fields(&self, struct_name: &str) -> Option<&Vec<(String, AstType)>> {
        self.structs.get(struct_name).map(|s| &s.fields)
    }

    /// Check if a struct exists
    pub fn has_struct(&self, name: &str) -> bool {
        self.structs.contains_key(name)
    }

    /// Get enum variants
    pub fn get_enum_variants(&self, enum_name: &str) -> Option<&Vec<(String, Option<AstType>)>> {
        self.enums.get(enum_name).map(|e| &e.variants)
    }

    /// Get the payload type of an enum variant
    pub fn get_enum_variant_payload(&self, enum_name: &str, variant_name: &str) -> Option<Option<&AstType>> {
        self.enums.get(enum_name).and_then(|e| {
            e.variants.iter()
                .find(|(n, _)| n == variant_name)
                .map(|(_, payload)| payload.as_ref())
        })
    }

    /// Register the type of an expression at a span
    pub fn register_expr_type(&mut self, span: &Span, ty: AstType) {
        self.expr_types.insert((span.start, span.end), ty);
    }

    /// Register the type of an expression by start/end positions
    pub fn register_expr_type_pos(&mut self, start: usize, end: usize, ty: AstType) {
        self.expr_types.insert((start, end), ty);
    }

    /// Get the type of an expression at a span
    pub fn get_expr_type(&self, span: &Span) -> Option<&AstType> {
        self.expr_types.get(&(span.start, span.end))
    }

    /// Get the type of an expression by start/end positions
    pub fn get_expr_type_pos(&self, start: usize, end: usize) -> Option<&AstType> {
        self.expr_types.get(&(start, end))
    }

    /// Register a variable's type
    pub fn register_var_type(&mut self, name: String, ty: AstType) {
        self.var_types.insert(name, ty);
    }

    /// Get a variable's type
    pub fn get_var_type(&self, name: &str) -> Option<&AstType> {
        self.var_types.get(name)
    }

    /// Clear variable types (for entering new scope)
    pub fn clear_var_types(&mut self) {
        self.var_types.clear();
    }

    // ========================================================================
    // Intrinsic Layout Methods
    // ========================================================================

    /// Register an intrinsic layout for a type name.
    /// Called when typechecker sees @compiler_intrinsic(layout = "...") attribute.
    pub fn register_intrinsic_layout(&mut self, type_name: &str, layout: IntrinsicLayout) {
        self.intrinsic_layouts.insert(type_name.to_string(), layout);
    }

    /// Get the intrinsic layout for a type name, if any.
    /// Use this instead of `if name == "Array"` checks in codegen.
    pub fn get_intrinsic_layout(&self, type_name: &str) -> Option<IntrinsicLayout> {
        self.intrinsic_layouts.get(type_name).copied()
    }

    /// Check if a type has a specific intrinsic layout.
    /// Example: `type_ctx.has_layout("Array", IntrinsicLayout::Array)`
    pub fn has_layout(&self, type_name: &str, layout: IntrinsicLayout) -> bool {
        self.get_intrinsic_layout(type_name) == Some(layout)
    }

    // ========================================================================
    // Method Registry Methods
    // ========================================================================

    /// Register a method signature for a type.
    /// Key format: "TypeName::method_name"
    pub fn register_method(&mut self, receiver: &str, method: &str, params: Vec<(String, AstType)>, return_type: AstType, is_static: bool) {
        let key = format!("{}::{}", receiver, method);
        self.methods.insert(key.clone(), MethodTypeInfo {
            receiver_type: receiver.to_string(),
            method_name: method.to_string(),
            params,
            return_type: return_type.clone(),
            is_static,
        });
        // Also register in method_returns for quick lookups
        let dot_key = format!("{}.{}", receiver, method);
        self.method_returns.insert(dot_key, return_type);
    }

    /// Get a method's full signature
    pub fn get_method(&self, receiver: &str, method: &str) -> Option<&MethodTypeInfo> {
        let key = format!("{}::{}", receiver, method);
        self.methods.get(&key)
    }

    /// Get a method's return type
    /// First checks TypeContext, then falls back to stdlib_types
    pub fn get_method_return_type(&self, receiver: &str, method: &str) -> Option<AstType> {
        // Check TypeContext first (populated by typechecker)
        if let Some(method_info) = self.get_method(receiver, method) {
            return Some(method_info.return_type.clone());
        }
        // Fall back to stdlib_types (statically parsed stdlib)
        crate::stdlib_types::stdlib_types().get_method_return_type(receiver, method).cloned()
    }

    /// Get a function's return type with stdlib_types fallback
    pub fn get_function_return_type_with_fallback(&self, module: &str, func: &str) -> Option<AstType> {
        // Check TypeContext first
        let key = format!("{}::{}", module, func);
        if let Some(func_info) = self.functions.get(&key) {
            return Some(func_info.return_type.clone());
        }
        // Also try just the function name
        if let Some(func_info) = self.functions.get(func) {
            return Some(func_info.return_type.clone());
        }
        // Fall back to stdlib_types
        crate::stdlib_types::stdlib_types().get_function_return_type(module, func).cloned()
    }

    /// Get struct definition with stdlib_types fallback
    pub fn get_struct_definition_with_fallback(&self, name: &str) -> Option<Vec<(String, AstType)>> {
        // Check TypeContext first
        if let Some(struct_info) = self.structs.get(name) {
            return Some(struct_info.fields.clone());
        }
        // Fall back to stdlib_types
        crate::stdlib_types::stdlib_types().get_struct_definition(name).map(|def| {
            def.fields.iter().map(|f| (f.name.clone(), f.type_.clone())).collect()
        })
    }

    /// Check if a method exists for a type
    pub fn has_method(&self, receiver: &str, method: &str) -> bool {
        let key = format!("{}::{}", receiver, method);
        self.methods.contains_key(&key)
    }
}
