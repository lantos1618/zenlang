include!("declaration_tasks_callables.rs");
include!("declaration_tasks_ast.rs");

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

#[derive(Default)]
struct DeclarationCollectionReplayTasks<'a> {
    ast: AstDeclarationCollectionTasks<'a>,
    resolver: ResolverDeclarationMetadataTasks<'a>,
    resolver_semantics: ResolverDeclarationSemanticValidationTasks<'a>,
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
