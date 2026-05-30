use std::collections::HashSet;

use crate::ast::{behavior_ref_display, Declaration, Program};
use crate::error::Diagnostic;

mod declaration_definition;
mod declaration_validation;
mod expression_validation;
mod expression_validation_constructs;
mod local_validation;
mod metadata_helpers;
mod symbol_table;
mod type_validation;

pub use symbol_table::{
    BehaviorMethodTypeMetadata, BehaviorRefMetadata, Namespace, Symbol, SymbolId, SymbolTable,
    TypeParameterBoundRefMetadata,
};

pub struct Resolver;

impl Resolver {
    pub fn resolve_program(&self, program: &Program) -> Result<SymbolTable, Vec<Diagnostic>> {
        let mut table = SymbolTable::default();
        let mut diagnostics = Vec::new();
        let mut seen_behavior_impls = HashSet::new();

        for decl in &program.declarations {
            if let Declaration::ImplBlock {
                type_name,
                behavior: Some(behavior),
                behavior_type_args,
                ..
            } = decl
            {
                let key = format!(
                    "{}::{}",
                    type_name,
                    behavior_ref_display(behavior, behavior_type_args)
                );
                if !seen_behavior_impls.insert(key) {
                    continue;
                }
            }
            if let Err(diagnostic) = self.define_declaration(&mut table, decl) {
                diagnostics.push(diagnostic);
            }
        }

        // Apply `@export({ ... })` manifests: mark the listed (otherwise
        // private) symbols public. Runs after all symbols are defined so an
        // export can name a declaration anywhere in the file.
        for decl in &program.declarations {
            if let Declaration::Export { names, span } = decl {
                for name in names {
                    if !table.mark_public(name) {
                        diagnostics.push(Diagnostic::error_code(
                            crate::error::CompilerDiagnosticCode::E0203,
                            format!("exported name `{name}` is not defined in this module"),
                            *span,
                        ));
                    }
                }
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
