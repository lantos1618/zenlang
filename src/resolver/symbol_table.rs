use std::collections::HashMap;

use crate::ast::{AstType, TypeParam};
use crate::error::{Diagnostic, Span};

use super::metadata_helpers::{
    resolver_type_parameter_bound_refs, resolver_type_parameter_bounds,
    resolver_type_parameter_names,
};

include!("symbol_table/core.rs");

impl SymbolTable {
    pub fn lookup(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        let id = self.by_name.get(&(namespace, name.to_string()))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn lookup_variant(&self, owner_name: &str, name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|symbol| {
            symbol.namespace == Namespace::Variant
                && symbol.name == name
                && symbol.variant_owner_name.as_deref() == Some(owner_name)
        })
    }

    pub fn lookup_scoped(&self, namespace: Namespace, name: &str) -> Option<&Symbol> {
        self.symbols
            .iter()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
    }

    pub fn lookup_in_scope(
        &self,
        namespace: Namespace,
        name: &str,
        scope_id: u32,
    ) -> Option<&Symbol> {
        let id = self
            .by_scoped_name
            .get(&(namespace, name.to_string(), scope_id))?;
        self.symbols.get(id.0 as usize)
    }

    pub fn symbols(&self) -> &[Symbol] {
        &self.symbols
    }

    pub(super) fn define(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        import_source: Option<String>,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        self.define_in_scope(
            namespace,
            name,
            is_public,
            SymbolMetadata {
                import_source,
                parameter_count: None,
                parameter_names: None,
                parameter_types: None,
                parameter_type_names: None,
                return_type: None,
                return_type_name: None,
                type_parameter_count: None,
                type_parameter_names: None,
                type_parameter_bounds: None,
                type_parameter_bound_refs: None,
                field_count: None,
                field_types: None,
                field_type_names: None,
                variant_names: None,
                variant_owner_name: None,
                variant_payload_count: None,
                variant_payload_type: None,
                variant_payload_type_name: None,
                behavior_method_signatures: None,
                behavior_method_types: None,
                behavior_parent_names: None,
                behavior_parent_refs: None,
                behavior_impl_names: None,
                behavior_impl_refs: None,
                behavior_required_names: None,
                behavior_required_refs: None,
                is_mutable: None,
            },
            0,
            definition_span,
        )
    }

    pub(super) fn define_value(
        &mut self,
        name: &str,
        is_public: bool,
        signature: ValueSignatureMetadata,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let parameter_count = signature.parameter_type_names.len();
        self.define_in_scope(
            Namespace::Value,
            name,
            is_public,
            SymbolMetadata {
                import_source: None,
                parameter_count: Some(parameter_count),
                parameter_names: Some(signature.parameter_names),
                parameter_types: Some(signature.parameter_types),
                parameter_type_names: Some(signature.parameter_type_names),
                return_type: Some(signature.return_type),
                return_type_name: Some(signature.return_type_name),
                type_parameter_count: Some(signature.type_parameter_count),
                type_parameter_names: Some(signature.type_parameter_names),
                type_parameter_bounds: Some(signature.type_parameter_bounds),
                type_parameter_bound_refs: Some(signature.type_parameter_bound_refs),
                field_count: None,
                field_types: None,
                field_type_names: None,
                variant_names: None,
                variant_owner_name: None,
                variant_payload_count: None,
                variant_payload_type: None,
                variant_payload_type_name: None,
                behavior_method_signatures: None,
                behavior_method_types: None,
                behavior_parent_names: None,
                behavior_parent_refs: None,
                behavior_impl_names: None,
                behavior_impl_refs: None,
                behavior_required_names: None,
                behavior_required_refs: None,
                is_mutable: None,
            },
            0,
            definition_span,
        )
    }

    pub(super) fn define_type_like(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        type_params: &[TypeParam],
        members: TypeLikeMembers,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let (field_types, field_type_names, variant_names) = match members {
            TypeLikeMembers::Fields(fields) => {
                let typed = fields
                    .iter()
                    .map(|(name, ty, _)| (name.clone(), ty.clone()))
                    .collect();
                let names = fields
                    .into_iter()
                    .map(|(name, _, type_name)| (name, type_name))
                    .collect();
                (Some(typed), Some(names), None)
            }
            TypeLikeMembers::Variants(variants) => (None, None, Some(variants)),
        };
        let field_count = field_type_names.as_ref().map(Vec::len);
        let type_parameter_count = type_params.len();
        self.define_in_scope(
            namespace,
            name,
            is_public,
            SymbolMetadata {
                import_source: None,
                parameter_count: None,
                parameter_names: None,
                parameter_types: None,
                parameter_type_names: None,
                return_type: None,
                return_type_name: None,
                type_parameter_count: Some(type_parameter_count),
                type_parameter_names: Some(resolver_type_parameter_names(type_params)),
                type_parameter_bounds: Some(resolver_type_parameter_bounds(type_params)),
                type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
                field_count,
                field_types,
                field_type_names,
                variant_names,
                variant_owner_name: None,
                variant_payload_count: None,
                variant_payload_type: None,
                variant_payload_type_name: None,
                behavior_method_signatures: None,
                behavior_method_types: None,
                behavior_parent_names: None,
                behavior_parent_refs: None,
                behavior_impl_names: None,
                behavior_impl_refs: None,
                behavior_required_names: None,
                behavior_required_refs: None,
                is_mutable: None,
            },
            0,
            definition_span,
        )
    }

