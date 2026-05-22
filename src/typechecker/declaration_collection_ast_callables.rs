use super::*;

mod resolver_templates;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_callable_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<CallableDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_callable_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_callable_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<CallableDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Function {
                name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(CallableDeclarationTask::Method {
                type_name,
                method_name,
                type_params,
                params,
                return_type,
                body,
                span: *span,
            }),
            _ => {}
        }
    }

    pub(super) fn collect_callable_declarations_from_tasks(
        &mut self,
        tasks: &[CallableDeclarationTask<'_>],
    ) {
        for task in tasks {
            match task {
                CallableDeclarationTask::Function {
                    name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_function_template(
                            name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        self.collect_ast_function_declaration(
                            name,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
                CallableDeclarationTask::Method {
                    type_name,
                    method_name,
                    type_params,
                    params,
                    return_type,
                    body,
                    span,
                } => {
                    if self.resolver_backed_collection {
                        self.collect_resolver_backed_method_template(
                            type_name,
                            method_name,
                            type_params,
                            params,
                            body,
                            *span,
                        );
                    } else {
                        let key = Self::method_key(type_name, method_name);
                        self.collect_ast_method_declaration(
                            &key,
                            type_params,
                            params,
                            return_type,
                            body,
                            *span,
                        );
                    }
                }
            }
        }
    }

    fn collect_ast_function_declaration(
        &mut self,
        name: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.functions.insert(
            name.to_string(),
            func_info_from_ast_signature(name.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_functions.insert(name.to_string(), template);
        }
    }

    fn collect_ast_method_declaration(
        &mut self,
        key: &str,
        type_params: &[ast::TypeParam],
        params: &[Param],
        return_type: &Option<AstType>,
        body: &Expression,
        span: Span,
    ) {
        self.validate_generic_bounds(type_params);
        self.methods.insert(
            key.to_string(),
            func_info_from_ast_signature(key.to_string(), type_params, params, return_type),
        );
        if let Some(template) =
            generic_template_from_type_params(type_params, params, return_type, body, span)
        {
            self.generic_methods.insert(key.to_string(), template);
        }
    }
}
