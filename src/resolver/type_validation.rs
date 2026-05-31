use std::collections::HashSet;

use crate::ast::{AstType, BuiltinTypeName, Param, TypeParam};
use crate::error::{CompilerDiagnosticCode::*, Diagnostic, Span};

use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(in crate::resolver) fn push_unknown_type_symbol(
        &self,
        diagnostics: &mut Vec<Diagnostic>,
        name: &str,
        span: Span,
    ) {
        diagnostics.push(Diagnostic::error_code(
            E0201,
            format!("unknown type symbol '{name}'"),
            span,
        ));
    }

    pub(in crate::resolver) fn push_unknown_behavior_symbol(
        &self,
        diagnostics: &mut Vec<Diagnostic>,
        name: &str,
        span: Span,
    ) {
        diagnostics.push(Diagnostic::error_code(
            E0202,
            format!("unknown behavior symbol '{name}'"),
            span,
        ));
    }

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
                diagnostics.push(Diagnostic::error_code(
                    E0214,
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
                diagnostics.push(Diagnostic::error_code(
                    E0213,
                    format!("duplicate type parameter `{}`", type_param.name),
                    type_param.span,
                ));
            }
            if let Some(constraint) = &type_param.constraint {
                if !self.is_known_behavior_name(table, constraint) {
                    self.push_unknown_behavior_symbol(diagnostics, constraint, type_param.span);
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
            if let Some(default) = &type_param.default {
                self.validate_type_ref(
                    table,
                    type_params,
                    default,
                    type_param.span,
                    allow_self_type,
                    diagnostics,
                );
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
        if !matches!(ast_type, AstType::SelfType)
            && (BuiltinTypeName::from_ast_type(ast_type).is_some()
                || matches!(ast_type, AstType::Inferred))
        {
            return;
        }

        match ast_type {
            AstType::Named(name) => {
                self.validate_type_name(table, type_params, name, span, diagnostics);
            }
            AstType::Generic { name, type_args } => {
                if !self.validate_type_name(table, type_params, name, span, diagnostics) {
                    return;
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
            | AstType::RawPtr(elem)
            | AstType::Future(elem) => {
                self.validate_type_ref(table, type_params, elem, span, allow_self_type, diagnostics)
            }
            AstType::Function { params, ret } => {
                for ty in params.iter().chain(std::iter::once(ret.as_ref())) {
                    self.validate_type_ref(
                        table,
                        type_params,
                        ty,
                        span,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            AstType::SelfType => {
                if !allow_self_type {
                    diagnostics.push(Diagnostic::error_code(
                        E0204,
                        "Self type is only valid in method or behavior contexts",
                        span,
                    ));
                }
            }
            _ => unreachable!("builtin and inferred types returned before validation"),
        }
    }

    fn validate_type_name(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        name: &str,
        span: Span,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> bool {
        if !self.is_known_type_name(table, type_params, name) {
            self.push_unknown_type_symbol(diagnostics, name, span);
            return false;
        }
        true
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
}
