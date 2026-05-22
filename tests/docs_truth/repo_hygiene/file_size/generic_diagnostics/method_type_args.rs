use super::*;

#[test]
fn generic_method_type_arg_tests_stay_split_by_call_surface() {
    let root = read("tests/generic_diagnostics/method_type_args.rs");
    let call_kinds = read("tests/generic_diagnostics/method_type_args/call_kinds.rs");
    let generic_methods = read("tests/generic_diagnostics/method_type_args/generic_methods.rs");
    let arity_followups = read("tests/generic_diagnostics/method_type_args/arity_followups.rs");
    let enum_methods = read("tests/generic_diagnostics/method_type_args/enum_methods.rs");

    assert!(
        root.lines().count() < 60,
        "method_type_args.rs should route focused generic method type-arg modules"
    );
    assert!(
        !root.contains("#[test]"),
        "method_type_args.rs should not own concrete generic method type-arg tests"
    );
    for module in [
        r#"#[path = "method_type_args/call_kinds.rs"]"#,
        r#"#[path = "method_type_args/generic_methods.rs"]"#,
        r#"#[path = "method_type_args/arity_followups.rs"]"#,
        r#"#[path = "method_type_args/enum_methods.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "method_type_args.rs should include focused module path `{module}`"
        );
    }

    assert!(
        call_kinds.contains("fn nongeneric_method_explicit_type_args_are_error"),
        "call_kinds.rs should cover non-generic method type-arg diagnostics"
    );
    assert!(
        call_kinds.contains("fn builtin_function_explicit_type_args_are_error"),
        "call_kinds.rs should cover builtin function type-arg diagnostics"
    );
    assert!(
        generic_methods.contains("fn generic_method_explicit_type_arg_arity_is_error"),
        "generic_methods.rs should cover generic method explicit type-arg arity"
    );
    assert!(
        generic_methods.contains("fn generic_method_inference_failure_is_error"),
        "generic_methods.rs should cover generic method inference diagnostics"
    );
    assert!(
        arity_followups
            .contains("fn generic_method_explicit_type_arg_arity_does_not_emit_inference_followup"),
        "arity_followups.rs should keep generic method followup suppression diagnostics"
    );
    assert!(
        enum_methods.contains("fn generic_result_enum_method_explicit_type_arg_arity_is_error"),
        "enum_methods.rs should keep generic enum method type-arg diagnostics"
    );
}
