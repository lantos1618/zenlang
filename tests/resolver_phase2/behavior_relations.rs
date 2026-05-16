use super::*;

#[test]
fn resolver_accepts_behavior_requires_known_type_and_behavior() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
"#,
    );

    Resolver::new()
        .resolve_program(&program)
        .expect("known requires assertion should resolve");
}

#[test]
fn resolver_rejects_duplicate_behavior_required_edges() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

Point: { x: i32 }

Point.requires(Json)
Point.requires(Json)
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior requires should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate required behavior `Json`")),
        "expected duplicate required behavior diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_behavior_requires_unknown_symbols() {
    let program = parse_program("Missing.requires(Json)");

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown requires symbols should fail");
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown type symbol 'Missing'")),
        "expected unknown type diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'Json'")),
        "expected unknown behavior diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_accepts_behavior_extends_known_behaviors() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
"#,
    );

    Resolver::new()
        .resolve_program(&program)
        .expect("known behavior inheritance should resolve");
}

#[test]
fn resolver_rejects_duplicate_behavior_parent_edges() {
    let program = parse_program(
        r#"
Json: behavior {
    stringify: (Self) str
}

PrettyJson: behavior {
    pretty: (Self) str
}

PrettyJson.extends(Json)
PrettyJson.extends(Json)
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate behavior inheritance should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate behavior parent `Json`")),
        "expected duplicate behavior parent diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_behavior_extends_unknown_symbols() {
    let program = parse_program("PrettyJson.extends(Json)");

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown behavior inheritance symbols should fail");
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'PrettyJson'")),
        "expected unknown child behavior diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown behavior symbol 'Json'")),
        "expected unknown parent behavior diagnostic, got {err:?}"
    );
}
