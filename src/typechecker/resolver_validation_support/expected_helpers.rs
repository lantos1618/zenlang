fn expected_return_metadata(return_type: &Option<AstType>) -> ExpectedReturnMetadata {
    ExpectedReturnMetadata::new(return_type)
}

fn visibility_name(is_public: bool) -> &'static str {
    if is_public {
        "public"
    } else {
        "private"
    }
}

fn mutability_name(is_mutable: Option<bool>) -> &'static str {
    match is_mutable {
        Some(true) => "mutable",
        Some(false) => "immutable",
        None => "unknown",
    }
}

fn resolver_count_display(count: Option<usize>) -> String {
    count
        .map(|count| count.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn resolver_metadata_display(value: Option<&str>) -> &str {
    value.unwrap_or("unknown")
}

fn resolver_ast_type_metadata_display(value: Option<&AstType>) -> String {
    optional_ast_type_display(value, "unknown")
}

fn optional_ast_type_display(value: Option<&AstType>, missing: &str) -> String {
    value
        .map(AstType::display_name)
        .unwrap_or_else(|| missing.to_string())
}

fn expected_parameter_metadata(params: &[Param]) -> Vec<ExpectedParameter> {
    let mut expected = Vec::new();
    for param in params {
        expected.push(ExpectedParameter::new(&param.name, &param.ty));
    }
    expected
}

fn expected_value_signature_metadata(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
) -> ExpectedValueSignature {
    ExpectedValueSignature::new(params, return_type, type_params)
}

fn expected_value_symbol(
    params: &[Param],
    return_type: &Option<AstType>,
    type_params: &[ast::TypeParam],
    is_public: bool,
) -> ExpectedValueSymbol {
    ExpectedValueSymbol::new(params, return_type, type_params, is_public)
}

fn expected_type_parameter_metadata(type_params: &[ast::TypeParam]) -> Vec<ExpectedTypeParameter> {
    let mut expected = Vec::new();
    for type_param in type_params {
        expected.push(ExpectedTypeParameter::new(type_param));
    }
    expected
}

fn expected_behavior_symbol(
    type_params: &[ast::TypeParam],
    methods: &[ast::BehaviorMethod],
    is_public: bool,
) -> ExpectedBehaviorSymbol {
    ExpectedBehaviorSymbol::new(type_params, methods, is_public)
}

fn expected_struct_symbol(
    type_params: &[ast::TypeParam],
    fields: &[StructField],
    is_public: bool,
) -> ExpectedStructSymbol {
    ExpectedStructSymbol::new(type_params, fields, is_public)
}

fn expected_enum_symbol(
    type_params: &[ast::TypeParam],
    variants: &[EnumVariant],
    is_public: bool,
) -> ExpectedEnumSymbol {
    ExpectedEnumSymbol::new(type_params, variants, is_public)
}

fn expected_variant_symbol(
    owner_name: &str,
    is_public: bool,
    payload: &Option<AstType>,
) -> ExpectedVariantSymbol {
    ExpectedVariantSymbol::new(owner_name, is_public, payload)
}

fn expected_import_symbol(source: &str) -> ExpectedImportSymbol {
    ExpectedImportSymbol::new(source)
}

fn expected_module_symbol(name: &str) -> ExpectedModuleSymbol {
    ExpectedModuleSymbol::new(name)
}

fn expected_local_symbol(is_mutable: bool, scope_id: u32) -> ExpectedLocalSymbol {
    ExpectedLocalSymbol::new(is_mutable, scope_id)
}

fn format_type_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_type_parameter_bounds(bounds: Option<&[TypeParameterBoundMetadata]>) -> String {
    format_resolver_display_list(bounds, |(name, behavior)| format!("{name}: {behavior}"))
}

fn format_type_parameter_bound_refs(bounds: Option<&[TypeParameterBoundRefMetadata]>) -> String {
    format_resolver_display_list(bounds, |bound| {
        format!(
            "{}: {}",
            bound.type_parameter,
            behavior_ref_display(&bound.behavior, &bound.type_args)
        )
    })
}

fn format_parameter_type_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn format_ast_type_list(types: Option<&[AstType]>) -> String {
    format_resolver_display_list(types, AstType::display_name)
}

fn format_parameter_names(names: Option<&[String]>) -> String {
    format_resolver_string_list(names)
}

fn expected_field_metadata(fields: &[StructField]) -> Vec<ExpectedField> {
    let mut expected = Vec::new();
    for field in fields {
        expected.push(ExpectedField::new(&field.name, &field.ty));
    }
    expected
}

fn format_field_types(fields: Option<&[(String, AstType)]>) -> String {
    format_resolver_named_list(fields, AstType::display_name)
}

fn format_field_type_names(fields: Option<&[(String, String)]>) -> String {
    format_resolver_named_list(fields, String::clone)
}

fn expected_variant_name_metadata(variants: &[EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name.clone())
        .collect()
}

fn format_variant_names(variants: Option<&[String]>) -> String {
    format_resolver_string_list(variants)
}

fn format_resolver_string_list(values: Option<&[String]>) -> String {
    format_resolver_display_list(values, String::clone)
}

fn format_resolver_display_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| format!("({})", join_resolver_display_values(values, display_value)))
        .unwrap_or_else(|| "unknown".to_string())
}

