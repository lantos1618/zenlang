use crate::ast::AstType;
use crate::stdlib_types::StdlibTypeRegistry;
use std::collections::HashMap;

pub mod environment;
pub mod instantiation;
pub mod monomorphization;

pub use environment::TypeEnvironment;
pub use instantiation::TypeInstantiator;
pub use monomorphization::Monomorphizer;

// ============================================================================
// Shared utility functions for type instantiation/monomorphization
// ============================================================================

/// Generate a unique name for an instantiated generic type
/// e.g., "Vec" + [i32] -> "Vec_i32"
pub(crate) fn generate_instantiated_name(base_name: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        return base_name.to_string();
    }
    let type_names: Vec<String> = type_args.iter().map(type_to_string).collect();
    format!("{}_{}", base_name, type_names.join("_"))
}

/// Convert an AstType to a string suitable for name mangling
pub(crate) fn type_to_string(ast_type: &AstType) -> String {
    // Use centralized primitive name lookup first
    if let Some(name) = ast_type.primitive_name() {
        return name.to_string();
    }

    match ast_type {
        AstType::Struct { name, .. } if StdlibTypeRegistry::is_string_type(name) => {
            "string".to_string()
        }
        t if t.is_ptr_type() => {
            if let Some(inner) = t.ptr_inner() {
                format!("ptr_{}", type_to_string(inner))
            } else {
                "ptr".to_string()
            }
        }
        AstType::Slice(inner) => format!("slice_{}", type_to_string(inner)),
        AstType::Generic { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args: Vec<String> = type_args.iter().map(type_to_string).collect();
                format!("{}_{}", name, args.join("_"))
            }
        }
        AstType::Struct { name, .. } => name.clone(),
        _ => "unknown".to_string(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenericInstance {
    pub base_name: String,
    pub type_args: Vec<AstType>,
    pub specialized_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeSubstitution {
    pub mappings: HashMap<String, AstType>,
}

impl Default for TypeSubstitution {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeSubstitution {
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
        }
    }

    pub fn add(&mut self, param: String, concrete: AstType) {
        self.mappings.insert(param, concrete);
    }

    #[allow(dead_code)] // get() is public API, may be used externally
    pub fn get(&self, param: &str) -> Option<&AstType> {
        self.mappings.get(param)
    }

    pub fn apply(&self, ast_type: &AstType) -> AstType {
        match ast_type {
            AstType::Generic { name, type_args } => {
                if type_args.is_empty() {
                    if let Some(concrete) = self.mappings.get(name) {
                        return concrete.clone();
                    }
                }

                AstType::Generic {
                    name: name.clone(),
                    type_args: type_args.iter().map(|t| self.apply(t)).collect(),
                }
            }
            t if t.is_ptr_type() => {
                if let Some(inner) = t.ptr_inner() {
                    AstType::ptr(self.apply(inner))
                } else {
                    ast_type.clone()
                }
            }
            AstType::Slice(inner) => AstType::Slice(Box::new(self.apply(inner))),
            // Option and Result are now Generic types - handled in Generic match above
            AstType::Ref(inner) => AstType::Ref(Box::new(self.apply(inner))),
            AstType::Function { args, return_type } => AstType::Function {
                args: args.iter().map(|t| self.apply(t)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            _ => ast_type.clone(),
        }
    }
}
