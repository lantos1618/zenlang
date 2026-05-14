use std::collections::{HashMap, HashSet};

use crate::ast::{
    AstType, BehaviorMethod, Declaration, Expression, Param, Pattern, Program, Statement,
    StringPart, StructField, TypeParam,
};
use crate::error::{Diagnostic, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

pub type MethodSignatureMetadata = (String, Vec<String>, String);
pub type TypeParameterBoundMetadata = (String, String);

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorMethodTypeMetadata {
    pub name: String,
    pub parameter_types: Vec<AstType>,
    pub return_type: AstType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BehaviorRefMetadata {
    pub name: String,
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
    parameter_names: Option<Vec<String>>,
    parameter_types: Option<Vec<AstType>>,
    parameter_type_names: Option<Vec<String>>,
    return_type: Option<AstType>,
    return_type_name: Option<String>,
    type_parameter_count: Option<usize>,
    type_parameter_names: Option<Vec<String>>,
    type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
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

struct ValueSignatureMetadata {
    parameter_names: Vec<String>,
    parameter_types: Vec<AstType>,
    parameter_type_names: Vec<String>,
    return_type: AstType,
    return_type_name: String,
    type_parameter_count: usize,
    type_parameter_names: Vec<String>,
    type_parameter_bounds: Vec<TypeParameterBoundMetadata>,
}

enum TypeLikeMembers {
    Fields(Vec<(String, AstType, String)>),
    Variants(Vec<String>),
}

#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    by_name: HashMap<(Namespace, String), SymbolId>,
    by_scoped_name: HashMap<(Namespace, String, u32), SymbolId>,
    next_scope_id: u32,
}

#[derive(Debug, Clone)]
struct ScopeStack {
    current_scope_id: u32,
    visible_names: HashMap<String, bool>,
}

impl ScopeStack {
    fn new(current_scope_id: u32) -> Self {
        Self {
            current_scope_id,
            visible_names: HashMap::new(),
        }
    }

    fn with_parent(current_scope_id: u32, parent: &ScopeStack) -> Self {
        Self {
            current_scope_id,
            visible_names: parent.visible_names.clone(),
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.visible_names.contains_key(name)
    }

    fn is_mutable(&self, name: &str) -> bool {
        self.visible_names.get(name).copied().unwrap_or(false)
    }

    fn insert(&mut self, name: String, mutable: bool) {
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

    #[cfg(test)]
    pub(crate) fn remove_for_test(&mut self, namespace: Namespace, name: &str) {
        self.symbols
            .retain(|symbol| symbol.namespace != namespace || symbol.name != name);
        self.by_name.clear();
        self.by_scoped_name.clear();
        for (idx, symbol) in self.symbols.iter_mut().enumerate() {
            symbol.id = SymbolId(idx as u32);
            if symbol.namespace != Namespace::Local {
                self.by_name
                    .insert((symbol.namespace, symbol.name.clone()), symbol.id);
            }
            self.by_scoped_name.insert(
                (symbol.namespace, symbol.name.clone(), symbol.scope_id),
                symbol.id,
            );
        }
    }

    #[cfg(test)]
    pub(crate) fn set_public_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_public: bool,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.is_public = is_public;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_import_source_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        import_source: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.import_source = import_source;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_for_test(&mut self, name: &str, is_mutable: Option<bool>) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == Namespace::Local && symbol.name == name)
        {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_local_mutability_in_scope_for_test(
        &mut self,
        name: &str,
        scope_id: u32,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self.symbols.iter_mut().find(|symbol| {
            symbol.namespace == Namespace::Local
                && symbol.name == name
                && symbol.scope_id == scope_id
        }) {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_mutability_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        is_mutable: Option<bool>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.is_mutable = is_mutable;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_count = parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_type_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_type_names = parameter_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.parameter_names = parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_return_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        return_type_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.return_type_name = return_type_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_count = type_parameter_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_names = type_parameter_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_type_parameter_bounds_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        type_parameter_bounds: Option<Vec<TypeParameterBoundMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.type_parameter_bounds = type_parameter_bounds;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_method_signatures_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_method_signatures: Option<Vec<MethodSignatureMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_method_signatures = behavior_method_signatures;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_parent_names = behavior_parent_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_parent_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_parent_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_parent_refs = behavior_parent_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_impl_names = behavior_impl_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_impl_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_impl_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_impl_refs = behavior_impl_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_required_names = behavior_required_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_behavior_required_refs_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        behavior_required_refs: Option<Vec<BehaviorRefMetadata>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.behavior_required_refs = behavior_required_refs;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.field_count = field_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_field_type_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        field_type_names: Option<Vec<(String, String)>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.field_type_names = field_type_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_names_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_names: Option<Vec<String>>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_names = variant_names;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_owner_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_owner_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_owner_name = variant_owner_name;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_count_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_count: Option<usize>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_payload_count = variant_payload_count;
        }
    }

    #[cfg(test)]
    pub(crate) fn set_variant_payload_type_name_for_test(
        &mut self,
        namespace: Namespace,
        name: &str,
        variant_payload_type_name: Option<String>,
    ) {
        if let Some(symbol) = self
            .symbols
            .iter_mut()
            .find(|symbol| symbol.namespace == namespace && symbol.name == name)
        {
            symbol.variant_payload_type_name = variant_payload_type_name;
        }
    }

    fn define(
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

    fn define_value(
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

    fn define_type_like(
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

    fn define_variant(
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

    fn define_behavior(
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

    fn define_local(
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

    fn new_scope(&mut self) -> u32 {
        self.next_scope_id += 1;
        self.next_scope_id
    }

    fn record_behavior_parent(&mut self, behavior: &str, parent_ref: BehaviorRefMetadata) -> bool {
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

    fn record_behavior_impl(&mut self, type_name: &str, behavior_ref: BehaviorRefMetadata) -> bool {
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

    fn record_behavior_required(
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

        for decl in &program.declarations {
            self.validate_declaration_types(&mut table, decl, &mut diagnostics);
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
                    &format!("{type_name}.{method_name}"),
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
                type_name, methods, ..
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
                        table.define_value(
                            &format!("{type_name}.{name}"),
                            *public,
                            resolver_value_signature(params, return_type, type_params),
                            *span,
                        )?;
                    }
                }
            }
            Declaration::Requires { .. }
            | Declaration::BehaviorExtends { .. }
            | Declaration::TopLevelExpr { .. }
            | Declaration::Error { .. } => {}
        }
        Ok(())
    }

    fn validate_declaration_types(
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
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                self.validate_params(table, type_params, params, false, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        false,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, false, diagnostics);
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
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                self.validate_type_param_constraints(table, type_params, true, diagnostics);
                self.validate_params(table, type_params, params, true, diagnostics);
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        true,
                        diagnostics,
                    );
                }
                let scope_id = table.new_scope();
                let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                self.validate_expr_refs(table, type_params, body, &mut locals, true, diagnostics);
            }
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                let mut seen_fields = HashSet::new();
                for field in fields {
                    if !seen_fields.insert(field.name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0211",
                            format!("duplicate field `{}` for struct `{name}`", field.name),
                            field.span,
                        ));
                    }
                    self.validate_type_ref(
                        table,
                        type_params,
                        &field.ty,
                        field.span,
                        false,
                        diagnostics,
                    );
                    if let Some(default) = &field.default {
                        let scope_id = table.new_scope();
                        let mut locals = ScopeStack::new(scope_id);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            default,
                            &mut locals,
                            false,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, false, diagnostics);
                for variant in variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type_ref(
                            table,
                            type_params,
                            payload,
                            variant.span,
                            false,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Behavior {
                name,
                type_params,
                methods,
                ..
            } => {
                self.validate_type_param_constraints(table, type_params, true, diagnostics);
                let mut seen_methods = HashSet::new();
                for method in methods {
                    if !seen_methods.insert(method.name.as_str()) {
                        diagnostics.push(Diagnostic::error(
                            "E0212",
                            format!("duplicate behavior method `{}` in `{name}`", method.name),
                            method.span,
                        ));
                    }
                    self.validate_params(table, type_params, &method.params, true, diagnostics);
                    if let Some(return_type) = &method.return_type {
                        self.validate_type_ref(
                            table,
                            type_params,
                            return_type,
                            method.span,
                            true,
                            diagnostics,
                        );
                    }
                    if let Some(default_body) = &method.default_body {
                        let scope_id = table.new_scope();
                        let mut locals =
                            self.param_locals(table, &method.params, scope_id, diagnostics);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            default_body,
                            &mut locals,
                            true,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::ImplBlock {
                type_name,
                behavior,
                behavior_type_args,
                methods,
                span,
                ..
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                if let Some(behavior) = behavior {
                    let behavior_known = self.is_known_behavior_name(table, behavior);
                    if !behavior_known {
                        diagnostics.push(Diagnostic::error(
                            "E0202",
                            format!("unknown behavior symbol '{behavior}'"),
                            *span,
                        ));
                    }
                    if self.is_known_type_name(table, &[], type_name) && behavior_known {
                        let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                        if !table.record_behavior_impl(
                            type_name,
                            BehaviorRefMetadata {
                                name: behavior.clone(),
                                type_args: behavior_type_args.clone(),
                            },
                        ) {
                            diagnostics.push(Diagnostic::error(
                                "E0217",
                                format!(
                                    "duplicate behavior implementation `{behavior_display}` for `{type_name}`"
                                ),
                                *span,
                            ));
                        }
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
                for method in methods {
                    if let Declaration::Function {
                        type_params,
                        params,
                        return_type,
                        body,
                        span,
                        ..
                    } = method
                    {
                        self.validate_type_param_constraints(table, type_params, true, diagnostics);
                        self.validate_params(table, type_params, params, true, diagnostics);
                        if let Some(return_type) = return_type {
                            self.validate_type_ref(
                                table,
                                type_params,
                                return_type,
                                *span,
                                true,
                                diagnostics,
                            );
                        }
                        let scope_id = table.new_scope();
                        let mut locals = self.param_locals(table, params, scope_id, diagnostics);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            body,
                            &mut locals,
                            true,
                            diagnostics,
                        );
                    }
                }
            }
            Declaration::Import { .. } | Declaration::Error { .. } => {}
            Declaration::Requires {
                type_name,
                behavior,
                behavior_type_args,
                span,
            } => {
                if !self.is_known_type_name(table, &[], type_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{type_name}'"),
                        *span,
                    ));
                }
                if !self.is_known_behavior_name(table, behavior) {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                } else if self.is_known_type_name(table, &[], type_name) {
                    let behavior_display = behavior_ref_display(behavior, behavior_type_args);
                    if !table.record_behavior_required(
                        type_name,
                        BehaviorRefMetadata {
                            name: behavior.clone(),
                            type_args: behavior_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0216",
                            format!(
                                "duplicate required behavior `{behavior_display}` for `{type_name}`"
                            ),
                            *span,
                        ));
                    }
                }
                for type_arg in behavior_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
            }
            Declaration::BehaviorExtends {
                behavior,
                parent,
                parent_type_args,
                span,
            } => {
                let behavior_known = self.is_known_behavior_name(table, behavior);
                let parent_known = self.is_known_behavior_name(table, parent);
                if !behavior_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{behavior}'"),
                        *span,
                    ));
                }
                if !parent_known {
                    diagnostics.push(Diagnostic::error(
                        "E0202",
                        format!("unknown behavior symbol '{parent}'"),
                        *span,
                    ));
                }
                for type_arg in parent_type_args {
                    self.validate_type_ref(table, &[], type_arg, *span, false, diagnostics);
                }
                if behavior_known && parent_known {
                    let parent_display = behavior_ref_display(parent, parent_type_args);
                    if !table.record_behavior_parent(
                        behavior,
                        BehaviorRefMetadata {
                            name: parent.clone(),
                            type_args: parent_type_args.clone(),
                        },
                    ) {
                        diagnostics.push(Diagnostic::error(
                            "E0215",
                            format!(
                                "duplicate behavior parent `{parent_display}` for `{behavior}`"
                            ),
                            *span,
                        ));
                    }
                }
            }
            Declaration::TopLevelExpr { expr, .. } => {
                let scope_id = table.new_scope();
                self.validate_expr_refs(
                    table,
                    &[],
                    expr,
                    &mut ScopeStack::new(scope_id),
                    false,
                    diagnostics,
                );
            }
        }
    }

    fn validate_params(
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

    fn validate_type_param_constraints(
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

    fn validate_type_ref(
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

    fn is_known_type_name(
        &self,
        table: &SymbolTable,
        type_params: &[TypeParam],
        name: &str,
    ) -> bool {
        table.lookup(Namespace::Type, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || type_params.iter().any(|type_param| type_param.name == name)
    }

    fn is_known_behavior_name(&self, table: &SymbolTable, name: &str) -> bool {
        table.lookup(Namespace::Behavior, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
    }

    fn validate_expr_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        expr: &Expression,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match expr {
            Expression::FunctionCall {
                name,
                module,
                type_args,
                args,
                span,
            } => {
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                if module.is_none() && !self.is_known_value_name(table, locals, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0203",
                        format!("unknown value symbol '{name}'"),
                        *span,
                    ));
                }
                for arg in args {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        arg,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Identifier { name, span } => {
                if !self.is_known_value_name(table, locals, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0203",
                        format!("unknown value symbol '{name}'"),
                        *span,
                    ));
                }
            }
            Expression::MethodCall {
                receiver,
                type_args,
                args,
                span,
                ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    receiver,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                for arg in args {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        arg,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::BinaryOp { left, right, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    left,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    right,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::UnaryOp { operand, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    operand,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::MemberAccess { object, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    object,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IndexAccess { object, index, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    object,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    index,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StructLiteral {
                name,
                type_args,
                fields,
                span,
            } => {
                if let Some(symbol) = table.lookup(Namespace::Type, name) {
                    if let Some(field_type_names) = symbol.field_type_names.as_ref() {
                        let expected_fields: HashSet<&str> = field_type_names
                            .iter()
                            .map(|(field_name, _)| field_name.as_str())
                            .collect();
                        let mut provided_fields = HashSet::new();

                        for (field_name, _) in fields {
                            if !provided_fields.insert(field_name.as_str()) {
                                diagnostics.push(Diagnostic::error(
                                    "E0208",
                                    format!("duplicate field `{field_name}` for struct `{name}`"),
                                    *span,
                                ));
                            }
                            if !expected_fields.contains(field_name.as_str()) {
                                diagnostics.push(Diagnostic::error(
                                    "E0209",
                                    format!("unknown field `{field_name}` for struct `{name}`"),
                                    *span,
                                ));
                            }
                        }

                        for expected_field in expected_fields {
                            if !provided_fields.contains(expected_field) {
                                diagnostics.push(Diagnostic::error(
                                    "E0210",
                                    format!("missing field `{expected_field}` for struct `{name}`"),
                                    *span,
                                ));
                            }
                        }
                    }
                } else if !self.is_known_type_name(table, type_params, name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{name}'"),
                        *span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                for (_, value) in fields {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        value,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::EnumVariant {
                enum_name,
                type_args,
                variant,
                payload,
                span,
            } => {
                if table.lookup(Namespace::Type, enum_name).is_some() {
                    if let Some(variant_symbol) = table.lookup_variant(enum_name, variant) {
                        match (
                            variant_symbol.variant_payload_count.unwrap_or(0),
                            payload.is_some(),
                        ) {
                            (1, false) => diagnostics.push(Diagnostic::error(
                                "E0206",
                                format!("enum variant `{enum_name}.{variant}` requires a payload"),
                                *span,
                            )),
                            (0, true) => diagnostics.push(Diagnostic::error(
                                "E0207",
                                format!(
                                    "enum variant `{enum_name}.{variant}` does not accept a payload"
                                ),
                                *span,
                            )),
                            _ => {}
                        }
                    } else {
                        diagnostics.push(Diagnostic::error(
                            "E0205",
                            format!("enum `{enum_name}` has no variant `{variant}`"),
                            *span,
                        ));
                    }
                } else if !self.is_known_type_name(table, type_params, enum_name) {
                    diagnostics.push(Diagnostic::error(
                        "E0201",
                        format!("unknown type symbol '{enum_name}'"),
                        *span,
                    ));
                }
                for type_arg in type_args {
                    self.validate_type_ref(
                        table,
                        type_params,
                        type_arg,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                if let Some(payload) = payload {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        payload,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::ArrayLiteral { elements, .. } => {
                for element in elements {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        element,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Match {
                scrutinee, arms, ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    scrutinee,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        let arm_scope_id = table.new_scope();
                        let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
                        self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
                        self.validate_expr_refs(
                            table,
                            type_params,
                            guard,
                            &mut arm_locals,
                            allow_self_type,
                            diagnostics,
                        );
                    }
                    let arm_scope_id = table.new_scope();
                    let mut arm_locals = ScopeStack::with_parent(arm_scope_id, locals);
                    self.bind_pattern_locals(table, &arm.pattern, &mut arm_locals, diagnostics);
                    self.validate_expr_refs(
                        table,
                        type_params,
                        &arm.body,
                        &mut arm_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::WhileLoop {
                condition, body, ..
            }
            | Expression::If {
                condition,
                then_body: body,
                ..
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    condition,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(
                    table,
                    type_params,
                    body,
                    &mut body_locals,
                    allow_self_type,
                    diagnostics,
                );
                if let Expression::If {
                    else_body: Some(else_body),
                    ..
                } = expr
                {
                    let else_scope_id = table.new_scope();
                    let mut else_locals = ScopeStack::with_parent(else_scope_id, locals);
                    self.validate_expr_refs(
                        table,
                        type_params,
                        else_body,
                        &mut else_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Loop { body, .. } => {
                let body_scope_id = table.new_scope();
                let mut body_locals = ScopeStack::with_parent(body_scope_id, locals);
                self.validate_expr_refs(
                    table,
                    type_params,
                    body,
                    &mut body_locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Block {
                statements, expr, ..
            } => {
                let block_scope_id = table.new_scope();
                let mut block_locals = ScopeStack::with_parent(block_scope_id, locals);
                for statement in statements {
                    self.validate_statement_refs(
                        table,
                        type_params,
                        statement,
                        &mut block_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
                if let Some(expr) = expr {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        expr,
                        &mut block_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Return { value, .. } => {
                if let Some(value) = value {
                    self.validate_expr_refs(
                        table,
                        type_params,
                        value,
                        locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
            Expression::Closure {
                params,
                return_type,
                body,
                span,
            } => {
                let closure_scope_id = table.new_scope();
                let mut closure_locals = ScopeStack::with_parent(closure_scope_id, locals);
                for param in params {
                    self.validate_type_ref(
                        table,
                        type_params,
                        &param.ty,
                        param.span,
                        allow_self_type,
                        diagnostics,
                    );
                    self.define_local_symbol(
                        table,
                        &param.name,
                        false,
                        param.span,
                        &mut closure_locals,
                        diagnostics,
                    );
                }
                if let Some(return_type) = return_type {
                    self.validate_type_ref(
                        table,
                        type_params,
                        return_type,
                        *span,
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_expr_refs(
                    table,
                    type_params,
                    body,
                    &mut closure_locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Cast {
                expr,
                target_type,
                span,
            } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_type_ref(
                    table,
                    type_params,
                    target_type,
                    *span,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::StringInterpolation { parts, .. } => {
                for part in parts {
                    if let StringPart::Expr(expr) = part {
                        self.validate_expr_refs(
                            table,
                            type_params,
                            expr,
                            locals,
                            allow_self_type,
                            diagnostics,
                        );
                    }
                }
            }
            Expression::Range { start, end, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    start,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    end,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::Defer { expr, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Expression::IntLiteral { .. }
            | Expression::FloatLiteral { .. }
            | Expression::StringLiteral { .. }
            | Expression::BoolLiteral { .. }
            | Expression::CharLiteral { .. }
            | Expression::Break { .. }
            | Expression::Continue { .. }
            | Expression::Error { .. } => {}
        }
    }

    fn validate_statement_refs(
        &self,
        table: &mut SymbolTable,
        type_params: &[TypeParam],
        statement: &Statement,
        locals: &mut ScopeStack,
        allow_self_type: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match statement {
            Statement::VarDecl {
                name,
                ty,
                value,
                mutable,
                constant,
                ..
            } => {
                if let Some(ty) = ty {
                    self.validate_type_ref(
                        table,
                        type_params,
                        ty,
                        statement.span(),
                        allow_self_type,
                        diagnostics,
                    );
                }
                self.validate_expr_refs(
                    table,
                    type_params,
                    value,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                if *constant || *mutable || !locals.is_mutable(name) {
                    self.define_local_symbol(
                        table,
                        name,
                        *mutable,
                        statement.span(),
                        locals,
                        diagnostics,
                    );
                }
            }
            Statement::Assignment { target, value, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    target,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
                self.validate_expr_refs(
                    table,
                    type_params,
                    value,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Statement::Expression { expr, .. } => {
                self.validate_expr_refs(
                    table,
                    type_params,
                    expr,
                    locals,
                    allow_self_type,
                    diagnostics,
                );
            }
            Statement::Block { stmts, .. } => {
                let block_scope_id = table.new_scope();
                let mut block_locals = ScopeStack::with_parent(block_scope_id, locals);
                for statement in stmts {
                    self.validate_statement_refs(
                        table,
                        type_params,
                        statement,
                        &mut block_locals,
                        allow_self_type,
                        diagnostics,
                    );
                }
            }
        }
    }

    fn is_known_value_name(&self, table: &SymbolTable, locals: &ScopeStack, name: &str) -> bool {
        table.lookup(Namespace::Value, name).is_some()
            || table.lookup(Namespace::Import, name).is_some()
            || locals.contains(name)
    }

    fn param_locals(
        &self,
        table: &mut SymbolTable,
        params: &[Param],
        scope_id: u32,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> ScopeStack {
        let mut locals = ScopeStack::new(scope_id);
        for param in params {
            self.define_local_symbol(
                table,
                &param.name,
                param.mutable,
                param.span,
                &mut locals,
                diagnostics,
            );
        }
        locals
    }

    fn define_local_symbol(
        &self,
        table: &mut SymbolTable,
        name: &str,
        mutable: bool,
        span: Span,
        locals: &mut ScopeStack,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match table.define_local(name, mutable, locals.current_scope_id, span) {
            Ok(_) => locals.insert(name.to_string(), mutable),
            Err(diagnostic) => diagnostics.push(*diagnostic),
        }
    }

    fn bind_pattern_locals(
        &self,
        table: &mut SymbolTable,
        pattern: &Pattern,
        locals: &mut ScopeStack,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match pattern {
            Pattern::Identifier { name, span } => {
                self.define_local_symbol(table, name, false, *span, locals, diagnostics);
            }
            Pattern::Struct { fields, .. } => {
                for (name, nested) in fields {
                    if let Some(nested) = nested {
                        self.bind_pattern_locals(table, nested, locals, diagnostics);
                    } else {
                        self.define_local_symbol(
                            table,
                            name,
                            false,
                            pattern.span(),
                            locals,
                            diagnostics,
                        );
                    }
                }
            }
            Pattern::Enum {
                payload: Some(payload),
                ..
            } => {
                self.bind_pattern_locals(table, payload, locals, diagnostics);
            }
            Pattern::Or { patterns, .. } => {
                for pattern in patterns {
                    self.bind_pattern_locals(table, pattern, locals, diagnostics);
                }
            }
            Pattern::Wildcard { .. }
            | Pattern::Literal { .. }
            | Pattern::Enum { payload: None, .. }
            | Pattern::Range { .. }
            | Pattern::BoolTrue { .. }
            | Pattern::BoolFalse { .. } => {}
        }
    }
}

fn resolver_return_type_name(return_type: &Option<AstType>) -> String {
    return_type
        .as_ref()
        .unwrap_or(&AstType::Void)
        .display_name()
}

fn resolver_param_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

fn resolver_param_type_names(params: &[Param]) -> Vec<String> {
    params.iter().map(|param| param.ty.display_name()).collect()
}

fn resolver_value_signature(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[TypeParam],
) -> ValueSignatureMetadata {
    ValueSignatureMetadata {
        parameter_names: resolver_param_names(params),
        parameter_types: params.iter().map(|param| param.ty.clone()).collect(),
        parameter_type_names: resolver_param_type_names(params),
        return_type: return_type.clone().unwrap_or(AstType::Void),
        return_type_name: resolver_return_type_name(return_type),
        type_parameter_count: type_params.len(),
        type_parameter_names: resolver_type_parameter_names(type_params),
        type_parameter_bounds: resolver_type_parameter_bounds(type_params),
    }
}

fn resolver_type_parameter_names(type_params: &[TypeParam]) -> Vec<String> {
    type_params
        .iter()
        .map(|type_param| type_param.name.clone())
        .collect()
}

fn resolver_type_parameter_bounds(type_params: &[TypeParam]) -> Vec<TypeParameterBoundMetadata> {
    type_params
        .iter()
        .filter_map(|type_param| {
            type_param_bound_display(type_param)
                .map(|constraint| (type_param.name.clone(), constraint))
        })
        .collect()
}

fn type_param_bound_display(type_param: &TypeParam) -> Option<String> {
    type_param.constraint.as_ref().map(|constraint| {
        if type_param.constraint_type_args.is_empty() {
            constraint.clone()
        } else {
            format!(
                "{}<{}>",
                constraint,
                type_param
                    .constraint_type_args
                    .iter()
                    .map(AstType::display_name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    })
}

fn behavior_ref_display(behavior: &str, type_args: &[AstType]) -> String {
    if type_args.is_empty() {
        behavior.to_string()
    } else {
        format!(
            "{}<{}>",
            behavior,
            type_args
                .iter()
                .map(AstType::display_name)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn resolver_field_types(fields: &[StructField]) -> Vec<(String, AstType, String)> {
    fields
        .iter()
        .map(|field| {
            (
                field.name.clone(),
                field.ty.clone(),
                field.ty.display_name(),
            )
        })
        .collect()
}

fn resolver_variant_names(variants: &[crate::ast::EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

fn resolver_behavior_method_signatures(methods: &[BehaviorMethod]) -> Vec<MethodSignatureMetadata> {
    methods
        .iter()
        .map(|method| {
            (
                method.name.clone(),
                resolver_param_type_names(&method.params),
                resolver_return_type_name(&method.return_type),
            )
        })
        .collect()
}

fn resolver_behavior_method_types(methods: &[BehaviorMethod]) -> Vec<BehaviorMethodTypeMetadata> {
    methods
        .iter()
        .map(|method| BehaviorMethodTypeMetadata {
            name: method.name.clone(),
            parameter_types: method.params.iter().map(|param| param.ty.clone()).collect(),
            return_type: method.return_type.clone().unwrap_or(AstType::Void),
        })
        .collect()
}
