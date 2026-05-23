use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(in crate::typechecker) fn collect_resolver_behavior_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorDeclarationMetadataTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_resolver_behavior_metadata_task(decl, &mut tasks);
        }
        tasks
    }

    pub(in crate::typechecker) fn push_resolver_behavior_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior {
            name,
            methods,
            span,
            ..
        } = decl
        {
            behavior_tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
            type_reference_tasks.push(ResolverTypeReferenceValidationTask::Behavior {
                name,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    fn push_resolver_behavior_metadata_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorDeclarationMetadataTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior { name, span, .. } = decl {
            tasks.push(ResolverBehaviorDeclarationMetadataTask {
                name: name.as_str(),
                span: *span,
            });
            true
        } else {
            false
        }
    }

    pub(in crate::typechecker) fn push_resolver_behavior_impl_replay_tasks<'a>(
        decl: &'a Declaration,
        behavior_impl_tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        let handled = Self::push_behavior_impl_block_declaration_task(decl, behavior_impl_tasks);
        if handled {
            let Declaration::ImplBlock {
                type_name, methods, ..
            } = decl
            else {
                return false;
            };
            type_reference_tasks
                .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
        }
        handled
    }

    #[cfg(test)]
    pub(in crate::typechecker) fn collect_resolver_behavior_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_behavior_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

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
