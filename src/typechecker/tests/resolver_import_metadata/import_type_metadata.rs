use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_import_absent_type_metadata() {
    let program = parse_program(
        r#"
{ io } = std
main = () i32 {
    0
}
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_parameter_names_for_test(Namespace::Import, "io", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Import, "io", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_type_parameter_names_for_test(Namespace::Import, "io", Some(vec!["T".to_string()]));
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Import,
        "io",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Import, "io", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Import, "io", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Import, "io", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Import,
        "io",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Import, "io", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Import,
        "io",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "StaticString".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Import,
        "io",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Import,
        "io",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver import type metadata should fail");

    for expected in [
        "resolver import symbol 'io' has parameter names metadata, expected none",
        "resolver import symbol 'io' has parameter types metadata, expected none",
        "resolver import symbol 'io' has typed parameter types metadata, expected none",
        "resolver import symbol 'io' has typed return type metadata, expected none",
        "resolver import symbol 'io' has type parameter count metadata, expected none",
        "resolver import symbol 'io' has type parameter names metadata, expected none",
        "resolver import symbol 'io' has type parameter bounds metadata, expected none",
        "resolver import symbol 'io' has typed type parameter bound refs metadata, expected none",
        "resolver import symbol 'io' has field count metadata, expected none",
        "resolver import symbol 'io' has field types metadata, expected none",
        "resolver import symbol 'io' has typed field types metadata, expected none",
        "resolver import symbol 'io' has variant names metadata, expected none",
        "resolver import symbol 'io' has variant owner metadata, expected none",
        "resolver import symbol 'io' has variant payload count metadata, expected none",
        "resolver import symbol 'io' has variant payload type metadata, expected none",
        "resolver import symbol 'io' has typed variant payload type metadata, expected none",
        "resolver import symbol 'io' has behavior methods metadata, expected none",
        "resolver import symbol 'io' has typed behavior methods metadata, expected none",
        "resolver import symbol 'io' has behavior parents metadata, expected none",
        "resolver import symbol 'io' has typed behavior parents metadata, expected none",
        "resolver import symbol 'io' has behavior impls metadata, expected none",
        "resolver import symbol 'io' has typed behavior impls metadata, expected none",
        "resolver import symbol 'io' has behavior requires metadata, expected none",
        "resolver import symbol 'io' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver import metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}
