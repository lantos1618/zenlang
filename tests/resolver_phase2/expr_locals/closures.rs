use super::*;

#[test]
fn resolver_records_closure_locals() {
    let program = parse_program(
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

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let mapper = table
        .lookup_scoped(Namespace::Local, "mapper")
        .expect("closure binding local symbol");
    let input = table
        .lookup_scoped(Namespace::Local, "input")
        .expect("closure parameter local symbol");
    let inner = table
        .lookup_scoped(Namespace::Local, "inner")
        .expect("closure body local symbol");

    assert_ne!(mapper.scope_id, input.scope_id);
    assert_ne!(input.scope_id, inner.scope_id);
    assert!(inner.scope_id > input.scope_id);
    assert_eq!(mapper.is_mutable, Some(false));
    assert_eq!(input.is_mutable, Some(false));
    assert_eq!(inner.is_mutable, Some(false));
}

#[test]
fn resolver_records_mutable_closure_parameter_locals() {
    let program = parse_program(
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

    let table = Resolver::new().resolve_program(&program).expect("resolve");
    let input = table
        .lookup_scoped(Namespace::Local, "input")
        .expect("closure parameter local symbol");

    assert_eq!(input.is_mutable, Some(true));
}
