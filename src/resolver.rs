use crate::ast::Program;
use crate::error::Diagnostic;

#[cfg(test)]
mod symbol_table_test_support;

mod declaration_definition;
mod declaration_validation;
mod expression_validation;
mod expression_validation_constructs;
mod local_validation;
mod metadata_helpers;
mod symbol_table;
mod type_validation;

pub use symbol_table::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, MethodSignatureMetadata, Namespace, Symbol,
    SymbolId, SymbolTable, TypeParameterBoundMetadata, TypeParameterBoundRefMetadata,
};

#[derive(Debug, Default)]
pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    pub fn resolve_program(&self, program: &Program) -> Result<SymbolTable, Vec<Diagnostic>> {
        let mut table = SymbolTable::default();
        let mut diagnostics = Vec::new();

        for decl in &program.declarations {
            if let Err(diagnostic) = self.define_declaration(&mut table, decl) {
                diagnostics.push(*diagnostic);
            }
        }

        for decl in &program.declarations {
            self.validate_declaration_types(&mut table, decl, &mut diagnostics);
        }

        if diagnostics.is_empty() {
            Ok(table)
        } else {
            Err(diagnostics)
        }
    }
}
