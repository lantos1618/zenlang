use super::*;

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
