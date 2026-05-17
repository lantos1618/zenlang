use super::*;

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
