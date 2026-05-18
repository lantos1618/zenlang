use super::*;

mod behavior_methods;

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_counts() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_count_for_test(Namespace::Type, "Box", Some(0));
    symbols.set_type_parameter_count_for_test(Namespace::Behavior, "Serializable", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic arity mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter count 0, expected 1")),
        "expected resolver type generic arity diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver behavior symbol 'Serializable' has type parameter count 0, expected 1"
        )),
        "expected resolver behavior generic arity diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_names() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Serializable<T>: behavior {
    encode: (T) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_names_for_test(Namespace::Type, "Box", Some(vec!["U".to_string()]));
    symbols.set_type_parameter_names_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec!["U".to_string()]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic parameter name mismatches should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has type parameter names '(U)', expected '(T)'")),
        "expected resolver type generic parameter name diagnostic, got {err:?}"
    );
    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter names '(U)', expected '(T)'"
            )),
            "expected resolver behavior generic parameter name diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_visibility() {
    let program = parse_program(
        r#"
pub Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Type, "Box", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Box' has visibility private, expected public")),
        "expected resolver type visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_visibility() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Behavior, "Json", true);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver behavior symbol 'Json' has visibility public, expected private")),
        "expected resolver behavior visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
Box<T: Json>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("T".to_string(), "Other".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Box' has type parameter bounds '(T: Other)', expected '(T: Json)'"
            )),
            "expected resolver type generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_type_parameter_bounds() {
    let program = parse_program(
        r#"
Json<T>: behavior {
    encode: (Self) T
}
Serializable<T: Json<T>>: behavior {
    serialize: (Self) T
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Behavior,
        "Serializable",
        Some(vec![("T".to_string(), "Json<i32>".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior generic bound mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver behavior symbol 'Serializable' has type parameter bounds '(T: Json<i32>)', expected '(T: Json<T>)'"
            )),
            "expected resolver behavior generic bound diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_type_like_absent_value_metadata() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
Json: behavior {
    encode: (Self) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Type, "Box", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Type, "Box", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Type, "Box", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Type, "Box", Some(AstType::I32));
    symbols.set_import_source_for_test(Namespace::Behavior, "Json", Some("std".to_string()));
    symbols.set_parameter_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Self".to_string()]),
    );
    symbols.set_parameter_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![AstType::SelfType]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver type-like value metadata should fail");

    for expected in [
        "resolver type symbol 'Box' has source 'std', expected none",
        "resolver type symbol 'Box' has parameter count metadata, expected none",
        "resolver type symbol 'Box' has return type metadata, expected none",
        "resolver type symbol 'Box' has typed return type metadata, expected none",
        "resolver behavior symbol 'Json' has source 'std', expected none",
        "resolver behavior symbol 'Json' has parameter names metadata, expected none",
        "resolver behavior symbol 'Json' has parameter types metadata, expected none",
        "resolver behavior symbol 'Json' has typed parameter types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver type-like value metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_behavior_absent_type_metadata() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Behavior, "Json", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Behavior, "Json", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Behavior,
        "Json",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Behavior, "Json", Some(AstType::I32));
    symbols.set_behavior_impl_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec!["Debug".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Behavior,
        "Json",
        Some(vec![BehaviorRefMetadata {
            name: "Debug".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver behavior type metadata should fail");

    for expected in [
        "resolver behavior symbol 'Json' has field count metadata, expected none",
        "resolver behavior symbol 'Json' has field types metadata, expected none",
        "resolver behavior symbol 'Json' has typed field types metadata, expected none",
        "resolver behavior symbol 'Json' has variant names metadata, expected none",
        "resolver behavior symbol 'Json' has variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has typed variant payload type metadata, expected none",
        "resolver behavior symbol 'Json' has behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior impls metadata, expected none",
        "resolver behavior symbol 'Json' has behavior requires metadata, expected none",
        "resolver behavior symbol 'Json' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver behavior type metadata diagnostic '{expected}', got {err:?}"
        );
    }
}
