use super::*;

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

#[test]
fn check_program_with_symbols_validates_resolver_function_absent_declaration_metadata() {
    let program = parse_program(
        r#"
main = () i32 { 0 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Value, "main", Some("std".to_string()));
    symbols.set_field_count_for_test(Namespace::Value, "main", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Value, "main", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Value,
        "main",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Value, "main", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Value,
        "main",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::Named("Self".to_string())],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Value,
        "main",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Value,
        "main",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver function declaration metadata should fail");

    for expected in [
        "resolver value symbol 'main' has source 'std', expected none",
        "resolver value symbol 'main' has field count metadata, expected none",
        "resolver value symbol 'main' has field types metadata, expected none",
        "resolver value symbol 'main' has typed field types metadata, expected none",
        "resolver value symbol 'main' has variant names metadata, expected none",
        "resolver value symbol 'main' has variant payload type metadata, expected none",
        "resolver value symbol 'main' has typed variant payload type metadata, expected none",
        "resolver value symbol 'main' has behavior methods metadata, expected none",
        "resolver value symbol 'main' has typed behavior methods metadata, expected none",
        "resolver value symbol 'main' has behavior parents metadata, expected none",
        "resolver value symbol 'main' has typed behavior parents metadata, expected none",
        "resolver value symbol 'main' has behavior impls metadata, expected none",
        "resolver value symbol 'main' has typed behavior impls metadata, expected none",
        "resolver value symbol 'main' has behavior requires metadata, expected none",
        "resolver value symbol 'main' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver function declaration metadata diagnostic '{expected}', got {err:?}"
        );
    }
}
