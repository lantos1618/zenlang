use super::*;

#[test]
fn resolver_records_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
Empty: { }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
            .field_count,
        Some(2)
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Empty")
            .expect("Empty symbol")
            .field_count,
        Some(0)
    );
}

#[test]
fn resolver_records_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
            .field_type_names
            .as_deref(),
        Some(
            &[
                ("x".to_string(), "i32".to_string()),
                ("y".to_string(), "f64".to_string())
            ][..]
        )
    );
}

#[test]
fn resolver_rejects_duplicate_struct_field_names() {
    let program = parse_program(
        r#"
Point: { x: i32, x: i64 }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate struct field names should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate field `x` for struct `Point`")),
        "expected duplicate struct field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Pipeline")
            .expect("Pipeline symbol")
            .field_type_names
            .as_deref(),
        Some(&[("callback".to_string(), "(i32) i32".to_string())][..])
    );
    assert_eq!(
        table
            .lookup(Namespace::Type, "Pipeline")
            .expect("Pipeline symbol")
            .field_types
            .as_deref(),
        Some(
            &[(
                "callback".to_string(),
                zen::ast::AstType::Function {
                    params: vec![zen::ast::AstType::I32],
                    ret: Box::new(zen::ast::AstType::I32),
                }
            )][..]
        )
    );
}

#[test]
fn resolver_records_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");

    assert_eq!(
        table
            .lookup(Namespace::Type, "Box")
            .expect("Box symbol")
            .field_type_names
            .as_deref(),
        Some(&[("value".to_string(), "T".to_string())][..])
    );
}

#[test]
fn resolver_records_struct_field_default_locals() {
    let program = parse_program(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("struct field default local symbol");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}

#[test]
fn resolver_rejects_duplicate_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, x: 2 }
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate field `x` for struct `Point`")),
        "expected duplicate struct literal field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, y: 2 }
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown field `y` for struct `Point`")),
        "expected unknown struct literal field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_missing_struct_literal_fields() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }

main = () i32 {
    point = Point { x: 1 }
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("missing struct literal field should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("missing field `y` for struct `Point`")),
        "expected missing struct literal field diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_struct_literal_types() {
    let program = parse_program(
        r#"
main = () i32 {
    point = Point { x: 1 }
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown struct literal type should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Point'")),
        "expected unknown struct literal type diagnostic, got {err:?}"
    );
}
