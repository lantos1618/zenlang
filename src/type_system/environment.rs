use crate::ast::{AstType, Function, StructDefinition, EnumDefinition, TypeParameter};
use std::collections::HashMap;
use super::{TypeSubstitution, GenericInstance};

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    generic_functions: HashMap<String, Function>,
    generic_structs: HashMap<String, StructDefinition>,
    generic_enums: HashMap<String, EnumDefinition>,
    instantiated_types: HashMap<String, Vec<GenericInstance>>,
    type_aliases: HashMap<String, AstType>,
    current_scope: Vec<TypeScope>,
}

#[derive(Debug, Clone)]
struct TypeScope {
    type_params: Vec<TypeParameter>,
    substitutions: TypeSubstitution,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            generic_enums: HashMap::new(),
            instantiated_types: HashMap::new(),
            type_aliases: HashMap::new(),
            current_scope: vec![TypeScope {
                type_params: Vec::new(),
                substitutions: TypeSubstitution::new(),
            }],
        }
    }

    pub fn register_generic_function(&mut self, func: Function) {
        if !func.type_params.is_empty() {
            self.generic_functions.insert(func.name.clone(), func);
        }
    }

    pub fn register_generic_struct(&mut self, struct_def: StructDefinition) {
        if !struct_def.type_params.is_empty() {
            self.generic_structs.insert(struct_def.name.clone(), struct_def);
        }
    }

    pub fn register_generic_enum(&mut self, enum_def: EnumDefinition) {
        if !enum_def.type_params.is_empty() {
            self.generic_enums.insert(enum_def.name.clone(), enum_def);
        }
    }

    pub fn push_scope(&mut self, type_params: Vec<TypeParameter>) {
        self.current_scope.push(TypeScope {
            type_params,
            substitutions: TypeSubstitution::new(),
        });
    }

    pub fn pop_scope(&mut self) {
        if self.current_scope.len() > 1 {
            self.current_scope.pop();
        }
    }

    pub fn add_substitution(&mut self, param: String, concrete: AstType) {
        if let Some(scope) = self.current_scope.last_mut() {
            scope.substitutions.add(param, concrete);
        }
    }

    pub fn resolve_type(&self, ast_type: &AstType) -> AstType {
        for scope in self.current_scope.iter().rev() {
            let resolved = scope.substitutions.apply(ast_type);
            if resolved != *ast_type {
                return resolved;
            }
        }
        ast_type.clone()
    }

    pub fn get_generic_function(&self, name: &str) -> Option<&Function> {
        self.generic_functions.get(name)
    }

    pub fn get_generic_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.generic_structs.get(name)
    }

    pub fn get_generic_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.generic_enums.get(name)
    }

    pub fn record_instantiation(&mut self, base_name: String, type_args: Vec<AstType>, specialized: AstType) {
        let instance = GenericInstance {
            base_name: base_name.clone(),
            type_args,
            specialized_type: specialized,
        };
        
        self.instantiated_types
            .entry(base_name)
            .or_insert_with(Vec::new)
            .push(instance);
    }

    pub fn get_instantiation(&self, base_name: &str, type_args: &[AstType]) -> Option<&AstType> {
        self.instantiated_types
            .get(base_name)?
            .iter()
            .find(|inst| inst.type_args == type_args)
            .map(|inst| &inst.specialized_type)
    }

    pub fn is_type_parameter(&self, name: &str) -> bool {
        self.current_scope.iter().any(|scope| {
            scope.type_params.iter().any(|param| param.name == name)
        })
    }

    pub fn validate_type_args(&self, expected: &[TypeParameter], provided: &[AstType]) -> Result<(), String> {
        if expected.len() != provided.len() {
            return Err(format!(
                "Type argument count mismatch: expected {}, got {}",
                expected.len(),
                provided.len()
            ));
        }
        
        for (param, _arg) in expected.iter().zip(provided.iter()) {
            if !param.constraints.is_empty() {
                // TODO: Check trait bounds when trait system is implemented
            }
        }
        
        Ok(())
    }
}