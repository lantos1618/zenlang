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
        None,
        &[],
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
