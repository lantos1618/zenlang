use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Point' has field count 1, expected 2")),
        "expected resolver struct field count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field type mismatch should fail");

    let expected =
        "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver struct field type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct function type field mismatch should fail");

    let expected = "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver struct function type field diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed struct field metadata mismatch should fail");

    let expected = "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver typed struct field diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic struct field mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
        )),
        "expected resolver generic struct field diagnostic, got {err:?}"
    );
}
