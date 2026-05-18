use crate::ast::Declaration;
use crate::error::Diagnostic;

use super::metadata_helpers::{
    resolver_behavior_impl_method_key, resolver_behavior_method_signatures,
    resolver_behavior_method_types, resolver_field_types, resolver_method_key,
    resolver_value_signature, resolver_variant_names,
};
use super::symbol_table::TypeLikeMembers;
use super::{Namespace, Resolver, SymbolTable};

impl Resolver {
    pub(super) fn define_declaration(
        &self,
        table: &mut SymbolTable,
        decl: &Declaration,
    ) -> Result<(), Box<Diagnostic>> {
        match decl {
            Declaration::Function {
                name,
                type_params,
                params,
                return_type,
                public,
                span,
                ..
            } => {
                table.define_value(
                    name,
                    *public,
                    resolver_value_signature(params, return_type, type_params),
                    *span,
                )?;
            }
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                public,
                span,
                ..
            } => {
                table.define_value(
                    &resolver_method_key(type_name, method_name),
                    *public,
                    resolver_value_signature(params, return_type, type_params),
                    *span,
                )?;
            }
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
                    resolver_behavior_method_signatures(methods),
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
                table.define(Namespace::Module, &source, false, None, *span)?;
                for name in names {
                    table.define(Namespace::Import, name, false, Some(source.clone()), *span)?;
                }
            }
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                ..
            } => {
                for method in methods {
                    if let Declaration::Function {
                        name,
                        type_params,
                        params,
                        return_type,
                        public,
                        span,
                        ..
                    } = method
                    {
                        let key = if let Some(behavior) = behavior {
                            resolver_behavior_impl_method_key(
                                type_name,
                                name,
                                behavior,
                                behavior_type_args,
                            )
                        } else {
                            resolver_method_key(type_name, name)
                        };
                        table.define_value(
                            &key,
                            *public,
                            resolver_value_signature(params, return_type, type_params),
                            *span,
                        )?;
                    }
                }
            }
            Declaration::Requires { .. }
            | Declaration::Derive { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
        Ok(())
    }
}
