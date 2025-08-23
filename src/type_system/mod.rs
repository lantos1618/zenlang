use crate::ast::AstType;
use std::collections::HashMap;

pub mod instantiation;
pub mod environment;
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
            AstType::Pointer(inner) => {
                AstType::Pointer(Box::new(self.apply(inner)))
            }
            AstType::Array(inner) => {
                AstType::Array(Box::new(self.apply(inner)))
            }
            AstType::Option(inner) => {
                AstType::Option(Box::new(self.apply(inner)))
            }
            AstType::Result { ok_type, err_type } => {
                AstType::Result {
                    ok_type: Box::new(self.apply(ok_type)),
                    err_type: Box::new(self.apply(err_type)),
                }
            }
            AstType::Ref(inner) => {
                AstType::Ref(Box::new(self.apply(inner)))
            }
            AstType::Function { args, return_type } => {
                AstType::Function {
                    args: args.iter().map(|t| self.apply(t)).collect(),
                    return_type: Box::new(self.apply(return_type)),
                }
            }
            _ => ast_type.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeConstraint {
    pub param: String,
    pub bounds: Vec<String>,
}

pub fn is_generic_type(ast_type: &AstType) -> bool {
    match ast_type {
        AstType::Generic { .. } => true,
        AstType::Pointer(inner) | AstType::Array(inner) | 
        AstType::Option(inner) | AstType::Ref(inner) => is_generic_type(inner),
        AstType::Result { ok_type, err_type } => is_generic_type(ok_type) || is_generic_type(err_type),
        AstType::Function { args, return_type } => {
            args.iter().any(is_generic_type) || is_generic_type(return_type)
        }
        _ => false,
    }
}

pub fn extract_type_parameters(ast_type: &AstType) -> Vec<String> {
    let mut params = Vec::new();
    extract_type_params_recursive(ast_type, &mut params);
    params
}

fn extract_type_params_recursive(ast_type: &AstType, params: &mut Vec<String>) {
    match ast_type {
        AstType::Generic { name, type_args } => {
            if type_args.is_empty() && !params.contains(name) {
                params.push(name.clone());
            }
            for arg in type_args {
                extract_type_params_recursive(arg, params);
            }
        }
        AstType::Pointer(inner) | AstType::Array(inner) | 
        AstType::Option(inner) | AstType::Ref(inner) => {
            extract_type_params_recursive(inner, params);
        }
        AstType::Result { ok_type, err_type } => {
            extract_type_params_recursive(ok_type, params);
            extract_type_params_recursive(err_type, params);
        }
        AstType::Function { args, return_type } => {
            for arg in args {
                extract_type_params_recursive(arg, params);
            }
            extract_type_params_recursive(return_type, params);
        }
        _ => {}
    }
}