use super::*;

impl TypeChecker {
    pub(super) fn collect_declaration_collection_replay_tasks(
        decls: &[Declaration],
    ) -> DeclarationCollectionReplayTasks<'_> {
        let mut tasks = DeclarationCollectionReplayTasks::default();
        for decl in decls {
            Self::push_ast_declaration_collection_tasks(decl, &mut tasks.ast);
            Self::push_resolver_declaration_metadata_tasks(decl, &mut tasks.resolver);
        }
        tasks
    }

    pub(super) fn collect_resolver_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> ResolverDeclarationMetadataTasks<'_> {
        let mut tasks = ResolverDeclarationMetadataTasks::default();
        for decl in decls {
            Self::push_resolver_declaration_metadata_tasks(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_ast_declaration_collection_tasks<'a>(
        decl: &'a Declaration,
        tasks: &mut AstDeclarationCollectionTasks<'a>,
    ) {
        Self::push_behavior_declaration_task(decl, &mut tasks.behaviors);
        Self::push_ast_type_declaration_task(decl, &mut tasks.types);
        Self::push_callable_declaration_task(decl, &mut tasks.callable);
        Self::push_impl_block_declaration_task(decl, &mut tasks.impl_blocks);
        Self::push_ast_import_declaration_task(decl, &mut tasks.imports);
        Self::push_self_type_context_validation_task(
            decl,
            &mut tasks.precollection_validations.self_type_contexts,
        );
        Self::push_behavior_extends_replay_task(
            decl,
            &mut tasks
                .precollection_validations
                .behavior_associations
                .extends,
        );
    }

    fn push_resolver_declaration_metadata_tasks<'a>(
        decl: &'a Declaration,
        tasks: &mut ResolverDeclarationMetadataTasks<'a>,
    ) {
        let callable_handled = Self::push_resolver_callable_replay_tasks(
            decl,
            &mut tasks.callable,
            &mut tasks.type_references,
        );
        let type_handled = if callable_handled {
            false
        } else {
            Self::push_resolver_type_replay_tasks(
                decl,
                &mut tasks.types,
                &mut tasks.type_references,
            )
        };
        let behavior_handled = if callable_handled || type_handled {
            false
        } else {
            Self::push_resolver_behavior_replay_tasks(
                decl,
                &mut tasks.behaviors,
                &mut tasks.type_references,
            )
        };
        let behavior_impl_handled = if callable_handled || type_handled || behavior_handled {
            false
        } else {
            Self::push_resolver_behavior_impl_replay_tasks(
                decl,
                &mut tasks.behavior_associations.impls,
                &mut tasks.type_references,
            )
        };
        if !callable_handled && !type_handled && !behavior_handled && !behavior_impl_handled {
            Self::push_resolver_type_reference_validation_task(decl, &mut tasks.type_references);
        }
        Self::push_behavior_extends_replay_task(decl, &mut tasks.behavior_associations.extends);
        Self::push_behavior_requires_replay_task(decl, &mut tasks.behavior_associations.requires);
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_type_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeDeclarationMetadataTask<'_>> {
        Self::collect_resolver_declaration_metadata_tasks(decls).types
    }

    pub(super) fn push_resolver_type_replay_tasks<'a>(
        decl: &'a Declaration,
        type_tasks: &mut Vec<ResolverTypeDeclarationMetadataTask<'a>>,
        type_reference_tasks: &mut Vec<ResolverTypeReferenceValidationTask<'a>>,
    ) -> bool {
        match decl {
            Declaration::Struct {
                name, fields, span, ..
            } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                type_reference_tasks.push(ResolverTypeReferenceValidationTask::Struct {
                    name,
                    fields,
                    span: *span,
                });
                true
            }
            Declaration::Enum { name, span, .. } => {
                type_tasks.push(ResolverTypeDeclarationMetadataTask::Enum { name, span: *span });
                type_reference_tasks
                    .push(ResolverTypeReferenceValidationTask::Enum { name, span: *span });
                true
            }
            _ => false,
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_behavior_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorDeclarationMetadataTask<'_>> {
        Self::collect_resolver_declaration_metadata_tasks(decls).behaviors
    }

    pub(super) fn push_resolver_behavior_replay_tasks<'a>(
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

    pub(super) fn push_resolver_behavior_impl_replay_tasks<'a>(
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
    pub(super) fn collect_resolver_behavior_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverBehaviorImplBlockDeclarationTask<'_>> {
        Self::collect_resolver_declaration_metadata_tasks(decls)
            .behavior_associations
            .impls
    }

    pub(super) fn push_behavior_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        {
            tasks.push(ResolverBehaviorImplBlockDeclarationTask {
                ast_type_name: type_name,
                behavior,
                behavior_type_args,
                methods,
                span: *span,
            });
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub(super) fn collect_resolver_callable_declaration_metadata_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverCallableDeclarationMetadataTask<'_>> {
        Self::collect_resolver_declaration_metadata_tasks(decls).callable
    }

    pub(super) fn push_resolver_callable_replay_tasks<'a>(
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
    pub(super) fn collect_resolver_type_reference_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<ResolverTypeReferenceValidationTask<'_>> {
        Self::collect_resolver_declaration_metadata_tasks(decls).type_references
    }

    fn push_resolver_type_reference_validation_task<'a>(
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
