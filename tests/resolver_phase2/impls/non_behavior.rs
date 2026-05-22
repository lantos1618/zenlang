use super::*;

#[test]
fn resolver_accepts_non_behavior_impl_blocks_as_method_symbols() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let symbols = Resolver::new()
        .resolve_program(&program)
        .expect("non-behavior impl blocks should resolve");

    let get = symbols
        .lookup(Namespace::Value, "Point.get")
        .expect("impl method symbol");
    assert_eq!(get.parameter_count, Some(1));
    assert_eq!(
        get.parameter_type_names.as_deref(),
        Some(&["Point".to_string()][..])
    );
}

#[test]
fn resolver_rejects_duplicate_non_behavior_impl_method_names() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate non-behavior impl methods should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate value symbol 'Point.get'")),
        "expected duplicate impl method symbol diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("impl method colliding with top-level method should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate value symbol 'Point.get'")),
        "expected duplicate method symbol diagnostic, got {err:?}"
    );
}
