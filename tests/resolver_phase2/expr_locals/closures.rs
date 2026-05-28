use super::*;

#[test]
fn resolver_records_closure_locals() {
    let table = resolved_symbols(
        r#"
main = () i32 {
    mapper = (input: i32) i32 {
        inner = input
        inner
    }
    0
}
"#,
    );

    let mapper = scoped_symbol(&table, Namespace::Local, "mapper");
    let input = scoped_symbol(&table, Namespace::Local, "input");
    let inner = scoped_symbol(&table, Namespace::Local, "inner");

    assert_ne!(mapper.scope_id, input.scope_id);
    assert_ne!(input.scope_id, inner.scope_id);
    assert!(inner.scope_id > input.scope_id);
    assert_eq!(mapper.is_mutable, Some(false));
    assert_eq!(input.is_mutable, Some(false));
    assert_eq!(inner.is_mutable, Some(false));
}

#[test]
fn resolver_records_mutable_closure_parameter_locals() {
    let table = resolved_symbols(
        r#"
main = () i32 {
    mapper = (mut input: i32) i32 {
        input = input + 1
        input
    }
    0
}
"#,
    );

    let input = scoped_symbol(&table, Namespace::Local, "input");

    assert_eq!(input.is_mutable, Some(true));
}
