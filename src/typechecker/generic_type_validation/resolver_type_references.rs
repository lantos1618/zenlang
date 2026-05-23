use super::*;

mod collected;

impl TypeChecker {
    pub(in crate::typechecker) fn validate_resolver_type_reference_task_list(
        &mut self,
        tasks: &[ResolverTypeReferenceValidationTask<'_>],
        symbols: Option<&SymbolTable>,
    ) {
        for task in tasks {
            match task {
                ResolverTypeReferenceValidationTask::Struct { name, fields, span } => {
                    self.validate_resolver_struct_type_references(symbols, name, fields, *span);
                }
                ResolverTypeReferenceValidationTask::Enum { name, span } => {
                    self.validate_resolver_enum_type_references(symbols, name, *span);
                }
                ResolverTypeReferenceValidationTask::Function { name, body, span } => {
                    self.validate_resolver_function_type_references(symbols, name, body, *span);
                }
                ResolverTypeReferenceValidationTask::Method {
                    type_name,
                    method_name,
                    body,
                    span,
                } => {
                    let ast_key = Self::method_key(type_name, method_name);
                    self.validate_resolver_method_type_references(
                        symbols, &ast_key, type_name, body, *span,
                    );
                }
                ResolverTypeReferenceValidationTask::Behavior {
                    name,
                    methods,
                    span,
                } => {
                    self.validate_resolver_behavior_type_references(symbols, name, methods, *span);
                }
                ResolverTypeReferenceValidationTask::ImplBlock { type_name, methods } => {
                    self.validate_resolver_impl_method_type_references(symbols, type_name, methods);
                }
                ResolverTypeReferenceValidationTask::TopLevelExpr { expr } => {
                    self.validate_generic_expr_type_references(expr, &HashSet::new());
                }
            }
        }
    }

    fn validate_resolver_enum_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        span: Span,
    ) {
        let Some((restored_name, scoped)) = self.resolver_scoped_symbol(
            symbols,
            Namespace::Type,
            name,
            span,
            Self::collected_type_type_param_scope,
        ) else {
            return;
        };
        self.validate_collected_enum_type_references(&restored_name, &scoped, span);
    }

    fn validate_resolver_struct_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        fields: &[StructField],
        span: Span,
    ) {
        let Some((restored_name, scoped)) = self.resolver_scoped_symbol(
            symbols,
            Namespace::Type,
            name,
            span,
            Self::collected_type_type_param_scope,
        ) else {
            return;
        };
        self.validate_collected_struct_type_references(&restored_name, &scoped, span);
        for field in fields {
            if let Some(default) = &field.default {
                self.validate_generic_expr_type_references(default, &scoped);
            }
        }
    }

    fn resolver_scoped_symbol(
        &self,
        symbols: Option<&SymbolTable>,
        namespace: Namespace,
        name: &str,
        span: Span,
        scoped_type_params: impl FnOnce(&Self, &str) -> Option<HashSet<String>>,
    ) -> Option<(String, HashSet<String>)> {
        let restored_name = Self::validation_symbol_name(symbols, namespace, name, span);
        let scoped = scoped_type_params(self, &restored_name)?;
        Some((restored_name, scoped))
    }

    fn validate_resolver_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        ast_key: &str,
        type_name: &str,
        body: &Expression,
        span: Span,
    ) {
        let restored_key = Self::validation_method_key(symbols, ast_key, type_name, span);
        if let Some(scoped) = self.collected_value_type_param_scope(&restored_key) {
            self.validate_collected_value_type_references(&restored_key, &scoped, span);
            self.validate_generic_expr_type_references(body, &scoped);
        }
    }

    fn validate_resolver_function_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        body: &Expression,
        span: Span,
    ) {
        let Some((restored_key, scoped)) = self.resolver_scoped_symbol(
            symbols,
            Namespace::Value,
            name,
            span,
            Self::collected_value_type_param_scope,
        ) else {
            return;
        };
        self.validate_collected_value_type_references(&restored_key, &scoped, span);
        self.validate_generic_expr_type_references(body, &scoped);
    }

    fn validate_resolver_behavior_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        name: &str,
        methods: &[BehaviorMethod],
        span: Span,
    ) {
        let Some((restored_name, scoped)) = self.resolver_scoped_symbol(
            symbols,
            Namespace::Behavior,
            name,
            span,
            Self::collected_behavior_type_param_scope,
        ) else {
            return;
        };
        self.validate_collected_behavior_type_references(&restored_name, &scoped, span);
        for method in methods {
            if let Some(default_body) = &method.default_body {
                self.validate_generic_expr_type_references(default_body, &scoped);
            }
        }
    }

    fn validate_resolver_impl_method_type_references(
        &mut self,
        symbols: Option<&SymbolTable>,
        type_name: &str,
        methods: &[Declaration],
    ) {
        for method in methods {
            if let Declaration::Function { name, body, .. } = method {
                let ast_key = Self::method_key(type_name, name);
                self.validate_resolver_method_type_references(
                    symbols,
                    &ast_key,
                    type_name,
                    body,
                    method.span(),
                );
            }
        }
    }
}
