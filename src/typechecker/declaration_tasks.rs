include!("declaration_tasks_callables.rs");

enum ResolverTypeDeclarationMetadataTask<'a> {
    Struct {
        name: &'a str,
        fields: &'a [StructField],
        span: Span,
    },
    Enum {
        name: &'a str,
        span: Span,
    },
}

struct ResolverStructFieldDefaultValidationTask<'a> {
    name: &'a str,
    span: Span,
}

enum AstTypeDeclarationTask<'a> {
    Struct {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        fields: &'a [StructField],
    },
    Enum {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        variants: &'a [EnumVariant],
    },
}

struct BehaviorDeclarationTask<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    methods: &'a [BehaviorMethod],
}

struct AstImportDeclarationTask<'a> {
    names: &'a [String],
    module_path: &'a [String],
}

#[derive(Default)]
struct AstDeclarationCollectionTasks<'a> {
    behaviors: Vec<BehaviorDeclarationTask<'a>>,
    types: Vec<AstTypeDeclarationTask<'a>>,
    callable: Vec<CallableDeclarationTask<'a>>,
    impl_blocks: Vec<ImplBlockDeclarationTask<'a>>,
    imports: Vec<AstImportDeclarationTask<'a>>,
    precollection_validations: AstPrecollectionValidationTasks<'a>,
}

#[derive(Default)]
struct DeclarationCollectionReplayTasks<'a> {
    ast: AstDeclarationCollectionTasks<'a>,
    resolver: ResolverDeclarationMetadataTasks<'a>,
    resolver_semantics: ResolverDeclarationSemanticValidationTasks<'a>,
}

struct AstStructFieldDefaultValidationTask<'a> {
    type_params: &'a [ast::TypeParam],
    fields: &'a [StructField],
}

enum AstTypeReferenceValidationTask<'a> {
    Struct {
        type_params: &'a [ast::TypeParam],
        fields: &'a [StructField],
    },
    Enum {
        type_params: &'a [ast::TypeParam],
        variants: &'a [EnumVariant],
    },
    Function {
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
    },
    Method {
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
    },
    Behavior {
        type_params: &'a [ast::TypeParam],
        methods: &'a [BehaviorMethod],
    },
    ImplBlock {
        methods: &'a [Declaration],
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

enum SelfTypeContextValidationTask<'a> {
    Struct {
        fields: &'a [StructField],
    },
    Enum {
        variants: &'a [EnumVariant],
    },
    Function {
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Method {
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Behavior {
        methods: &'a [BehaviorMethod],
    },
    ImplBlock {
        behavior_type_args: &'a [AstType],
        methods: &'a [Declaration],
        span: Span,
    },
    Requires {
        behavior_type_args: &'a [AstType],
        span: Span,
    },
    BehaviorExtends {
        parent_type_args: &'a [AstType],
        span: Span,
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

enum ResolverTypeReferenceValidationTask<'a> {
    Struct {
        name: &'a str,
        fields: &'a [StructField],
        span: Span,
    },
    Enum {
        name: &'a str,
        span: Span,
    },
    Function {
        name: &'a str,
        body: &'a Expression,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        body: &'a Expression,
        span: Span,
    },
    Behavior {
        name: &'a str,
        methods: &'a [BehaviorMethod],
        span: Span,
    },
    ImplBlock {
        type_name: &'a str,
        methods: &'a [Declaration],
    },
    TopLevelExpr {
        expr: &'a Expression,
    },
}

struct ResolverBehaviorDeclarationMetadataTask<'a> {
    name: &'a str,
    span: Span,
}

include!("declaration_tasks_behavior_associations.rs");

#[derive(Default)]
struct AstDeclarationValidationTasks<'a> {
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
    type_references: Vec<AstTypeReferenceValidationTask<'a>>,
    struct_field_defaults: Vec<AstStructFieldDefaultValidationTask<'a>>,
}

#[derive(Default)]
struct AstPrecollectionValidationTasks<'a> {
    self_type_contexts: Vec<SelfTypeContextValidationTask<'a>>,
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for AstDeclarationValidationTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        &self.behavior_associations
    }
}

#[derive(Default)]
struct ResolverDeclarationMetadataTasks<'a> {
    callable: Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    types: Vec<ResolverTypeDeclarationMetadataTask<'a>>,
    behaviors: Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
    type_references: Vec<ResolverTypeReferenceValidationTask<'a>>,
}

#[derive(Default)]
struct ResolverDeclarationSemanticValidationTasks<'a> {
    behavior_associations: BehaviorAssociationValidationTasks<'a>,
    type_references: Vec<ResolverTypeReferenceValidationTask<'a>>,
    struct_defaults: Vec<ResolverStructFieldDefaultValidationTask<'a>>,
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for ResolverDeclarationMetadataTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        &self.behavior_associations
    }
}

impl<'a> BehaviorAssociationValidationTaskSource<'a>
    for ResolverDeclarationSemanticValidationTasks<'a>
{
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        &self.behavior_associations
    }
}

struct ResolverTypeBehaviorRefreshTask {
    restored_name: String,
}

include!("resolver_validation_support.rs");

struct DefaultBehaviorMethod {
    name: String,
    params: Vec<Param>,
    return_type: Option<AstType>,
    body: Expression,
    span: Span,
}
