use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_module_symbols() {
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
    symbols.set_public_for_test(Namespace::Module, "std", true);
    symbols.set_import_source_for_test(Namespace::Module, "std", Some("other".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module metadata mismatch should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has visibility public, expected private")),
        "expected resolver module visibility diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has source 'other', expected none")),
        "expected resolver module source diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_declaration_metadata() {
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
    symbols.set_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_return_type_name_for_test(Namespace::Module, "std", Some("i32".to_string()));
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module declaration metadata should fail");

    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has parameter count metadata, expected none")),
        "expected resolver module parameter metadata diagnostic, got {err:?}"
    );
    assert!(
        err.iter().any(|d| d
            .message
            .contains("resolver module symbol 'std' has return type metadata, expected none")),
        "expected resolver module return metadata diagnostic, got {err:?}"
    );
}

#[test]
fn check_program_with_symbols_validates_resolver_module_absent_type_metadata() {
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
    symbols.set_parameter_names_for_test(Namespace::Module, "std", Some(vec!["x".to_string()]));
    symbols.set_parameter_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["i32".to_string()]),
    );
    symbols.set_parameter_types_for_test(Namespace::Module, "std", Some(vec![AstType::I32]));
    symbols.set_return_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_type_parameter_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_type_parameter_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["T".to_string()]),
    );
    symbols.set_type_parameter_bounds_for_test(
        Namespace::Module,
        "std",
        Some(vec![("T".to_string(), "Json".to_string())]),
    );
    symbols.set_type_parameter_bound_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![TypeParameterBoundRefMetadata {
            type_parameter: "T".to_string(),
            behavior: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_field_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    symbols.set_variant_names_for_test(Namespace::Module, "std", Some(vec!["Some".to_string()]));
    symbols.set_variant_owner_name_for_test(Namespace::Module, "std", Some("Option".to_string()));
    symbols.set_variant_payload_count_for_test(Namespace::Module, "std", Some(1));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Module,
        "std",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Module, "std", Some(AstType::I32));
    symbols.set_behavior_method_signatures_for_test(
        Namespace::Module,
        "std",
        Some(vec![(
            "encode".to_string(),
            vec!["Self".to_string()],
            "str".to_string(),
        )]),
    );
    symbols.set_behavior_method_types_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorMethodTypeMetadata {
            name: "encode".to_string(),
            parameter_names: vec!["self".to_string()],
            parameter_types: vec![AstType::SelfType],
            return_type: AstType::Str,
        }]),
    );
    symbols.set_behavior_parent_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_parent_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_impl_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_impl_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    symbols.set_behavior_required_names_for_test(
        Namespace::Module,
        "std",
        Some(vec!["Json".to_string()]),
    );
    symbols.set_behavior_required_refs_for_test(
        Namespace::Module,
        "std",
        Some(vec![BehaviorRefMetadata {
            name: "Json".to_string(),
            type_args: Vec::new(),
        }]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver module type metadata should fail");

    for expected in [
        "resolver module symbol 'std' has parameter names metadata, expected none",
        "resolver module symbol 'std' has parameter types metadata, expected none",
        "resolver module symbol 'std' has typed parameter types metadata, expected none",
        "resolver module symbol 'std' has typed return type metadata, expected none",
        "resolver module symbol 'std' has type parameter count metadata, expected none",
        "resolver module symbol 'std' has type parameter names metadata, expected none",
        "resolver module symbol 'std' has type parameter bounds metadata, expected none",
        "resolver module symbol 'std' has typed type parameter bound refs metadata, expected none",
        "resolver module symbol 'std' has field count metadata, expected none",
        "resolver module symbol 'std' has field types metadata, expected none",
        "resolver module symbol 'std' has typed field types metadata, expected none",
        "resolver module symbol 'std' has variant names metadata, expected none",
        "resolver module symbol 'std' has variant owner metadata, expected none",
        "resolver module symbol 'std' has variant payload count metadata, expected none",
        "resolver module symbol 'std' has variant payload type metadata, expected none",
        "resolver module symbol 'std' has typed variant payload type metadata, expected none",
        "resolver module symbol 'std' has behavior methods metadata, expected none",
        "resolver module symbol 'std' has typed behavior methods metadata, expected none",
        "resolver module symbol 'std' has behavior parents metadata, expected none",
        "resolver module symbol 'std' has typed behavior parents metadata, expected none",
        "resolver module symbol 'std' has behavior impls metadata, expected none",
        "resolver module symbol 'std' has typed behavior impls metadata, expected none",
        "resolver module symbol 'std' has behavior requires metadata, expected none",
        "resolver module symbol 'std' has typed behavior requires metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver module metadata diagnostic `{expected}`, got {err:?}"
        );
    }
}
