use super::*;
mod literals;

#[test]
fn resolver_records_struct_field_types() {
    let table = resolved_symbols(
        r#"
Point: { x: i32, y: f64 }
"#,
    );

    assert_eq!(
        symbol(&table, Namespace::Type, "Point")
            .field_types
            .as_deref(),
        Some(
            &[
                ("x".to_string(), zen::ast::AstType::I32),
                ("y".to_string(), zen::ast::AstType::F64)
            ][..]
        )
    );
}

#[test]
fn resolver_rejects_duplicate_struct_field_names() {
    let err = resolver_errors(
        r#"
Point: { x: i32, x: i64 }
"#,
        "duplicate struct field names should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate field `x` for struct `Point`");
}

#[test]
fn resolver_records_struct_function_type_fields() {
    let table = resolved_symbols(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );

    let pipeline = symbol(&table, Namespace::Type, "Pipeline");
    assert_field_type_metadata(
        pipeline.field_types.as_deref(),
        &[("callback", "(i32) i32")],
    );
    assert_eq!(
        pipeline.field_types.as_deref(),
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
    let table = resolved_symbols(
        r#"
Box<T>: { value: T }
"#,
    );

    assert_field_type_metadata(
        symbol(&table, Namespace::Type, "Box")
            .field_types
            .as_deref(),
        &[("value", "T")],
    );
}

#[test]
fn resolver_records_struct_field_default_locals() {
    let table = resolved_symbols(
        r#"
Point: {
    x: i32 = {
        value = 1
        value
    }
}
"#,
    );

    let value = scoped_symbol(&table, Namespace::Local, "value");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}
