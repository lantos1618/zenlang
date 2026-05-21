use crate::ast::Declaration;
use crate::error::Diagnostic;

use super::{Resolver, SymbolTable};

mod behavior_associations;
mod callables;
mod top_level_expr;
mod type_declarations;

use callables::{
    FunctionDeclarationValidation, ImplBlockDeclarationValidation, MethodDeclarationValidation,
};

impl Resolver {
    pub(super) fn validate_declaration_types(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match decl {
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.validate_function_declaration(FunctionDeclarationValidation {
                    table,
                    type_params,
                    params,
                    return_type,
                    body,
                    span: *span,
                    diagnostics,
                });
            }
            Declaration::Method {
                type_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => {
                self.validate_method_declaration(MethodDeclarationValidation {
                    table,
                    type_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span: *span,
                    diagnostics,
                });
            }
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => {
                self.validate_struct_declaration(table, name, type_params, fields, diagnostics);
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.validate_enum_declaration(table, type_params, variants, diagnostics);
            }
            Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } => {
                self.validate_behavior_declaration(table, name, type_params, methods, diagnostics);
            }
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                span,
                ..
            } => {
                self.validate_impl_block_declaration(ImplBlockDeclarationValidation {
                    table,
                    type_name,
                    behavior,
                    behavior_type_args,
                    methods,
                    span: *span,
                    diagnostics,
                });
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                self.validate_requires_declaration(
                    table,
                    type_name,
                    behavior,
                    behavior_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::Derive {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                self.validate_derive_declaration(
                    table,
                    type_name,
                    behavior,
                    behavior_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } => {
                self.validate_behavior_extends_declaration(
                    table,
                    behavior,
                    parent,
                    parent_type_args,
                    *span,
                    diagnostics,
                );
            }
            Declaration::TopLevelExpr { expr, .. } => {
                self.validate_top_level_expr_declaration(table, expr, diagnostics);
            }
        }
    }
}
