use super::*;

#[test]
fn resolver_records_pattern_locals() {
    let table = resolved_symbols(
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

    let value = scoped_symbol(&table, Namespace::Local, "value");
    let inner = scoped_symbol(&table, Namespace::Local, "inner");

    assert_ne!(value.scope_id, inner.scope_id);
    assert_eq!(inner.is_mutable, Some(false));
    assert!(inner.scope_id > value.scope_id);
}
