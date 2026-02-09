use crate::ast::{
    AstType, Declaration, EnumDefinition, Function, Program, StructDefinition, TypeParameter,
};
use crate::error::CompileError;
use std::collections::HashMap;

/// TypeEnvironment queries generic types from a Program instead of duplicating storage.
/// This eliminates duplication - the Program is the single source of truth.
#[derive(Debug)]
pub struct TypeEnvironment<'prog> {
    /// Reference to the Program containing generic type definitions
    program: &'prog Program,
    /// Index mapping: type name -> index in program.declarations for fast lookup
    generic_indices: HashMap<String, usize>,
}

impl<'prog> TypeEnvironment<'prog> {
    /// Create a new TypeEnvironment that queries the given Program for generic types
    pub fn new(program: &'prog Program) -> Self {
        let mut generic_indices = HashMap::new();

        // Build index of generic types for fast lookup
        for (idx, decl) in program.declarations.iter().enumerate() {
            let name = match decl {
                Declaration::Function(func) if !func.type_params.is_empty() => Some(&func.name),
                Declaration::Struct(struct_def) if !struct_def.type_params.is_empty() => {
                    Some(&struct_def.name)
                }
                Declaration::Enum(enum_def) if !enum_def.type_params.is_empty() => {
                    Some(&enum_def.name)
                }
                _ => None,
            };
            if let Some(name) = name {
                generic_indices.insert(name.clone(), idx);
            }
        }

        Self {
            program,
            generic_indices,
        }
    }

    /// Get a generic function by name (queries Program, doesn't store duplicates)
    pub fn get_generic_function(&self, name: &str) -> Option<&Function> {
        self.generic_indices
            .get(name)
            .and_then(|&idx| match &self.program.declarations[idx] {
                Declaration::Function(func) if !func.type_params.is_empty() => Some(func),
                _ => None,
            })
    }

    /// Get a generic struct by name (queries Program, doesn't store duplicates)
    pub fn get_generic_struct(&self, name: &str) -> Option<&StructDefinition> {
        self.generic_indices
            .get(name)
            .and_then(|&idx| match &self.program.declarations[idx] {
                Declaration::Struct(struct_def) if !struct_def.type_params.is_empty() => {
                    Some(struct_def)
                }
                _ => None,
            })
    }

    /// Get a generic enum by name (queries Program, doesn't store duplicates)
    pub fn get_generic_enum(&self, name: &str) -> Option<&EnumDefinition> {
        self.generic_indices
            .get(name)
            .and_then(|&idx| match &self.program.declarations[idx] {
                Declaration::Enum(enum_def) if !enum_def.type_params.is_empty() => Some(enum_def),
                _ => None,
            })
    }

    #[allow(dead_code)] // validate_type_args is public API for future use
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
                // Trait bounds checking deferred until trait system is implemented
            }
        }

        Ok(())
    }
}
