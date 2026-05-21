use super::*;

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
