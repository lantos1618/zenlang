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
