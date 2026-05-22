use super::*;

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
