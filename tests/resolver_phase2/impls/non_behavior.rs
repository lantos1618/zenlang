use super::*;

#[test]
fn resolver_accepts_non_behavior_impl_blocks_as_method_symbols() {
    let symbols = resolved_symbols(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
    );

    let get = symbol(&symbols, Namespace::Value, "Point.get");
    assert_string_metadata(get.parameter_names.as_deref(), &["self"]);
    assert_type_metadata(get.parameter_types.as_deref(), &["Point"]);
}

#[test]
fn resolver_rejects_duplicate_non_behavior_impl_method_names() {
    let err = resolver_errors(
        r#"
Point: { x: i32 }

Point.impl = {
    get = (self: Point) i32 { self.x }
    get = (self: Point) i32 { self.x }
}
"#,
        "duplicate non-behavior impl methods should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate value symbol 'Point.get'");
}

#[test]
fn resolver_rejects_non_behavior_impl_method_colliding_with_top_level_method() {
    let err = resolver_errors(
        r#"
Point: { x: i32 }

Point.get = (self: Point) i32 { self.x }

Point.impl = {
    get = (self: Point) i32 { self.x }
}
"#,
        "impl method colliding with top-level method should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate value symbol 'Point.get'");
}
