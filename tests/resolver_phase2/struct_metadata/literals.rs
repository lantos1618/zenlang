use super::*;

#[test]
fn resolver_rejects_duplicate_struct_literal_fields() {
    let err = resolver_errors(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, x: 2 }
    0
}
"#,
        "duplicate struct literal field should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate field `x` for struct `Point`");
}

#[test]
fn resolver_rejects_unknown_struct_literal_fields() {
    let err = resolver_errors(
        r#"
Point: { x: i32 }

main = () i32 {
    point = Point { x: 1, y: 2 }
    0
}
"#,
        "unknown struct literal field should fail in resolver",
    );

    assert_resolver_error_contains(&err, "unknown field `y` for struct `Point`");
}

#[test]
fn resolver_rejects_missing_struct_literal_fields() {
    let err = resolver_errors(
        r#"
Point: { x: i32, y: i32 }

main = () i32 {
    point = Point { x: 1 }
    0
}
"#,
        "missing struct literal field should fail in resolver",
    );

    assert_resolver_error_contains(&err, "missing field `y` for struct `Point`");
}

#[test]
fn resolver_rejects_unknown_struct_literal_types() {
    let err = resolver_errors(
        r#"
main = () i32 {
    point = Point { x: 1 }
    0
}
"#,
        "unknown struct literal type should fail in resolver",
    );

    assert_resolver_error_contains(&err, "unknown type symbol 'Point'");
}
