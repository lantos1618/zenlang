use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_resolver_callable_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverCallableDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_callable_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_resolver_callable_replay_tasks<'a>(
        decl: &'a Declaration,
        callable_tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                callable_tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
                true
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                callable_tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    fn push_resolver_callable_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverCallableDeclarationMetadataTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function { name, span, .. } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Function { name, span: *span });
                true
            }
            Declaration::Method {
                type_name,
                method_name,
                span,
                ..
            } => {
                tasks.push(ResolverCallableDeclarationMetadataTask::Method {
                    type_name,
                    method_name,
                    span: *span,
                });
                true
            }
            Declaration::ImplBlock {
                type_name,
                behavior: None,
                methods,
                ..
            } => {
                tasks
                    .push(ResolverCallableDeclarationMetadataTask::TypeImpl { type_name, methods });
                true
            }
            _ => false,
        }
    }
}
