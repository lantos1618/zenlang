struct ImportedMethodSignature<'a> {
    name: &'a str,
    type_params: &'a [ast::TypeParam],
    params: &'a [Param],
    return_type: &'a Option<AstType>,
    body: &'a Expression,
    span: Span,
}

impl<'a> ImportedMethodSignature<'a> {
    fn from_function_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Function {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn from_method_declaration(name: &'a str, decl: &'a Declaration) -> Option<Self> {
        let Declaration::Method {
            type_params,
            params,
            return_type,
            body,
            span,
            ..
        } = decl
        else {
            return None;
        };

        Some(Self {
            name,
            type_params,
            params,
            return_type,
            body,
            span: *span,
        })
    }

    fn func_info(&self, key: String) -> FuncInfo {
        func_info_from_ast_signature(key, self.type_params, self.params, self.return_type)
    }

    fn generic_template(&self) -> Option<GenericFunctionTemplate> {
        generic_template_from_type_params(
            self.type_params,
            self.params,
            self.return_type,
            self.body,
            self.span,
        )
    }
}
