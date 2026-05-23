use super::super::*;

#[test]
fn phase5_generic_diagnostics_pin_codes_in_unit_tests() {
    let generic_diagnostics = read("tests/generic_diagnostics.rs");
    let function_inference =
        read("tests/generic_diagnostics/inference_conflicts/functions/basic.rs");
    let function_compound_inference =
        read("tests/generic_diagnostics/inference_conflicts/functions/compound_params.rs");
    let function_generic_param_inference =
        read("tests/generic_diagnostics/inference_conflicts/functions/generic_params.rs");
    let method_receiver_inference =
        read("tests/generic_diagnostics/inference_conflicts/methods/receiver.rs");
    let method_compound_inference =
        read("tests/generic_diagnostics/inference_conflicts/methods/compound_params.rs");
    let method_generic_param_inference =
        read("tests/generic_diagnostics/inference_conflicts/methods/generic_params.rs");
    let method_type_args = read("tests/generic_diagnostics/method_type_args.rs");
    let method_type_arg_enum_methods =
        read("tests/generic_diagnostics/method_type_args/enum_methods.rs");
    let method_type_arg_followups =
        read("tests/generic_diagnostics/method_type_args/arity_followups.rs");
    let type_bounds = read("tests/generic_diagnostics/bounds.rs");
    let function_bounds = read("tests/generic_diagnostics/call_site_bounds.rs");
    let method_bounds = read("tests/generic_diagnostics/call_site_bounds/methods.rs");
    let generic_bound_validation = read("src/typechecker/generic_bound_validation.rs");

    assert!(
        generic_diagnostics.contains("fn assert_diagnostic_code_and_message("),
        "generic diagnostics tests should have a focused helper for checking code plus message"
    );
    assert!(
        generic_diagnostics.contains("fn assert_inference_conflict("),
        "generic diagnostics tests should have a focused helper for checking inference conflict code plus message"
    );
    assert!(
        generic_diagnostics.contains("fn assert_generic_arity_diagnostic("),
        "generic diagnostics tests should have a focused helper for checking generic arity code plus message"
    );
    assert!(
        generic_diagnostics.contains("fn assert_nongeneric_type_args_diagnostic("),
        "generic diagnostics tests should have a focused helper for checking non-generic type-argument code plus message"
    );

    assert_inference_helper_pins_code(&generic_diagnostics);
    assert_arity_helper_pins_code(&generic_diagnostics);
    assert_nongeneric_type_args_helper_pins_code(&generic_diagnostics);
    assert_inference_source_uses_helper(&function_inference);
    assert_inference_source_uses_helper(&function_compound_inference);
    assert_inference_source_uses_helper(&function_generic_param_inference);
    assert_inference_source_uses_helper(&method_receiver_inference);
    assert_inference_source_uses_helper(&method_compound_inference);
    assert_inference_source_uses_helper(&method_generic_param_inference);
    assert_arity_source_uses_helper(&method_type_args);
    assert_arity_source_uses_helper(&method_type_arg_enum_methods);
    assert_arity_source_uses_helper(&method_type_arg_followups);
    assert_nongeneric_type_args_source_uses_helper(&method_type_args);
    assert_source_pins_code(&type_bounds, "E6004");
    assert_source_pins_code(&function_bounds, "E6004");
    assert_source_pins_code(&method_bounds, "E6004");

    assert!(
        !generic_bound_validation.contains("E6012"),
        "generic behavior-bound arity should use public arity code E5001, not stale internal code E6012"
    );
}

fn assert_inference_source_uses_helper(source: &str) {
    assert!(
        source.contains("assert_inference_conflict("),
        "Phase 5 inference conflict diagnostics should use the E5000 helper"
    );
}

fn assert_inference_helper_pins_code(source: &str) {
    let normalized = source.split_whitespace().collect::<String>();
    assert!(
        normalized.contains(r#"assert_diagnostic_code_and_message(errors,"E5000""#),
        "Phase 5 inference conflict helper should pin diagnostic code E5000"
    );
}

fn assert_arity_source_uses_helper(source: &str) {
    assert!(
        source.contains("assert_generic_arity_diagnostic("),
        "Phase 5 generic arity diagnostics should use the E5001 helper"
    );
}

fn assert_arity_helper_pins_code(source: &str) {
    let normalized = source.split_whitespace().collect::<String>();
    assert!(
        normalized.contains(r#"assert_diagnostic_code_and_message(errors,"E5001""#),
        "Phase 5 generic arity helper should pin diagnostic code E5001"
    );
}

fn assert_nongeneric_type_args_source_uses_helper(source: &str) {
    assert!(
        source.contains("assert_nongeneric_type_args_diagnostic("),
        "Phase 5 non-generic type-argument diagnostics should use the E5002 helper"
    );
}

fn assert_nongeneric_type_args_helper_pins_code(source: &str) {
    let normalized = source.split_whitespace().collect::<String>();
    assert!(
        normalized.contains(r#"assert_diagnostic_code_and_message(errors,"E5002""#),
        "Phase 5 non-generic type-argument helper should pin diagnostic code E5002"
    );
}

fn assert_source_pins_code(source: &str, code: &str) {
    let normalized = source.split_whitespace().collect::<String>();
    assert!(
        normalized.contains(&format!(
            r#"assert_diagnostic_code_and_message(&errors,"{code}""#
        )),
        "Phase 5 generic diagnostics unit tests should pin diagnostic code {code}"
    );
}
