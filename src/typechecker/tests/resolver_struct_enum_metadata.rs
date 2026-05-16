use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_counts() {
    let program = parse_program(
        r#"
Point: { x: i32, y: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_count_for_test(Namespace::Type, "Point", Some(1));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver type symbol 'Point' has field count 1, expected 2")),
        "expected resolver struct field count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32, y: f64 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Point",
        Some(vec![
            ("x".to_string(), "i32".to_string()),
            ("y".to_string(), "i32".to_string()),
        ]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct field type mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Point' has fields '(x: i32, y: i32)', expected '(x: i32, y: f64)'"
            )),
            "expected resolver struct field type diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_function_type_fields() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct function type field mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver struct function type field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_typed_field_metadata() {
    let program = parse_program(
        r#"
Pipeline: { callback: (i32) i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Pipeline",
        Some(vec![("callback".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed struct field metadata mismatch should fail");

    assert!(
            err.iter().any(|d| d.message.contains(
                "resolver type symbol 'Pipeline' has typed fields '(callback: i32)', expected '(callback: (i32) i32)'"
            )),
            "expected resolver typed struct field diagnostic, got {err:?}"
        );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_struct_field_types() {
    let program = parse_program(
        r#"
Box<T>: { value: T }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Box",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic struct field mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver type symbol 'Box' has fields '(value: i32)', expected '(value: T)'"
        )),
        "expected resolver generic struct field diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Point", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Type,
        "Point",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
    symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct/enum kind metadata should fail");

    for expected in [
        "resolver type symbol 'Point' has variant names metadata, expected none",
        "resolver type symbol 'Point' has variant payload type metadata, expected none",
        "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
        "resolver type symbol 'Option' has field count metadata, expected none",
        "resolver type symbol 'Option' has field types metadata, expected none",
        "resolver type symbol 'Option' has typed field types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
        );
    }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_counts() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_count_for_test(Namespace::Variant, "Some", Some(0));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload count mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload count 0, expected 1")),
        "expected resolver enum variant payload count diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_visibility() {
    let program = parse_program(
        r#"
pub Option<T>: Some(T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_public_for_test(Namespace::Variant, "Some", false);
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant visibility mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has visibility private, expected public")),
        "expected resolver enum variant visibility diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_payload_types() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Some",
        Some("bool".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant payload type mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Some' has payload type 'bool', expected 'i32'")),
        "expected resolver enum variant payload type diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("i32".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_typed_payload_metadata() {
    let program = parse_program(
        r#"
Callback: Wrap((i32) i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_for_test(Namespace::Variant, "Wrap", Some(AstType::I32));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver typed enum payload metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d.message.contains(
            "resolver variant symbol 'Wrap' has typed payload type 'i32', expected '(i32) i32'"
        )),
        "expected resolver typed enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads() {
    let program = parse_program(
        r#"
Callback<T>: Wrap((T) T), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Wrap",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum function type payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Wrap' has payload type 'T', expected '(T) T'")),
        "expected resolver generic enum function type payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_generic_enum_payload_types() {
    let program = parse_program(
        r#"
Result<T, E>: Ok(T), Err(E)
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Variant,
        "Err",
        Some("T".to_string()),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver generic enum payload mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver variant symbol 'Err' has payload type 'T', expected 'E'")),
        "expected resolver generic enum payload diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_variant_absent_other_metadata() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_import_source_for_test(Namespace::Variant, "Some", Some("std".to_string()));
    symbols.set_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["value".to_string()]),
    );
    symbols.set_parameter_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Variant, "Some", Some(vec![AstType::I32]));
    symbols.set_return_type_name_for_test(Namespace::Variant, "Some", Some("i32".to_string()));
    symbols.set_return_type_for_test(Namespace::Variant, "Some", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Variant, "Some", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Variant, "Some", Some(vec!["Other".to_string()]));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Variant,
        "Some",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Variant,
        "Some",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver variant non-variant metadata should fail");

    for expected in [
            "resolver variant symbol 'Some' has source 'std', expected none",
            "resolver variant symbol 'Some' has parameter count metadata, expected none",
            "resolver variant symbol 'Some' has parameter names metadata, expected none",
            "resolver variant symbol 'Some' has parameter types metadata, expected none",
            "resolver variant symbol 'Some' has typed parameter types metadata, expected none",
            "resolver variant symbol 'Some' has return type metadata, expected none",
            "resolver variant symbol 'Some' has typed return type metadata, expected none",
            "resolver variant symbol 'Some' has type parameter count metadata, expected none",
            "resolver variant symbol 'Some' has type parameter names metadata, expected none",
            "resolver variant symbol 'Some' has type parameter bounds metadata, expected none",
            "resolver variant symbol 'Some' has typed type parameter bound refs metadata, expected none",
            "resolver variant symbol 'Some' has field count metadata, expected none",
            "resolver variant symbol 'Some' has field types metadata, expected none",
            "resolver variant symbol 'Some' has typed field types metadata, expected none",
            "resolver variant symbol 'Some' has variant names metadata, expected none",
            "resolver variant symbol 'Some' has behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior methods metadata, expected none",
            "resolver variant symbol 'Some' has behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior parents metadata, expected none",
            "resolver variant symbol 'Some' has behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior impls metadata, expected none",
            "resolver variant symbol 'Some' has behavior requires metadata, expected none",
            "resolver variant symbol 'Some' has typed behavior requires metadata, expected none",
        ] {
            assert!(
                err.iter().any(|d| d.message.contains(expected)),
                "expected resolver variant metadata diagnostic '{expected}', got {err:?}"
            );
        }
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Option", Some(vec!["Some".to_string()]));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant names mismatch should fail");

    let expected = "resolver type symbol 'Option' has variants '(Some)', expected '(Some, None)'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant names diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_enum_variant_owner_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_owner_name_for_test(Namespace::Variant, "Some", Some("Result".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver enum variant owner mismatch should fail");

    let expected = "resolver variant symbol 'Some' has owner 'Result', expected 'Option'";
    assert!(
        err.iter().any(|d| d.message.contains(expected)),
        "expected resolver enum variant owner diagnostic, got {err:?}"
    );
}
