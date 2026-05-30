#[derive(Default)]
pub struct TypeChecker {
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    functions: HashMap<String, FuncInfo>,
    methods: HashMap<String, FuncInfo>, // key: "TypeName.method_name"
    behaviors: HashMap<String, BehaviorInfo>,
    behavior_extends: HashMap<String, Vec<BehaviorParentRef>>,
    behavior_extends_spans: HashMap<String, Span>,
    behavior_impls: HashSet<(String, String)>,
    behavior_refs_by_key: HashMap<String, BehaviorParentRef>,
    generic_behavior_impls: Vec<GenericBehaviorImplTemplate>,
    generic_functions: HashMap<String, GenericFunctionTemplate>,
    generic_methods: HashMap<String, GenericFunctionTemplate>,
    specialized_functions: Vec<TypedFunction>,
    specializations_seen: HashMap<String, String>,
    specialization_name_owners: HashMap<String, String>,
    specialized_types: Vec<TypedTypeDef>,
    specialized_types_seen: HashMap<String, String>,
    specialized_type_name_owners: HashMap<String, String>,
    specialized_type_generic_names: HashMap<String, String>,
    specialized_type_args: HashMap<String, Vec<AstType>>,
    type_substitutions: Vec<HashMap<String, Type>>,
    imports: HashSet<String>,
    /// Opaque `@extern` C type names — valid types (used behind pointers in FFI
    /// signatures) with no Zen definition.
    extern_types: HashSet<String>,
    scopes: Vec<HashMap<String, VarInfo>>,
    diagnostics: Vec<Diagnostic>,
    current_return_type: Option<Type>,
    current_self_type: Option<Type>,
    pending_defers: Vec<TypedExpression>,
}

impl TypeChecker {
    pub fn new() -> Self {
        Self::default()
    }
}
