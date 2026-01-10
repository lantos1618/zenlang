use crate::ast::AstType;
use std::collections::HashMap;

pub mod environment;
pub mod instantiation;
pub mod monomorphization;

pub use environment::TypeEnvironment;
pub use instantiation::TypeInstantiator;
pub use monomorphization::Monomorphizer;

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
            AstType::Array(inner) => AstType::Array(Box::new(self.apply(inner))),
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

