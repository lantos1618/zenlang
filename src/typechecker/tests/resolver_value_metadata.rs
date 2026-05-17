use super::*;

mod absent_declaration_metadata;

#[test]
fn check_program_with_symbols_validates_resolver_function_visibility() {
    let program = parse_program(
        r#"
pub exported = () i32 { 1 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Value, "exported", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'exported' has visibility private, expected public")),
        "expected resolver function visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_return_type() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "main", Some("bool".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function return mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'main' has return type 'bool', expected 'i32'")),
        "expected resolver function return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_return_metadata() {
    let program = parse_program(
        r#"
factory = () (i32) i32 {
    (value: i32) i32 { value }
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_return_type_name_for_test(Namespace::Value, "factory", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function type return metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'factory' has return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver function type return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_typed_signature_metadata() {
    let program = parse_program(
        r#"
apply = (callback: (i32) i32) (i32) i32 {
    callback
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_types_for_test(Namespace::Value, "apply", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Value, "apply", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed function signature metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'apply' has typed parameter types '(i32)', expected '((i32) i32)'"
            )),
            "expected resolver typed parameter diagnostic, got {err:?}"
        );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'apply' has typed return type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed return diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_counts() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Value, "identity", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic arity mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver value symbol 'identity' has type parameter count 0, expected 1")),
        "expected resolver function generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_names() {
    let program = parse_program(
        r#"
identity<T> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(
        Namespace::Value,
        "identity",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic parameter name mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver value symbol 'identity' has type parameter names '(U)', expected '(T)'"
        )),
        "expected resolver function generic parameter name diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) str
}
encode<T: Json> = (value: T) str { "encoded" }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Value,
        "encode",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver value symbol 'encode' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver function generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_function_type_parameter_bound_refs() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
identity<T: Json<T>> = (value: T) T { value }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Value,
        "identity",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: vec![AstType::Str],
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function generic bound ref mismatch should fail");

    let expected = "resolver value symbol 'identity' has type parameter bound refs '(T: Json<str>)', expected '(T: Json<T>)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver function generic bound ref diagnostic, got {err:?}"
    );
}
