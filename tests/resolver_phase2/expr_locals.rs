use super::*;

#[path = "expr_locals/closures.rs"]
mod closures;
#[path = "expr_locals/patterns.rs"]
mod patterns;

#[test]
fn resolver_rejects_unknown_unqualified_function_calls() {
    let program = parse_program(
        r#"
known = () i32 { 1 }
main = () i32 { missing() }
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown function call should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing'")),
        "expected missing function diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_parameter_and_local_symbols() {
    let program = parse_program(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    frozen = value
    frozen
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let input = table
        .lookup_scoped(Namespace::Local, "input")
        .expect("parameter symbol");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("local symbol");
    let frozen = table
        .lookup_scoped(Namespace::Local, "frozen")
        .expect("immutable local symbol");

    assert_ne!(input.id, value.id);
    assert_ne!(input.scope_id, value.scope_id);
    assert_eq!(input.is_mutable, Some(true));
    assert_eq!(value.is_mutable, Some(true));
    assert_eq!(frozen.is_mutable, Some(false));
}

#[test]
fn resolver_records_top_level_expr_locals() {
    let program = parse_program(
        r#"
value := 1
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("top-level expr local symbol");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}

#[test]
fn resolver_rejects_unknown_enum_variant_expressions() {
    let program = parse_program(
        r#"
Status: Ok, Err

main = () i32 {
    value = Status.Pending
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown enum variant expression should fail in resolver");

    assert!(
        err.iter()
            .any(|d| d.message.contains("enum `Status` has no variant `Pending`")),
        "expected unknown enum variant diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_missing_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.Some
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("missing enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.Some` requires a payload")),
        "expected missing enum variant payload diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unexpected_enum_variant_payload_expressions() {
    let program = parse_program(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.None(1)
    0
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unexpected enum variant payload expression should fail in resolver");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("enum variant `Maybe.None` does not accept a payload")),
        "expected unexpected enum variant payload diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_records_same_name_locals_in_distinct_scopes() {
    let program = parse_program(
        r#"
main = () i32 {
    value := 1
    {
        value := 2
        value
    }
    value
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let values: Vec<_> = table
        .symbols()
        .iter()
        .filter(|symbol| symbol.namespace == Namespace::Local && symbol.name == "value")
        .collect();

    assert_eq!(values.len(), 2);
    assert_ne!(values[0].id, values[1].id);
    assert_ne!(values[0].scope_id, values[1].scope_id);
    assert!(values.iter().all(|symbol| symbol.is_mutable == Some(false)));
}

#[test]
fn resolver_rejects_duplicate_bindings_in_same_scope() {
    let program = parse_program(
        r#"
main = (input: i32, input: i32) i32 {
    value = 1
    value = 2
    value
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("duplicate locals should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate local symbol 'input'")),
        "expected duplicate parameter diagnostic, got {err:?}"
    );
    assert!(
        err.iter()
            .any(|d| d.message.contains("duplicate local symbol 'value'")),
        "expected duplicate local diagnostic, got {err:?}"
    );
}

#[test]
fn resolver_rejects_unknown_local_identifier_references() {
    let program = parse_program(
        r#"
main = () i32 {
    missing_local
}
"#,
    );

    let err = Resolver::new()
        .resolve_program(&program)
        .expect_err("unknown local identifier should fail");

    assert!(
        err.iter()
            .any(|d| d.message.contains("unknown value symbol 'missing_local'")),
        "expected missing local diagnostic, got {err:?}"
    );
}
