use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_declaration_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has parameter count metadata, expected none")),
        "expected resolver local parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver local symbol 'a' has return type metadata, expected none")),
        "expected resolver local return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_local_absent_type_metadata() {
    let program = parse_program(
        r#"
add = (a: i32, b: i32) i32 { a + b }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Local, "a", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(Namespace::Local, "a", Some(vec!["i32".to_string()]));
    symbols.set_parameter_types_for_test(Namespace::Local, "a", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Local, "a", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Local,
        "a",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Local, "a", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Local, "a", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Local, "a", Some(1));
    symbols.set_variant_payload_type_name_for_test(Namespace::Local, "a", Some("i32".to_string()));
    symbols.set_variant_payload_type_for_test(Namespace::Local, "a", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Local,
        "a",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "StaticString".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(Namespace::Local, "a", Some(vec!["Json".to_string()]));
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Local,
        "a",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Local,
        "a",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver local type metadata should fail");

    for expected in [
        "resolver local symbol 'a' has parameter names metadata, expected none",
        "resolver local symbol 'a' has parameter types metadata, expected none",
        "resolver local symbol 'a' has typed parameter types metadata, expected none",
        "resolver local symbol 'a' has typed return type metadata, expected none",
        "resolver local symbol 'a' has type parameter count metadata, expected none",
        "resolver local symbol 'a' has type parameter names metadata, expected none",
        "resolver local symbol 'a' has type parameter bounds metadata, expected none",
        "resolver local symbol 'a' has typed type parameter bound refs metadata, expected none",
        "resolver local symbol 'a' has field count metadata, expected none",
        "resolver local symbol 'a' has field types metadata, expected none",
        "resolver local symbol 'a' has typed field types metadata, expected none",
        "resolver local symbol 'a' has variant names metadata, expected none",
        "resolver local symbol 'a' has variant owner metadata, expected none",
        "resolver local symbol 'a' has variant payload count metadata, expected none",
        "resolver local symbol 'a' has variant payload type metadata, expected none",
        "resolver local symbol 'a' has typed variant payload type metadata, expected none",
        "resolver local symbol 'a' has behavior methods metadata, expected none",
        "resolver local symbol 'a' has typed behavior methods metadata, expected none",
        "resolver local symbol 'a' has behavior parents metadata, expected none",
        "resolver local symbol 'a' has typed behavior parents metadata, expected none",
        "resolver local symbol 'a' has behavior impls metadata, expected none",
        "resolver local symbol 'a' has typed behavior impls metadata, expected none",
        "resolver local symbol 'a' has behavior requires metadata, expected none",
        "resolver local symbol 'a' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver local metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}
