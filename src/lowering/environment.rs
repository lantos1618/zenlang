use super::GenericInstance;
use crate::ast::{AstType, EnumDefinition, Function, StructDefinition, TypeParameter};
use crate::error::CompileError;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    generic_functions: HashMap<String, Function>,
    generic_structs: HashMap<String, StructDefinition>,
    generic_enums: HashMap<String, EnumDefinition>,
    #[allow(dead_code)] // instantiated_types reserved for future use
    instantiated_types: HashMap<String, Vec<GenericInstance>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            generic_functions: HashMap::new(),
            generic_structs: HashMap::new(),
            generic_enums: HashMap::new(),
            instantiated_types: HashMap::new(),
        }
    }

    pub fn register_generic_function(&mut self, func: Function) {
        if !func.type_params.is_empty() {
            self.generic_functions.insert(func.name.clone(), func);
        }
    }

    pub fn register_generic_struct(&mut self, struct_def: StructDefinition) {
        if !struct_def.type_params.is_empty() {
            self.generic_structs
                .insert(struct_def.name.clone(), struct_def);
        }
    }

    pub fn register_generic_enum(&mut self, enum_def: EnumDefinition) {
        if !enum_def.type_params.is_empty() {
            self.generic_enums.insert(enum_def.name.clone(), enum_def);
        }
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

    pub fn validate_type_args(
        &self,
        expected: &[TypeParameter],
        provided: &[AstType],
    ) -> Result<(), CompileError> {
        if expected.len() != provided.len() {
            return Err(CompileError::TypeError(
                format!(
                    "Type argument count mismatch: expected {}, got {}",
                    expected.len(),
                    provided.len()
                ),
                None,
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
