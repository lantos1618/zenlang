use std::collections::HashSet;

use crate::ast::{AstType, Param, TypeParam};
use crate::error::{Diagnostic, Span};

use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn validate_params(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        params: &[Param],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_params = HashSet::new();
        for param in params {
            if !seen_params.insert(param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0214",
                    format!("duplicate parameter `{}`", param.name),
                    param.span,
                ));
            }
            self.validate_type_ref(
                table,
                type_params,
                &param.ty,
                param.span,
                allow_self_type,
                diagnostics,
            );
        }
    }

    pub(super) fn validate_type_param_constraints(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let mut seen_type_params = HashSet::new();
        for type_param in type_params {
            if !seen_type_params.insert(type_param.name.as_str()) {
                diagnostics.push(Diagnostic::error(
                    "E0213",
                    format!("duplicate type parameter `{}`", type_param.name),
                    type_param.span,
                ));
            }
            if let Some(constraint) = &type_param.constraint {
                if !self.is_known_behavior_name(table, constraint) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{constraint}'"),
                        type_param.span,
                    ));
                }
                for type_arg in &type_param.constraint_type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        type_param.span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
        }
    }

    pub(super) fn validate_type_ref(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        ast_type: &AstType,
        span: Span,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match ast_type {
            AstType::Named(name) => {
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
            }
            AstType::Generic { name, type_args } => {
                if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            AstType::Array { elem, .. }
            | AstType::Slice(elem)
            | AstType::Ptr(elem)
            | AstType::MutPtr(elem)
            | AstType::RawPtr(elem) => {
                self.validate_type_ref(
                    table,
                    type_params,
                    elem,
                    span,
                    allow_self_type,
                    diagnostics,
                );
            }
            AstType::Function { params, ret } => {
                for param in params {
                    self.validate_type_ref(
                        table,
                        type_params,
                        param,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_type_ref(table, type_params, ret, span, allow_self_type, diagnostics);
            }
            AstType::SelfType => {
                if !allow_self_type {
                    diagnostics.push(Diagnostic::error(
                        "E0204",
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
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
            | AstType::Inferred => {}
        }
    }

    pub(super) fn is_known_type_name(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        name: &str,
    ) -> bool {
        table.lookup(Namespace::Type, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || type_params.iter().any(|type_param| type_param.name == name)
    }

    pub(super) fn is_known_behavior_name(&self, table: &SymbolTable, name: &str) -> bool {
        table.lookup(Namespace::Behavior, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
    }

    pub(super) fn behavior_type_params_for_ref(
        &self,
        table: &SymbolTable,
        behavior: &str,
    ) -> Vec<TypeParam> {
        table
            .lookup(Namespace::Behavior, behavior)
            .and_then(|symbol| symbol.type_parameter_names.as_ref())
            .map(|names| {
                names
                    .iter()
                    .map(|name| TypeParam {
                        name: name.clone(),
                        constraint: None,
                        constraint_type_args: Vec::new(),
                        span: Span::dummy(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}
