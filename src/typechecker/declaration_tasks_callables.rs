struct ResolverCallableSignature<'a> {
    parameter_names: &'a [String],
    parameter_types: &'a [AstType],
    return_type: &'a AstType,
}

struct ResolverTypeParameterMetadata<'a> {
    names: &'a [String],
    bound_refs: &'a [TypeParameterBoundRefMetadata],
}

enum ResolverCallableDeclarationMetadataTask<'a> {
    Function {
        name: &'a str,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        span: Span,
    },
    TypeImpl {
        type_name: &'a str,
        methods: &'a [Declaration],
    },
}

enum CallableDeclarationTask<'a> {
    Function {
        name: &'a str,
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
    Method {
        type_name: &'a str,
        method_name: &'a str,
        type_params: &'a [ast::TypeParam],
        params: &'a [Param],
        return_type: &'a Option<AstType>,
        body: &'a Expression,
        span: Span,
    },
}
