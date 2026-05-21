use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn collect_ast_import_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstImportDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_import_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_ast_import_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstImportDeclarationTask<'a>>,
    ) {
        if let Declaration::Import {
            names, module_path, ..
        } = decl
        {
            tasks.push(AstImportDeclarationTask { names, module_path });
        }
    }

    pub(super) fn collect_ast_import_declarations_from_tasks(
        &mut self,
        tasks: &[AstImportDeclarationTask<'_>],
    ) {
        for task in tasks {
            for name in task.names {
                self.imports.insert(name.clone(), task.module_path.to_vec());
            }
        }
    }

    #[cfg(test)]
    pub(super) fn collect_impl_block_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<ImplBlockDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_impl_block_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_impl_block_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<ImplBlockDeclarationTask<'a>>,
    ) {
        if let Declaration::ImplBlock {
            type_name,
            behavior,
            behavior_type_args,
            methods,
            ..
        } = decl
        {
            tasks.push(ImplBlockDeclarationTask {
                type_name,
                behavior: behavior.as_deref(),
                behavior_type_args,
                methods,
            });
        }
    }

    pub(super) fn collect_impl_block_declarations_from_tasks(
        &mut self,
        tasks: &[ImplBlockDeclarationTask<'_>],
    ) {
        for task in tasks {
            if self.resolver_backed_collection {
                self.collect_resolver_backed_impl_block_templates(task.type_name, task.methods);
            } else {
                self.collect_ast_impl_block_declaration(
                    task.type_name,
                    task.behavior,
                    task.behavior_type_args,
                    task.methods,
                );
            }
        }
    }

    fn collect_ast_impl_block_declaration(
        &mut self,
        type_name: &str,
        behavior: Option<&str>,
        behavior_type_args: &[AstType],
        methods: &[Declaration],
    ) {
        for method in methods {
            self.collect_impl_method_signature(type_name, behavior, behavior_type_args, method);
        }
        if let Some(behavior) = behavior {
            self.collect_behavior_default_method_signatures(
                type_name,
                behavior,
                behavior_type_args,
                methods,
            );
        }
    }

    fn collect_resolver_backed_impl_block_templates(
        &mut self,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            self.collect_resolver_backed_impl_method_template(type_name, method);
        }
    }

    #[cfg(test)]
    pub(super) fn collect_ast_type_declaration_tasks(
        decls: &[Declaration],
    ) -> Vec<AstTypeDeclarationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_type_declaration_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_ast_type_declaration_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstTypeDeclarationTask<'a>>,
    ) {
        match decl {
            Declaration::Struct {
                name,
                type_params,
                fields,
                ..
            } => tasks.push(AstTypeDeclarationTask::Struct {
                name,
                type_params,
                fields,
            }),
            Declaration::Enum {
                name,
                type_params,
                variants,
                ..
            } => tasks.push(AstTypeDeclarationTask::Enum {
                name,
                type_params,
                variants,
            }),
            _ => {}
        }
    }

    pub(super) fn collect_ast_type_declarations_from_tasks(
        &mut self,
        tasks: &[AstTypeDeclarationTask<'_>],
    ) {
        for task in tasks {
            match task {
                AstTypeDeclarationTask::Struct {
                    name,
                    type_params,
                    fields,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.structs.insert(
                        (*name).to_string(),
                        struct_info_from_ast_fields((*name).to_string(), type_params, fields),
                    );
                }
                AstTypeDeclarationTask::Enum {
                    name,
                    type_params,
                    variants,
                } => {
                    self.validate_generic_bounds(type_params);
                    self.enums.insert(
                        (*name).to_string(),
                        enum_info_from_ast_variants((*name).to_string(), type_params, variants),
                    );
                }
            }
        }
    }
}
