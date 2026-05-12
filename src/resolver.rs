use std::collections::HashMap;

use crate::ast::{AstType, Declaration, Param, Program, TypeParam};
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

        let imported_names = self.collect_imported_names(program);
        for decl in &program.declarations {
            self.validate_declaration_types(&table, &imported_names, decl, &mut diagnostics);
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

    fn validate_declaration_types(
        &self,
        table: &SymbolTable,
        imported_names: &[String],
        decl: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match decl {
            Declaration::Function {
                type_params,
                params,
                return_type,
                span,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                self.validate_params(table, imported_names, type_params, params, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        imported_names,
                        type_params,
                        return_type,
                        *span,
                        diagnostics,
                    );
                }
            }
            Declaration::Method {
                type_params,
                params,
                return_type,
                span,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                self.validate_params(table, imported_names, type_params, params, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        imported_names,
                        type_params,
                        return_type,
                        *span,
                        diagnostics,
                    );
                }
            }
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for field in fields {
                    self.validate_type_ref(
                        table,
                        imported_names,
                        type_params,
                        &field.ty,
                        field.span,
                        diagnostics,
                    );
                }
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type_ref(
                            table,
                            imported_names,
                            type_params,
                            payload,
                            variant.span,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, diagnostics);
                for method in methods {
                    self.validate_params(
                        table,
                        imported_names,
                        type_params,
                        &method.params,
                        diagnostics,
                    );
                    if let Some(return_type) = &method.return_type {
                        self.validate_type_ref(
                            table,
                            imported_names,
                            type_params,
                            return_type,
                            method.span,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::ImplBlock { methods, .. } => {
                for method in methods {
                    self.validate_declaration_types(table, imported_names, method, diagnostics);
                }
            }
            Declaration::Import { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
    }

    fn validate_params(
        &self,
        table: &SymbolTable,
        imported_names: &[String],
        type_params: &[TypeParam],
        params: &[Param],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for param in params {
            self.validate_type_ref(
                table,
                imported_names,
                type_params,
                &param.ty,
                param.span,
                diagnostics,
            );
        }
    }

    fn validate_type_param_constraints(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for type_param in type_params {
            if let Some(constraint) = &type_param.constraint {
                if table.lookup(Namespace::Behavior, constraint).is_none() {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{constraint}'"),
                        type_param.span,
                    ));
                }
            }
        }
    }

    fn validate_type_ref(
        &self,
        table: &SymbolTable,
        imported_names: &[String],
        type_params: &[TypeParam],
        ast_type: &AstType,
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if !self.is_known_type_name(table, imported_names, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
            }
            AstType::Generic { name, type_args } => {
                if !self.is_known_type_name(table, imported_names, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        imported_names,
                        type_params,
                        type_arg,
                        span,
                        diagnostics,
                    );
                }
            }
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem) => {
                self.validate_type_ref(table, imported_names, type_params, elem, span, diagnostics);
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_type_ref(
                        table,
                        imported_names,
                        type_params,
                        param,
                        span,
                        diagnostics,
                    );
                }
                self.validate_type_ref(table, imported_names, type_params, ret, span, diagnostics);
            }
            AstType::I8
            | AstType::I16
            | AstType::I32
            | AstType::I64
            | AstType::U8
            | AstType::U16
            | AstType::U32
            | AstType::U64
            | AstType::Usize
            | AstType::F32
            | AstType::F64
            | AstType::Bool
            | AstType::Void
            | AstType::Str
            | AstType::String
            | AstType::SelfType
            | AstType::Inferred => {}
        }
    }

    fn is_known_type_name(
        &self,
        table: &SymbolTable,
        imported_names: &[String],
        type_params: &[TypeParam],
        name: &str,
    ) -> bool {
        table.lookup(Namespace::Type, name).is_some()
            || imported_names.iter().any(|imported| imported == name)
            || type_params.iter().any(|type_param| type_param.name == name)
    }

    fn collect_imported_names(&self, program: &Program) -> Vec<String> {
        program
            .declarations
            .iter()
            .filter_map(|decl| match decl {
                Declaration::Import { names, .. } => Some(names),
                _ => None,
            })
            .flatten()
            .cloned()
            .collect()
    }
}
