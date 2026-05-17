use super::*;

impl TypeChecker {
    pub(super) fn collect_resolver_declaration_semantic_validation_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationSemanticValidationTasks<'_> {
        let mut tasks = ResolverDeclarationSemanticValidationTasks::default();
        for decl in decls {
            Self::push_resolver_declaration_semantic_validation_tasks(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_resolver_declaration_semantic_validation_tasks<'a>(
        decl: &'a Declaration,
        tasks: &mut ResolverDeclarationSemanticValidationTasks<'a>,
    ) {
        let callable_handled =
            Self::push_resolver_callable_type_reference_task(decl, &mut tasks.type_references);
        let type_handled = if callable_handled {
            false
        } else {
            Self::push_resolver_type_semantic_validation_task(
                decl,
                &mut tasks.type_references,
                &mut tasks.struct_defaults,
            )
        };
        let behavior_handled = if callable_handled || type_handled {
            false
        } else {
            Self::push_resolver_behavior_type_reference_task(decl, &mut tasks.type_references)
        };
        let behavior_impl_handled = if callable_handled || type_handled || behavior_handled {
            false
        } else {
            Self::push_resolver_behavior_impl_type_reference_task(decl, &mut tasks.type_references)
        };
        if !callable_handled && !type_handled && !behavior_handled && !behavior_impl_handled {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks.type_references);
        }
        Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
        Self::push_behavior_requires_replay_task(decl, &mut tasks.behavior_associations.requires);
        Self::push_behavior_impl_block_declaration_task(
            decl,
            &mut tasks.behavior_associations.impls,
        );
    }

    fn push_resolver_type_semantic_validation_task<'a>(
        decl: &'a Declaration,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
        struct_default_tasks: &mut Vec<ResolverStructFieldDefaultValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                struct_default_tasks
                    .push(ResolverStructFieldDefaultValidationTask { name, span: *span });
                true
            }
            Declaration::Enum { name, span, .. } => {
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
                true
            }
            _ => false,
        }
    }

    fn push_resolver_callable_type_reference_task<'a>(
        decl: &'a Declaration,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Function {
                name, body, span, ..
            } => {
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
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
                true
            }
            _ => false,
        }
    }

    fn push_resolver_behavior_type_reference_task<'a>(
        decl: &'a Declaration,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::Behavior {
            name,
            methods,
            span,
            ..
        } = decl
        {
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

    fn push_resolver_behavior_impl_type_reference_task<'a>(
        decl: &'a Declaration,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name, methods, ..
        } = decl
        {
            type_reference_tasks
                .push(ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods });
            true
        } else {
            false
        }
    }
}
