use super::*;

#[test]
fn resolver_records_pattern_locals() {
    let program = parse_program(
        r#"
Option:
    None,
    Some(i32)

main = (value: Option) i32 {
    value ?
        | Some(inner) { inner }
        | None { 0 }
}
"#,
    );

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let value = table
        .lookup_scoped(Namespace::Local, "value")
        .expect("parameter local symbol");
    let inner = table
        .lookup_scoped(Namespace::Local, "inner")
        .expect("pattern local symbol");

    assert_ne!(value.scope_id, inner.scope_id);
    assert_eq!(inner.is_mutable, Some(false));
    assert!(inner.scope_id > value.scope_id);
}
