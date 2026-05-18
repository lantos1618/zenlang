use std::collections::HashSet;

use crate::ast::{Declaration, Program};
use crate::error::Diagnostic;

use self::metadata_helpers::behavior_ref_display;

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
        let mut seen_behavior_impls = HashSet::new();

        for decl in &program.declarations {
            let skip_duplicate_behavior_impl_methods = match decl {
                Declaration::ImplBlock {
                    type_name,
                    behavior: Some(behavior),
                    behavior_type_args,
                    ..
                } => {
                    let key = format!(
                        "{}::{}",
                        type_name,
                        behavior_ref_display(behavior, behavior_type_args)
                    );
                    !seen_behavior_impls.insert(key)
                }
                _ => false,
            };
            if let Err(diagnostic) =
                self.define_declaration(&mut table, decl, skip_duplicate_behavior_impl_methods)
            {
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
