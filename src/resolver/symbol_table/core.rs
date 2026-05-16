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
