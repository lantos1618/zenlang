use super::*;

#[test]
fn resolver_accepts_behavior_requires_known_type_and_behavior() {
    resolved_symbols(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );
}

#[test]
fn resolver_rejects_duplicate_behavior_required_edges() {
    let err = resolver_errors(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

Point: { x: i32 }

Point.requires(Json)
Point.requires(Json)
"#,
        "duplicate behavior requires should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate required behavior `Json`");
}

#[test]
fn resolver_rejects_behavior_requires_unknown_symbols() {
    let err = resolver_errors(
        "Missing.requires(Json)",
        "unknown requires symbols should fail",
    );

    assert_resolver_error_contains(&err, "unknown type symbol 'Missing'");
    assert_resolver_error_contains(&err, "unknown behavior symbol 'Json'");
}

#[test]
fn resolver_accepts_behavior_extends_known_behaviors() {
    resolved_symbols(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );
}

#[test]
fn resolver_rejects_duplicate_behavior_parent_edges() {
    let err = resolver_errors(
        r#"
Json: behavior {
    stringify: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
        "duplicate behavior inheritance should fail in resolver",
    );

    assert_resolver_error_contains(&err, "duplicate behavior parent `Json`");
}

#[test]
fn resolver_rejects_behavior_extends_unknown_symbols() {
    let err = resolver_errors(
        "PrettyJson.extends(Json)",
        "unknown behavior inheritance symbols should fail",
    );

    assert_resolver_error_contains(&err, "unknown behavior symbol 'PrettyJson'");
    assert_resolver_error_contains(&err, "unknown behavior symbol 'Json'");
}
