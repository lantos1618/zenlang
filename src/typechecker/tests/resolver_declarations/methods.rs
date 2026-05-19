use super::*;

#[test]
fn check_program_with_symbols_requires_resolver_method_receiver_type() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Point.label = () StaticString { "point" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.remove_for_test(Namespace::Type, "Point");
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("missing receiver type resolver symbol should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver symbol table missing type symbol 'Point'")),
        "expected missing method receiver type symbol diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.get<T> = (self: Box<T>) T {
    self.value
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.get",
        Some(vec!["Box<i32>".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method signature mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'Box.get' has parameter types '(Box<i32>)', expected '(Box<T>)'"
        )),
        "expected resolver method signature diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_method_function_type_signature() {
    let program = parse_program(
        r#"
Box<T>: {
    value: T
}

Box.map<T> = (self: Box<T>, callback: (T) T) (T) T {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_type_names_for_test(
        Namespace::Value,
        "Box.map",
        Some(vec!["Box<T>".to_string(), "T".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver method function type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'Box.map' has parameter types '(Box<T>, T)', expected '(Box<T>, (T) T)'"
            )),
            "expected resolver method function type diagnostic, got {err:?}"
        );
}
