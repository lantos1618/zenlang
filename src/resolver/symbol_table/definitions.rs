use crate::ast::TypeParam;
use crate::error::Diagnostic;

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
            empty_symbol_metadata(import_source),
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
        self.define_in_scope(
            Namespace::Value,
            name,
            is_public,
            value_symbol_metadata(signature),
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
        self.define_in_scope(
            namespace,
            name,
            is_public,
            type_like_symbol_metadata(type_params, members),
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
        self.define_in_scope(
            Namespace::Variant,
            name,
            is_public,
            variant_symbol_metadata(owner_name, variant_payload_type),
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
        self.define_in_scope(
            Namespace::Behavior,
            name,
            is_public,
            behavior_symbol_metadata(type_params, behavior_method_signatures, behavior_method_types),
            0,
            definition_span,
        )
    }

}
