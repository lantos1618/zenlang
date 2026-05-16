use std::collections::HashMap;

use crate::ast::{AstType, TypeParam};
use crate::error::{Diagnostic, Span};

use super::{
    behavior_ref_display, resolver_type_parameter_bound_refs, resolver_type_parameter_bounds,
    resolver_type_parameter_names,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

pub type MethodSignatureMetadata = (String, Vec<String>, String);
pub type TypeParameterBoundMetadata = (String, String);

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorMethodTypeMetadata {
    pub name: String,
    pub parameter_names: Vec<String>,
    pub parameter_types: Vec<AstType>,
    pub return_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorRefMetadata {
    pub name: String,
    pub type_args: Vec<AstType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypeParameterBoundRefMetadata {
    pub type_parameter: String,
    pub behavior: String,
    pub type_args: Vec<AstType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Namespace {
    Value,
    Type,
    Module,
    Import,
    Local,
    Behavior,
    Variant,
}

impl Namespace {
    pub(crate) fn diagnostic_name(self) -> &'static str {
        match self {
            Namespace::Value => "value",
            Namespace::Type => "type",
            Namespace::Module => "module",
            Namespace::Import => "import",
            Namespace::Local => "local",
            Namespace::Behavior => "behavior",
            Namespace::Variant => "variant",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub id: SymbolId,
    pub namespace: Namespace,
    pub name: String,
    pub is_public: bool,
    pub import_source: Option<String>,
    pub parameter_count: Option<usize>,
    pub parameter_names: Option<Vec<String>>,
    pub parameter_types: Option<Vec<AstType>>,
    pub parameter_type_names: Option<Vec<String>>,
    pub return_type: Option<AstType>,
    pub return_type_name: Option<String>,
    pub type_parameter_count: Option<usize>,
    pub type_parameter_names: Option<Vec<String>>,
    pub type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
    pub type_parameter_bound_refs: Option<Vec<TypeParameterBoundRefMetadata>>,
    pub field_count: Option<usize>,
    pub field_types: Option<Vec<(String, AstType)>>,
    pub field_type_names: Option<Vec<(String, String)>>,
    pub variant_names: Option<Vec<String>>,
    pub variant_owner_name: Option<String>,
    pub variant_payload_count: Option<usize>,
    pub variant_payload_type: Option<AstType>,
    pub variant_payload_type_name: Option<String>,
    pub behavior_method_signatures: Option<Vec<MethodSignatureMetadata>>,
    pub behavior_method_types: Option<Vec<BehaviorMethodTypeMetadata>>,
    pub behavior_parent_names: Option<Vec<String>>,
    pub behavior_parent_refs: Option<Vec<BehaviorRefMetadata>>,
    pub behavior_impl_names: Option<Vec<String>>,
    pub behavior_impl_refs: Option<Vec<BehaviorRefMetadata>>,
    pub behavior_required_names: Option<Vec<String>>,
    pub behavior_required_refs: Option<Vec<BehaviorRefMetadata>>,
    pub is_mutable: Option<bool>,
    pub scope_id: u32,
    pub definition_span: Span,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct SymbolMetadata {
    import_source: Option<String>,
    parameter_count: Option<usize>,
    pub(super) parameter_names: Option<Vec<String>>,
    pub(super) parameter_types: Option<Vec<AstType>>,
    pub(super) parameter_type_names: Option<Vec<String>>,
    pub(super) return_type: Option<AstType>,
    pub(super) return_type_name: Option<String>,
    pub(super) type_parameter_count: Option<usize>,
    pub(super) type_parameter_names: Option<Vec<String>>,
    pub(super) type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
    pub(super) type_parameter_bound_refs: Option<Vec<TypeParameterBoundRefMetadata>>,
    field_count: Option<usize>,
    field_types: Option<Vec<(String, AstType)>>,
    field_type_names: Option<Vec<(String, String)>>,
    variant_names: Option<Vec<String>>,
    variant_owner_name: Option<String>,
    variant_payload_count: Option<usize>,
    variant_payload_type: Option<AstType>,
    variant_payload_type_name: Option<String>,
    behavior_method_signatures: Option<Vec<MethodSignatureMetadata>>,
    behavior_method_types: Option<Vec<BehaviorMethodTypeMetadata>>,
    behavior_parent_names: Option<Vec<String>>,
    behavior_parent_refs: Option<Vec<BehaviorRefMetadata>>,
    behavior_impl_names: Option<Vec<String>>,
    behavior_impl_refs: Option<Vec<BehaviorRefMetadata>>,
    behavior_required_names: Option<Vec<String>>,
    behavior_required_refs: Option<Vec<BehaviorRefMetadata>>,
    is_mutable: Option<bool>,
}

pub(super) struct ValueSignatureMetadata {
    pub(super) parameter_names: Vec<String>,
    pub(super) parameter_types: Vec<AstType>,
    pub(super) parameter_type_names: Vec<String>,
    pub(super) return_type: AstType,
    pub(super) return_type_name: String,
    pub(super) type_parameter_count: usize,
    pub(super) type_parameter_names: Vec<String>,
    pub(super) type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
    pub(super) type_parameter_bound_refs: Vec<TypeParameterBoundRefMetadata>,
}

pub(super) enum TypeLikeMembers {
    Fields(Vec<(String, AstType, String)>),
    Variants(Vec<String>),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    pub(super) symbols: Vec<Symbol>,
    pub(super) by_name: HashMap<(Namespace, String), SymbolId>,
    pub(super) by_scoped_name: HashMap<(Namespace, String, u32), SymbolId>,
    pub(super) next_scope_id: u32,
}

#[derive(Debug, Clone)]
pub(super) struct ScopeStack {
    pub(super) current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ScopeStack {
    pub(super) fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    pub(super) fn with_parent(current_scope_id: u32, parent: &ScopeStack) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    pub(super) fn contains(&self, name: &str) -> bool {
        self.visible_names.contains_key(name)
    }

    pub(super) fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    pub(super) fn insert(&mut self, name: String, mutable: bool) {
        self.visible_names.insert(name, mutable);
    }
}

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

    pub(super) fn record_behavior_parent(
        &mut self,
        behavior: &str,
        parent_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Behavior && symbol.name == behavior)
        {
            let parent = behavior_ref_display(&parent_ref.name, &parent_ref.type_args);
            let parents = symbol.behavior_parent_names.get_or_insert_with(Vec::new);
            if parents.iter().any(|recorded| recorded == &parent) {
                return false;
            }
            parents.push(parent);
            symbol
                .behavior_parent_refs
                .get_or_insert_with(Vec::new)
                .push(parent_ref);
        }
        true
    }

    pub(super) fn record_behavior_impl(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Type && symbol.name == type_name)
        {
            let behavior = behavior_ref_display(&behavior_ref.name, &behavior_ref.type_args);
            let impls = symbol.behavior_impl_names.get_or_insert_with(Vec::new);
            if impls.iter().any(|recorded| recorded == &behavior) {
                return false;
            }
            impls.push(behavior);
            symbol
                .behavior_impl_refs
                .get_or_insert_with(Vec::new)
                .push(behavior_ref);
        }
        true
    }

    pub(super) fn record_behavior_required(
        &mut self,
        type_name: &str,
        behavior_ref: BehaviorRefMetadata,
    ) -> bool {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Type && symbol.name == type_name)
        {
            let behavior = behavior_ref_display(&behavior_ref.name, &behavior_ref.type_args);
            let required = symbol.behavior_required_names.get_or_insert_with(Vec::new);
            if required.iter().any(|recorded| recorded == &behavior) {
                return false;
            }
            required.push(behavior);
            symbol
                .behavior_required_refs
                .get_or_insert_with(Vec::new)
                .push(behavior_ref);
        }
        true
    }
}
