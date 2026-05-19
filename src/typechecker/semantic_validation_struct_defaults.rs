use super::*;

impl TypeChecker {
    #[cfg(test)]
    pub(super) fn validate_struct_field_defaults(
        &mut self,
        decls: &[Declaration],
        symbols: Option<&SymbolTable>,
    ) {
        if self.resolver_backed_collection {
            let tasks = Self::collect_resolver_declaration_semantic_validation_tasks(decls);
            self.validate_resolver_struct_field_default_task_list(&tasks.struct_defaults, symbols);
            return;
        }

        let tasks = Self::collect_ast_struct_field_default_validation_tasks(decls);
        self.validate_ast_struct_field_default_tasks(&tasks);
    }

    #[cfg(test)]
    pub(super) fn collect_ast_struct_field_default_validation_tasks(
        decls: &[Declaration],
    ) -> Vec<AstStructFieldDefaultValidationTask<'_>> {
        let mut tasks = Vec::new();
        for decl in decls {
            Self::push_ast_struct_field_default_validation_task(decl, &mut tasks);
        }
        tasks
    }

    pub(super) fn push_ast_struct_field_default_validation_task<'a>(
        decl: &'a Declaration,
        tasks: &mut Vec<AstStructFieldDefaultValidationTask<'a>>,
    ) {
        if let Declaration::Struct {
            type_params,
            fields,
            ..
        } = decl
        {
            tasks.push(AstStructFieldDefaultValidationTask {
                type_params,
                fields,
            });
        }
    }

    pub(super) fn validate_ast_struct_field_default_tasks(
        &mut self,
        tasks: &[AstStructFieldDefaultValidationTask<'_>],
    ) {
        for task in tasks {
            self.validate_ast_struct_field_defaults(!task.type_params.is_empty(), task.fields);
        }
    }

    pub(super) fn validate_resolver_struct_field_defaults(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let restored_name = Self::validation_symbol_name(symbols, Namespace::Type, name, span);
        let Some(info) = self.structs.get(&restored_name).cloned() else {
            return;
        };
        if !info.type_params.is_empty() {
            return;
        }
        for (field_name, expected) in &info.fields {
            if let Some(default) = info.field_defaults.get(field_name) {
                self.validate_struct_field_default(field_name, expected, default);
            }
        }
    }

    pub(super) fn validate_resolver_struct_field_default_task_list(
        &mut self,
        tasks: &[ResolverStructFieldDefaultValidationTask<'_>],
        symbols: Option<&SymbolTable>,
    ) {
        for task in tasks {
            self.validate_resolver_struct_field_defaults(symbols, task.name, task.span);
        }
    }

    pub(super) fn validate_ast_struct_field_defaults(
        &mut self,
        has_type_params: bool,
        fields: &[StructField],
    ) {
        if has_type_params {
            return;
        }
        for field in fields {
            if let Some(default) = &field.default {
                self.validate_struct_field_default(&field.name, &field.ty, default);
            }
        }
    }

    pub(super) fn validate_struct_field_default(
        &mut self,
        field_name: &str,
        expected: &AstType,
        default: &Expression,
    ) {
        let expected = self.resolve_type(expected);
        self.push_scope();
        let actual = self.check_expr(default);
        self.pop_scope();

        let Ok(actual) = actual else {
            self.diagnostics.push(actual.expect_err("checked error"));
            return;
        };
        let actual_ty = if (expected.is_integer()
            && matches!(actual.kind, TypedExprKind::IntLiteral(_)))
            || (expected.is_float() && matches!(actual.kind, TypedExprKind::FloatLiteral(_)))
        {
            expected.clone()
        } else {
            actual.ty.clone()
        };

        if !self.types_compatible(&expected, &actual_ty) {
            self.diagnostics.push(Diagnostic::error(
                "E3073",
                format!(
                    "field `{}` default expects `{}`, found `{}`",
                    field_name,
                    expected.display_name(),
                    actual.ty.display_name()
                ),
                actual.span,
            ));
        }
    }
}
