#[derive(Default)]
struct BehaviorAssociationValidationTasks<'a> {
    extends: Vec<BehaviorExtendsValidationTask<'a>>,
    impls: Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    requires: Vec<BehaviorRequiresValidationTask<'a>>,
}

trait BehaviorAssociationValidationTaskSource<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a>;
}

impl<'a> BehaviorAssociationValidationTaskSource<'a> for BehaviorAssociationValidationTasks<'a> {
    fn behavior_association_tasks(&self) -> &BehaviorAssociationValidationTasks<'a> {
        self
    }
}

struct ResolverBehaviorImplBlockDeclarationTask<'a> {
    ast_type_name: &'a str,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
    span: Span,
}

struct ResolverBehaviorImplBlockTask<'a> {
    ast_type_name: &'a str,
    restored_type_name: String,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
}

struct ImplBlockDeclarationTask<'a> {
    type_name: &'a str,
    behavior: Option<&'a str>,
    behavior_type_args: &'a [AstType],
    methods: &'a [Declaration],
}

struct BehaviorRequiresValidationTask<'a> {
    type_name: &'a str,
    behavior: &'a str,
    behavior_type_args: &'a [AstType],
    span: Span,
}

struct EffectiveBehaviorImplMethod<'a> {
    declaration: &'a Declaration,
    method_name: String,
}

struct BehaviorExtendsValidationTask<'a> {
    behavior: &'a str,
    parent: &'a str,
    parent_type_args: &'a [AstType],
    span: Span,
}
