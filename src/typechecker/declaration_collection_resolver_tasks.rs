use super::*;

mod callables;
mod replay_kinds;

impl TypeChecker {
    pub(super) fn collect_declaration_collection_replay_tasks(
        decls: &[Declaration],
    ) -> DeclarationCollectionReplayTasks<'_> {
        let mut tasks = DeclarationCollectionReplayTasks::default();
        for decl in decls {
            Self::push_ast_declaration_collection_tasks(decl, &mut tasks.ast);
            Self::push_resolver_declaration_metadata_tasks(decl, &mut tasks.resolver);
            Self::push_resolver_declaration_semantic_validation_tasks(
                decl,
                &mut tasks.resolver_semantics,
            );
        }
        tasks
    }

    #[cfg(test)]
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

    pub(super) fn push_behavior_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ResolverBehaviorImplBlockDeclarationTask<'a>>,
    ) -> bool {
        if let Declaration::ImplBlock {
            type_name,
            type_args,
            behavior: Some(behavior),
            behavior_type_args,
            methods,
            span,
            ..
        } = decl
        {
            tasks.push(ResolverBehaviorImplBlockDeclarationTask {
                ast_type_name: type_name,
                type_args,
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
}
