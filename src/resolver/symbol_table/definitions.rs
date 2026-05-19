use crate::ast::TypeParam;
use crate::error::Diagnostic;

use super::metadata_helpers::{
    resolver_type_parameter_bound_refs, resolver_type_parameter_bounds,
    resolver_type_parameter_names,
};

impl SymbolTable {
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

}
