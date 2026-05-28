use super::*;
mod closures;
mod patterns;

#[test]
fn resolver_rejects_unknown_unqualified_function_calls() {
    let err = resolver_errors(
        r#"
known = () i32 { 1 }
main = () i32 { missing() }
"#,
        "unknown function call should fail",
    );

    assert_resolver_error_contains(&err, "unknown value symbol 'missing'");
}

#[test]
fn resolver_records_parameter_and_local_symbols() {
    let table = resolved_symbols(
        r#"
main = (mut input: i32) i32 {
    value ::= input
    frozen = value
    frozen
}
"#,
    );

    let input = scoped_symbol(&table, Namespace::Local, "input");
    let value = scoped_symbol(&table, Namespace::Local, "value");
    let frozen = scoped_symbol(&table, Namespace::Local, "frozen");

    assert_ne!(input.id, value.id);
    assert_ne!(input.scope_id, value.scope_id);
    assert_eq!(input.is_mutable, Some(true));
    assert_eq!(value.is_mutable, Some(true));
    assert_eq!(frozen.is_mutable, Some(false));
}

#[test]
fn resolver_records_top_level_expr_locals() {
    let table = resolved_symbols(
        r#"
value := 1
"#,
    );

    let value = scoped_symbol(&table, Namespace::Local, "value");

    assert_eq!(value.is_mutable, Some(false));
    assert!(value.scope_id > 0);
}

#[test]
fn resolver_rejects_unknown_enum_variant_expressions() {
    let err = resolver_errors(
        r#"
Status: Ok, Err

main = () i32 {
    value = Status.Pending
    0
}
"#,
        "unknown enum variant expression should fail in resolver",
    );

    assert_resolver_error_contains(&err, "enum `Status` has no variant `Pending`");
}

#[test]
fn resolver_rejects_missing_enum_variant_payload_expressions() {
    let err = resolver_errors(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.Some
    0
}
"#,
        "missing enum variant payload expression should fail in resolver",
    );

    assert_resolver_error_contains(&err, "enum variant `Maybe.Some` requires a payload");
}

#[test]
fn resolver_rejects_unexpected_enum_variant_payload_expressions() {
    let err = resolver_errors(
        r#"
Maybe: Some(i32), None

main = () i32 {
    value = Maybe.None(1)
    0
}
"#,
        "unexpected enum variant payload expression should fail in resolver",
    );

    assert_resolver_error_contains(&err, "enum variant `Maybe.None` does not accept a payload");
}

#[test]
fn resolver_records_same_name_locals_in_distinct_scopes() {
    let table = resolved_symbols(
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
    let err = resolver_errors(
        r#"
main = (input: i32, input: i32) i32 {
    value = 1
    value = 2
    value
}
"#,
        "duplicate locals should fail",
    );

    assert_resolver_error_contains(&err, "duplicate local symbol 'input'");
    assert_resolver_error_contains(&err, "duplicate local symbol 'value'");
}

#[test]
fn resolver_rejects_unknown_local_identifier_references() {
    let err = resolver_errors(
        r#"
main = () i32 {
    missing_local
}
"#,
        "unknown local identifier should fail",
    );

    assert_resolver_error_contains(&err, "unknown value symbol 'missing_local'");
}
