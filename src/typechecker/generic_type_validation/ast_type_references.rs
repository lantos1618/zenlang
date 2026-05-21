use super::*;

mod validation;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_ast_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeReferenceValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_reference_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_ast_type_reference_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeReferenceValidationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Struct {
                type_params,
                fields,
            }),
            Declaration::Enum {
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Enum {
                type_params,
                variants,
            }),
            Declaration::Function {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Function {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Method {
                type_params,
                params,
                return_type,
                body,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Method {
                type_params,
                params,
                return_type,
                body,
            }),
            Declaration::Behavior {
                type_params,
                methods,
                ..
            } => tasks.push(AstTypeReferenceValidationTask::Behavior {
                type_params,
                methods,
            }),
            Declaration::ImplBlock { methods, .. } => {
                tasks.push(AstTypeReferenceValidationTask::ImplBlock { methods });
            }
            Declaration::TopLevelExpr { expr, .. } => {
                tasks.push(AstTypeReferenceValidationTask::TopLevelExpr { expr });
            }
            _ => {}
        }
    }
}