    pub(super) fn define_variant(
        &mut self,
        owner_name: &str,
        name: &str,
        is_public: bool,
        variant_payload_type: Option<AstType>,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let variant_payload_count = usize::from(variant_payload_type.is_some());
        let variant_payload_type_name = variant_payload_type.as_ref().map(AstType::display_name);
        self.define_in_scope(
            Namespace::Variant,
            name,
            is_public,
            SymbolMetadata {
                import_source: None,
                parameter_count: None,
                parameter_names: None,
                parameter_types: None,
                parameter_type_names: None,
                return_type: None,
                return_type_name: None,
                type_parameter_count: None,
                type_parameter_names: None,
                type_parameter_bounds: None,
                type_parameter_bound_refs: None,
                field_count: None,
                field_types: None,
                field_type_names: None,
                variant_names: None,
                variant_owner_name: Some(owner_name.to_string()),
                variant_payload_count: Some(variant_payload_count),
                variant_payload_type,
                variant_payload_type_name,
                behavior_method_signatures: None,
                behavior_method_types: None,
                behavior_parent_names: None,
                behavior_parent_refs: None,
                behavior_impl_names: None,
                behavior_impl_refs: None,
                behavior_required_names: None,
                behavior_required_refs: None,
                is_mutable: None,
            },
            0,
            definition_span,
        )
    }

    pub(super) fn define_behavior(
        &mut self,
        name: &str,
        is_public: bool,
        type_params: &[TypeParam],
        behavior_method_signatures: Vec<MethodSignatureMetadata>,
        behavior_method_types: Vec<BehaviorMethodTypeMetadata>,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let type_parameter_count = type_params.len();
        self.define_in_scope(
            Namespace::Behavior,
            name,
            is_public,
            SymbolMetadata {
                import_source: None,
                parameter_count: None,
                parameter_names: None,
                parameter_types: None,
                parameter_type_names: None,
                return_type: None,
                return_type_name: None,
                type_parameter_count: Some(type_parameter_count),
                type_parameter_names: Some(resolver_type_parameter_names(type_params)),
                type_parameter_bounds: Some(resolver_type_parameter_bounds(type_params)),
                type_parameter_bound_refs: Some(resolver_type_parameter_bound_refs(type_params)),
                field_count: None,
                field_types: None,
                field_type_names: None,
                variant_names: None,
                variant_owner_name: None,
                variant_payload_count: None,
                variant_payload_type: None,
                variant_payload_type_name: None,
                behavior_method_signatures: Some(behavior_method_signatures),
                behavior_method_types: Some(behavior_method_types),
                behavior_parent_names: None,
                behavior_parent_refs: None,
                behavior_impl_names: None,
                behavior_impl_refs: None,
                behavior_required_names: None,
                behavior_required_refs: None,
                is_mutable: None,
            },
            0,
            definition_span,
        )
    }

    fn define_in_scope(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
        metadata: SymbolMetadata,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        let scoped_key = (namespace, name.to_string(), scope_id);
        let duplicate = if namespace == Namespace::Variant {
            self.symbols.iter().any(|symbol| {
                symbol.namespace == Namespace::Variant
                    && symbol.name == name
                    && symbol.variant_owner_name == metadata.variant_owner_name
            })
        } else {
            self.by_scoped_name.contains_key(&scoped_key)
        };
        if duplicate {
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
        if namespace != Namespace::Local {
            self.by_name.insert((namespace, name.to_string()), id);
        }
        self.symbols.push(Symbol {
            id,
            namespace,
            name: name.to_string(),
            is_public,
            import_source: metadata.import_source,
            parameter_count: metadata.parameter_count,
            parameter_names: metadata.parameter_names,
            parameter_types: metadata.parameter_types,
            parameter_type_names: metadata.parameter_type_names,
            return_type: metadata.return_type,
            return_type_name: metadata.return_type_name,
            type_parameter_count: metadata.type_parameter_count,
            type_parameter_names: metadata.type_parameter_names,
            type_parameter_bounds: metadata.type_parameter_bounds,
            type_parameter_bound_refs: metadata.type_parameter_bound_refs,
            field_count: metadata.field_count,
            field_types: metadata.field_types,
            field_type_names: metadata.field_type_names,
            variant_names: metadata.variant_names,
            variant_owner_name: metadata.variant_owner_name,
            variant_payload_count: metadata.variant_payload_count,
            variant_payload_type: metadata.variant_payload_type,
            variant_payload_type_name: metadata.variant_payload_type_name,
            behavior_method_signatures: metadata.behavior_method_signatures,
            behavior_method_types: metadata.behavior_method_types,
            behavior_parent_names: metadata.behavior_parent_names,
            behavior_parent_refs: metadata.behavior_parent_refs,
            behavior_impl_names: metadata.behavior_impl_names,
            behavior_impl_refs: metadata.behavior_impl_refs,
            behavior_required_names: metadata.behavior_required_names,
            behavior_required_refs: metadata.behavior_required_refs,
            is_mutable: metadata.is_mutable,
            scope_id,
            definition_span,
        });
        self.by_scoped_name.insert(scoped_key, id);
        Ok(id)
    }

    pub(super) fn define_local(
        &mut self,
        name: &str,
        mutable: bool,
        scope_id: u32,
        definition_span: Span,
    ) -> Result<SymbolId, Box<Diagnostic>> {
        self.define_in_scope(
            Namespace::Local,
            name,
            false,
            SymbolMetadata {
                is_mutable: Some(mutable),
                ..SymbolMetadata::default()
            },
            scope_id,
            definition_span,
        )
    }

    pub(super) fn new_scope(&mut self) -> u32 {
        self.next_scope_id += 1;
        self.next_scope_id
    }
}

include!("symbol_table/behavior_edges.rs");
