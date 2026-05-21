use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_resolver_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_resolver_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Function {
                    name,
                    body,
                    span: *span,
                });
            }
            Declaration::Method {
                type_name,
                method_name,
                body,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span: *span,
                });
            }
            Declaration::ImplBlock {
                type_name, methods, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
            }
            Declaration::Struct {
                name, fields, span, ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
            }
            Declaration::Enum { name, span, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
            }
            Declaration::Behavior {
                name,
                methods,
                span,
                ..
            } => {
                tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span: *span,
                });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(ResolverTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }
}
