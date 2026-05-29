use crate::ast::{behavior_impl_method_symbol_key, method_symbol_key, Declaration};
use crate::error::Diagnostic;

use super::metadata_helpers::{
    resolver_behavior_method_types, resolver_field_types, resolver_value_signature,
    resolver_variant_names,
};
use super::symbol_table::TypeLikeMembers;
use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn define_declaration(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
    ) -> Result<(), Diagnostic> {
        if let Some(callable) = decl.as_callable() {
            let key = match decl {
                Declaration::Method { type_name, .. } => {
                    method_symbol_key(type_name, callable.name)
                }
                _ => callable.name.to_string(),
            };
            table.define_value(
                &key,
                callable.public,
                resolver_value_signature(
                    callable.params,
                    callable.return_type,
                    callable.type_params,
                ),
                callable.span,
            )?;
            return Ok(());
        }

        match decl {
            Declaration::Struct {
                name,
                type_params,
                fields,
                public,
                span,
                ..
            } => {
                table.define_type_like(
                    Namespace::Type,
                    name,
                    *public,
                    type_params,
                    TypeLikeMembers::Fields(resolver_field_types(fields)),
                    *span,
                )?;
            }
            Declaration::Enum {
                name,
                type_params,
                variants,
                public,
                span,
                ..
            } => {
                table.define_type_like(
                    Namespace::Type,
                    name,
                    *public,
                    type_params,
                    TypeLikeMembers::Variants(resolver_variant_names(variants)),
                    *span,
                )?;
                for variant in variants {
                    table.define_variant(
                        name,
                        &variant.name,
                        *public,
                        variant.payload.clone(),
                        variant.span,
                    )?;
                }
            }
            Declaration::Behavior {
                name,
                type_params,
                methods,
                public,
                span,
                ..
            } => {
                table.define_behavior(
                    name,
                    *public,
                    type_params,
                    resolver_behavior_method_types(methods),
                    *span,
                )?;
            }
            Declaration::Import {
                names,
                module_path,
                span,
                ..
            } => {
                let source = module_path.join(".");
                // Several imports can share a module root (`{ io } = std` and
                // `{ math } = std`). The module symbol is bookkeeping for the
                // symbol-table JSON, so define it once and let later imports
                // from the same root reuse it rather than collide (E0200).
                if table.lookup(Namespace::Module, &source).is_none() {
                    table.define(Namespace::Module, &source, false, None, *span)?;
                }
                for name in names {
                    table.define(Namespace::Import, name, false, Some(source.clone()), *span)?;
                }
            }
            Declaration::ImplBlock {
                type_name,
                type_args,
                behavior,
                behavior_type_args,
                methods,
                ..
            } => {
                for method in methods {
                    if let Some(method) = method.as_callable() {
                        let key = behavior_impl_method_symbol_key(
                            type_name,
                            method.name,
                            behavior.as_deref(),
                            behavior_type_args,
                            type_args,
                        );
                        table.define_value(
                            &key,
                            method.public,
                            resolver_value_signature(
                                method.params,
                                method.return_type,
                                method.type_params,
                            ),
                            method.span,
                        )?;
                    }
                }
            }
            // Top-level `name = value` / `name := value` defines a module-level
            // value symbol so other declarations can reference it.
            Declaration::TopLevelExpr { expr, span } => {
                if let Some(name) = top_level_binding_name(expr) {
                    table.define_value(
                        name,
                        false,
                        resolver_value_signature(&[], &None, &[]),
                        *span,
                    )?;
                }
            }
            Declaration::Function { .. }
            | Declaration::Method { .. }
            | Declaration::Requires { .. }
            | Declaration::Derive { .. }
            | Declaration::BehaviorExtends { .. } => {}
        }
        Ok(())
    }
}

/// The bound name of a top-level binding (`name = value` / `name := value`),
/// which the parser lowers to a single-statement `VarDecl` block.
fn top_level_binding_name(expr: &crate::ast::Expression) -> Option<&str> {
    use crate::ast::{Expression, Statement};
    if let Expression::Block {
        statements,
        expr: None,
        ..
    } = expr
    {
        if let [Statement::VarDecl { name, .. }] = statements.as_slice() {
            return Some(name);
        }
    }
    None
}
