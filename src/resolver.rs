use std::collections::HashMap;

use crate::ast::{Declaration, Program};
use crate::error::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Value,
    Type,
    Module,
    Behavior,
    Variant,
}

impl Namespace {
    fn diagnostic_name(self) -> &'static str {
        match self {
            Namespace::Value => "value",
            Namespace::Type => "type",
            Namespace::Module => "module",
            Namespace::Behavior => "behavior",
            Namespace::Variant => "variant",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub id: SymbolId,
    pub namespace: Namespace,
    pub name: String,
    pub is_public: bool,
    pub definition_span: Span,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    by_name: HashMap<(Namespace, String), SymbolId>,
}

impl SymbolTable {
    pub fn lookup(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        let id = self.by_name.get(&(namespace, name.to_string()))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    fn define(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let key = (namespace, name.to_string());
        if self.by_name.contains_key(&key) {
            return Err(Box::new(Diagnostic::error(
                "E0200",
                format!(
                    "duplicate {} symbol '{}'",
                    namespace.diagnostic_name(),
                    name
                ),
                definition_span,
            )));
        }

        let id = SymbolId(self.symbols.len() as u32);
        self.symbols.push(Symbol {
            id,
            namespace,
            name: name.to_string(),
            is_public,
            definition_span,
        });
        self.by_name.insert(key, id);
        Ok(id)
    }
}

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

        if diagnostics.is_empty() {
            Ok(table)
        } else {
            Err(diagnostics)
        }
    }

    fn define_declaration(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
    ) -> Result<(), Box<Diagnostic>> {
        match decl {
            Declaration::Function {
                name, public, span, ..
            } => {
                table.define(Namespace::Value, name, *public, *span)?;
            }
            Declaration::Method {
                type_name,
                method_name,
                public,
                span,
                ..
            } => {
                table.define(
                    Namespace::Value,
                    &format!("{type_name}.{method_name}"),
                    *public,
                    *span,
                )?;
            }
            Declaration::Struct {
                name, public, span, ..
            } => {
                table.define(Namespace::Type, name, *public, *span)?;
            }
            Declaration::Enum {
                name,
                variants,
                public,
                span,
                ..
            } => {
                table.define(Namespace::Type, name, *public, *span)?;
                for variant in variants {
                    table.define(Namespace::Variant, &variant.name, *public, variant.span)?;
                }
            }
            Declaration::Behavior { name, span, .. } => {
                table.define(Namespace::Behavior, name, false, *span)?;
            }
            Declaration::Import {
                module_path, span, ..
            } => {
                table.define(Namespace::Module, &module_path.join("."), false, *span)?;
            }
            Declaration::ImplBlock { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
        Ok(())
    }
}
