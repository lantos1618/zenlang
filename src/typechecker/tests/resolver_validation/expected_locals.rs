use super::*;

#[test]
fn expected_resolver_impl_method_symbols_collect_value_symbols_and_locals() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}
"#,
    );
    let Declaration::ImplBlock {
        type_name, methods, ..
    } = &program.declarations[1]
    else {
        panic!("expected impl block");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = ResolverExpectedSymbolSets::default();

    collect_expected_resolver_impl_method_symbols(
        type_name,
        methods,
        &mut scope_cursor,
        &mut expected,
    );

    assert!(expected
        .declarations
        .contains(&(Namespace::Value, "Point.x_value".to_string())));
    assert!(expected.locals.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_callable_locals_collect_params_and_body() {
    let program = parse_program(
        r#"
main = (input: i32) i32 {
    value := input
    value
}
"#,
    );
    let Declaration::Function { params, body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = HashSet::new();

    expected_resolver_callable_locals(params, body, &mut scope_cursor, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "input"));
    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_scoped_expr_locals_collects_block_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    value
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut expected = HashSet::new();

    expected_resolver_scoped_expr_locals(body, &mut scope_cursor, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_child_expr_locals_collects_branch_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    loop {
        value := 1
        break
    }
    value
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block { statements, .. } = body else {
        panic!("expected block");
    };
    let Some(ast::Statement::Expression { expr, .. }) = statements.first() else {
        panic!("expected expression statement");
    };
    let Expression::Loop { body, .. } = expr else {
        panic!("expected loop expression");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_child_expr_locals(body, &mut scope_cursor, &locals, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "value"));
}

#[test]
fn expected_resolver_pattern_expr_locals_collects_pattern_and_body_bindings() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) {
            doubled := inner
            doubled
        }
        | None { 0 }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[1] else {
        panic!("expected function");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected block");
    };
    let Expression::Match { arms, .. } = expr.as_ref() else {
        panic!("expected match expression");
    };
    let arm = arms.first().expect("first match arm");
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_pattern_expr_locals(
        &arm.pattern,
        &arm.body,
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "inner"));
    assert!(expected.iter().any(|(name, _)| name == "doubled"));
}

#[test]
fn expected_resolver_pattern_locals_collects_struct_shorthand_bindings() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }

main = (point: Point) i32 {
    point ?
        | Point { x, y } { x + y }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[1] else {
        panic!("expected function");
    };
    let Expression::Block {
        expr: Some(expr), ..
    } = body
    else {
        panic!("expected block");
    };
    let Expression::Match { arms, .. } = expr.as_ref() else {
        panic!("expected match expression");
    };
    let arm = arms.first().expect("first match arm");
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_pattern_locals(&arm.pattern, &mut scope_cursor, &mut locals, &mut expected);

    assert!(expected.iter().any(|(name, _)| name == "x"));
    assert!(expected.iter().any(|(name, _)| name == "y"));
}

#[test]
fn expected_resolver_block_locals_collects_statement_and_final_expr_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    (input: i32) i32 {
        input
    }
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block {
        statements, expr, ..
    } = body
    else {
        panic!("expected block");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_block_locals(
        statements,
        expr.as_deref(),
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "value"));
    assert!(expected.iter().any(|(name, _)| name == "input"));
}

#[test]
fn expected_resolver_statement_locals_preserve_mutable_handoff() {
    let mut scope_cursor = ResolverScopeCursor::default();
    let mut locals = scope_cursor.new_scope();
    locals.insert("value".to_string(), true);
    let mut expected = HashSet::new();

    if resolver_var_decl_binds_local("value", false, false, &locals) {
        expected_resolver_var_decl_local("value", false, &mut locals, &mut expected);
    }

    assert!(
        expected.iter().all(|(name, _)| name != "value"),
        "immutable declaration should reuse the mutable handoff binding"
    );
}

#[test]
fn expected_resolver_closure_locals_collects_params_and_body_bindings() {
    let program = parse_program(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        inner := input
        inner
    }
    0
}
"#,
    );
    let Declaration::Function { body, .. } = &program.declarations[0] else {
        panic!("expected function");
    };
    let Expression::Block { statements, .. } = body else {
        panic!("expected block");
    };
    let Some(ast::Statement::VarDecl {
        value:
            Expression::Closure {
                params,
                body: closure_body,
                ..
            },
        ..
    }) = statements.first()
    else {
        panic!("expected closure var declaration");
    };
    let mut scope_cursor = ResolverScopeCursor::default();
    let locals = scope_cursor.new_scope();
    let mut expected = HashSet::new();

    expected_resolver_closure_locals(
        params,
        closure_body,
        &mut scope_cursor,
        &locals,
        &mut expected,
    );

    assert!(expected.iter().any(|(name, _)| name == "input"));
    assert!(expected.iter().any(|(name, _)| name == "inner"));
}
