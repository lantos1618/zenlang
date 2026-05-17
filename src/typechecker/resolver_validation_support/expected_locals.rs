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

fn expected_resolver_expr_locals(
    expr: &Expression,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match expr {
        Expression::BinaryOp { left, right, .. } => {
            expected_resolver_expr_locals(left, scope_cursor, locals, expected);
            expected_resolver_expr_locals(right, scope_cursor, locals, expected);
        }
        Expression::UnaryOp { operand, .. } => {
            expected_resolver_expr_locals(operand, scope_cursor, locals, expected);
        }
        Expression::FunctionCall { args, .. } => {
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MethodCall { receiver, args, .. } => {
            expected_resolver_expr_locals(receiver, scope_cursor, locals, expected);
            for arg in args {
                expected_resolver_expr_locals(arg, scope_cursor, locals, expected);
            }
        }
        Expression::MemberAccess { object, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
        }
        Expression::IndexAccess { object, index, .. } => {
            expected_resolver_expr_locals(object, scope_cursor, locals, expected);
            expected_resolver_expr_locals(index, scope_cursor, locals, expected);
        }
        Expression::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::EnumVariant { payload, .. } => {
            if let Some(payload) = payload {
                expected_resolver_expr_locals(payload, scope_cursor, locals, expected);
            }
        }
        Expression::ArrayLiteral { elements, .. } => {
            for element in elements {
                expected_resolver_expr_locals(element, scope_cursor, locals, expected);
            }
        }
        Expression::Match {
            scrutinee, arms, ..
        } => {
            expected_resolver_expr_locals(scrutinee, scope_cursor, locals, expected);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    expected_resolver_pattern_expr_locals(
                        &arm.pattern,
                        guard,
                        scope_cursor,
                        locals,
                        expected,
                    );
                }
                expected_resolver_pattern_expr_locals(
                    &arm.pattern,
                    &arm.body,
                    scope_cursor,
                    locals,
                    expected,
                );
            }
        }
        Expression::WhileLoop {
            condition, body, ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::Loop { body, .. } => {
            expected_resolver_child_expr_locals(body, scope_cursor, locals, expected);
        }
        Expression::If {
            condition,
            then_body,
            else_body,
            ..
        } => {
            expected_resolver_expr_locals(condition, scope_cursor, locals, expected);
            expected_resolver_child_expr_locals(then_body, scope_cursor, locals, expected);
            if let Some(else_body) = else_body {
                expected_resolver_child_expr_locals(else_body, scope_cursor, locals, expected);
            }
        }
        Expression::Block {
            statements, expr, ..
        } => {
            expected_resolver_block_locals(
                statements,
                expr.as_deref(),
                scope_cursor,
                locals,
                expected,
            );
        }
        Expression::Return { value, .. } => {
            if let Some(value) = value {
                expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            }
        }
        Expression::Closure { params, body, .. } => {
            expected_resolver_closure_locals(params, body, scope_cursor, locals, expected);
        }
        Expression::Cast { expr, .. } | Expression::Defer { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        Expression::StringInterpolation { parts, .. } => {
            for part in parts {
                if let ast::StringPart::Expr(expr) = part {
                    expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
                }
            }
        }
        Expression::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        Expression::Identifier { .. }
        | Expression::IntLiteral { .. }
        | Expression::FloatLiteral { .. }
        | Expression::StringLiteral { .. }
        | Expression::BoolLiteral { .. }
        | Expression::CharLiteral { .. }
        | Expression::Break { .. }
        | Expression::Continue { .. }
        | Expression::LoopControl { .. }
        | Expression::Error { .. } => {}
    }
}

fn expected_resolver_statement_locals(
    statement: &ast::Statement,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match statement {
        ast::Statement::VarDecl {
            name,
            value,
            mutable,
            constant,
            ..
        } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
            if resolver_var_decl_binds_local(name, *mutable, *constant, locals) {
                expected_resolver_var_decl_local(name, *mutable, locals, expected);
            }
        }
        ast::Statement::Assignment { target, value, .. } => {
            expected_resolver_expr_locals(target, scope_cursor, locals, expected);
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Statement::Expression { expr, .. } => {
            expected_resolver_expr_locals(expr, scope_cursor, locals, expected);
        }
        ast::Statement::Block { stmts, .. } => {
            expected_resolver_block_locals(stmts, None, scope_cursor, locals, expected);
        }
    }
}

fn expected_resolver_var_decl_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, mutable, locals, expected);
}

fn resolver_var_decl_binds_local(
    name: &str,
    mutable: bool,
    constant: bool,
    locals: &ResolverLocalScope,
) -> bool {
    constant || mutable || !locals.is_mutable(name)
}

fn expected_resolver_pattern_locals(
    pattern: &ast::Pattern,
    scope_cursor: &mut ResolverScopeCursor,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    match pattern {
        ast::Pattern::Identifier { name, .. } => {
            expected_resolver_pattern_binding(name, locals, expected);
        }
        ast::Pattern::Struct { fields, .. } => {
            for (name, nested) in fields {
                if let Some(nested) = nested {
                    expected_resolver_pattern_locals(nested, scope_cursor, locals, expected);
                } else {
                    expected_resolver_pattern_binding(name, locals, expected);
                }
            }
        }
        ast::Pattern::Enum {
            payload: Some(payload),
            ..
        } => {
            expected_resolver_pattern_locals(payload, scope_cursor, locals, expected);
        }
        ast::Pattern::Or { patterns, .. } => {
            for pattern in patterns {
                expected_resolver_pattern_locals(pattern, scope_cursor, locals, expected);
            }
        }
        ast::Pattern::Literal { value, .. } => {
            expected_resolver_expr_locals(value, scope_cursor, locals, expected);
        }
        ast::Pattern::Range { start, end, .. } => {
            expected_resolver_expr_locals(start, scope_cursor, locals, expected);
            expected_resolver_expr_locals(end, scope_cursor, locals, expected);
        }
        ast::Pattern::Wildcard { .. }
        | ast::Pattern::Enum { payload: None, .. }
        | ast::Pattern::BoolTrue { .. }
        | ast::Pattern::BoolFalse { .. } => {}
    }
}

fn expected_resolver_pattern_binding(
    name: &str,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected_resolver_local(name, false, locals, expected);
}

fn expected_resolver_local(
    name: &str,
    mutable: bool,
    locals: &mut ResolverLocalScope,
    expected: &mut HashSet<(String, u32)>,
) {
    expected.insert((name.to_string(), locals.current_scope_id));
    locals.insert(name.to_string(), mutable);
}