fn join_resolver_strings(values: &[String]) -> String {
    values.join(", ")
}

fn join_resolver_display_values<T>(values: &[T], display_value: impl Fn(&T) -> String) -> String {
    let entries = values.iter().map(display_value).collect::<Vec<_>>();
    join_resolver_strings(&entries)
}

fn format_resolver_named_list<T>(
    values: Option<&[(String, T)]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    values
        .map(|values| {
            let entries = values
                .iter()
                .map(|(name, value)| format!("{name}: {}", display_value(value)))
                .collect::<Vec<_>>();
            format!("({})", join_resolver_strings(&entries))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn expected_behavior_method_metadata(
    methods: &[ast::BehaviorMethod],
) -> Vec<ExpectedBehaviorMethod> {
    let mut expected = Vec::new();
    for method in methods {
        expected.push(ExpectedBehaviorMethod::new(method));
    }
    expected
}

fn expected_behavior_edge(behavior: &str, type_args: &[AstType]) -> ExpectedBehaviorEdge {
    ExpectedBehaviorEdge::new(behavior, type_args)
}

fn push_expected_behavior_impl_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected.impls.push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_required_edge(
    expected: &mut ExpectedBehaviorAssociations,
    type_name: &str,
    behavior: &str,
    behavior_type_args: &[AstType],
) {
    expected
        .required
        .push(type_name, behavior, behavior_type_args);
}

fn push_expected_behavior_parent_edge(
    expected: &mut ExpectedBehaviorEdges,
    behavior: &str,
    parent: &str,
    parent_type_args: &[AstType],
) {
    expected.push(behavior, parent, parent_type_args);
}

fn collect_expected_resolver_impl_method_symbols(
    type_name: &str,
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
                method_signature_key(type_name, name),
                params,
                body,
                scope_cursor,
                expected,
            );
        }
    }
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

fn format_behavior_method_signatures(methods: Option<&[MethodSignatureMetadata]>) -> String {
    format_resolver_display_list(methods, |(name, params, return_type)| {
        format!("{name}({}) {return_type}", params.join(", "))
    })
}

fn format_behavior_method_types(methods: Option<&[BehaviorMethodTypeMetadata]>) -> String {
    format_resolver_display_list(methods, |method| {
        let params = method
            .parameter_types
            .iter()
            .enumerate()
            .map(|(index, ty)| {
                let name = method
                    .parameter_names
                    .get(index)
                    .map(String::as_str)
                    .unwrap_or("_");
                format!("{name}: {}", ty.display_name())
            })
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{}({}) {}",
            method.name,
            params,
            method.return_type.display_name()
        )
    })
}

fn format_behavior_ref_names(parents: Option<&[String]>) -> String {
    format_resolver_nonempty_joined_list(parents, String::clone)
}

fn format_behavior_refs(refs: Option<&[BehaviorRefMetadata]>) -> String {
    format_resolver_nonempty_joined_list(refs, |behavior| {
        behavior_ref_display(&behavior.name, &behavior.type_args)
    })
}

fn format_resolver_nonempty_joined_list<T>(
    values: Option<&[T]>,
    display_value: impl Fn(&T) -> String,
) -> String {
    match values {
        Some(values) if !values.is_empty() => join_resolver_display_values(values, display_value),
        _ => "none".to_string(),
    }
}

fn behavior_ref_names_match(actual: Option<&[String]>, expected: &[String]) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}

fn behavior_refs_match(
    actual: Option<&[BehaviorRefMetadata]>,
    expected: &[BehaviorRefMetadata],
) -> bool {
    match actual {
        Some(actual) => actual == expected,
        None => expected.is_empty(),
    }
}
