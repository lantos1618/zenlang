use super::*;

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
