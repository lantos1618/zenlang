use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_self_type_context_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<SelfTypeContextValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_self_type_context_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_self_type_context_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<SelfTypeContextValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct { fields, .. } => {
                tasks.push(SelfTypeContextValidationTask::Struct { fields });
            }
            Declaration::Enum { variants, .. } => {
                tasks.push(SelfTypeContextValidationTask::Enum { variants });
            }
            Declaration::Function {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Function {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Method {
                params,
                return_type,
                body,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Method {
                params,
                return_type,
                body,
                span: *span,
            }),
            Declaration::Behavior { methods, .. } => {
                tasks.push(SelfTypeContextValidationTask::Behavior { methods });
            }
            Declaration::ImplBlock {
                behavior_type_args,
                methods,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::ImplBlock {
                behavior_type_args,
                methods,
                span: *span,
            }),
            Declaration::Requires {
                behavior_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::Requires {
                behavior_type_args,
                span: *span,
            }),
            Declaration::BehaviorExtends {
                parent_type_args,
                span,
                ..
            } => tasks.push(SelfTypeContextValidationTask::BehaviorExtends {
                parent_type_args,
                span: *span,
            }),
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(SelfTypeContextValidationTask::TopLevelExpr { expr });
            }
            Declaration::Derive { .. } | Declaration::Import { .. } | Declaration::Error { .. } => {
            }
        }
    }
}
