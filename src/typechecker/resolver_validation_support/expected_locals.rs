fn collect_expected_resolver_impl_method_symbols(
    type_name: &str,
    behavior: Option<&str>,
    behavior_type_args: &[AstType],
    methods: &[Declaration],
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    for method in methods {
        if let Declaration::Function {
            name, params, body, ..
        } = method
        {
            push_expected_resolver_callable_symbol(
                expected_resolver_impl_method_key(type_name, name, behavior, behavior_type_args),
                params,
                body,
                scope_cursor,
                expected,
            );
        }
    }
}

fn expected_resolver_impl_method_key(
    type_name: &str,
    method_name: &str,
    behavior: Option<&str>,
    behavior_type_args: &[AstType],
) -> String {
    behavior_impl_method_signature_key(type_name, method_name, behavior, behavior_type_args)
}

fn push_resolver_validation_association_source<'a>(
    namespace: Namespace,
    name: &'a str,
    span: Span,
    symbols: &'a SymbolTable,
    expected: &mut ResolverExpectedSymbolSets,
    sources: &mut Vec<ResolverValidationBehaviorAssociationSource<'a>>,
) {
    expected.declarations.insert((namespace, name.to_string()));
    if let Some(symbol) = symbols.lookup(namespace, name) {
        sources.push(ResolverValidationBehaviorAssociationSource { name, symbol, span });
    }
}

fn push_expected_resolver_import_symbols(
    names: &[String],
    module_path: &[String],
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected.validate_imports = true;
    expected
        .declarations
        .insert((Namespace::Module, module_path.join(".")));
    for name in names {
        expected
            .declarations
            .insert((Namespace::Import, name.clone()));
    }
}

fn push_expected_resolver_variant_symbols(
    variants: &[EnumVariant],
    expected: &mut ResolverExpectedSymbolSets,
) {
    for variant in variants {
        expected
            .declarations
            .insert((Namespace::Variant, variant.name.clone()));
    }
}

fn push_expected_resolver_scoped_expr_symbols(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected_resolver_scoped_expr_locals(expr, scope_cursor, &mut expected.locals);
}

fn push_expected_resolver_callable_symbol(
    name: String,
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut ResolverExpectedSymbolSets,
) {
    expected.declarations.insert((Namespace::Value, name));
    expected_resolver_callable_locals(params, body, scope_cursor, &mut expected.locals);
}

fn expected_resolver_callable_locals(
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut locals = scope_cursor.new_scope();
    expected_resolver_parameter_locals(params, &mut locals, expected);
    expected_resolver_expr_locals(body, scope_cursor, &mut locals, expected);
}

fn expected_resolver_scoped_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut locals = scope_cursor.new_scope();
    expected_resolver_expr_locals(expr, scope_cursor, &mut locals, expected);
}

fn expected_resolver_child_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut child_locals = scope_cursor.child_scope(locals);
    expected_resolver_expr_locals(expr, scope_cursor, &mut child_locals, expected);
}

fn expected_resolver_pattern_expr_locals(
    pattern: &ast::Pattern,
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut pattern_locals = scope_cursor.child_scope(locals);
    expected_resolver_pattern_locals(pattern, scope_cursor, &mut pattern_locals, expected);
    expected_resolver_expr_locals(expr, scope_cursor, &mut pattern_locals, expected);
}

fn expected_resolver_block_locals(
    statements: &[ast::Statement],
    expr: Option<&Expression>,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut block_locals = scope_cursor.child_scope(locals);
    for statement in statements {
        expected_resolver_statement_locals(statement, scope_cursor, &mut block_locals, expected);
    }
    if let Some(expr) = expr {
        expected_resolver_expr_locals(expr, scope_cursor, &mut block_locals, expected);
    }
}

fn expected_resolver_closure_locals(
    params: &[Param],
    body: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    let mut closure_locals = scope_cursor.child_scope(locals);
    expected_resolver_parameter_locals(params, &mut closure_locals, expected);
    expected_resolver_expr_locals(body, scope_cursor, &mut closure_locals, expected);
}

fn expected_resolver_parameter_locals(
    params: &[Param],
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    for param in params {
        expected_resolver_local(&param.name, param.mutable, locals, expected);
    }
}
