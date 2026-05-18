use super::*;

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
            "StaticString".to_string(),
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
