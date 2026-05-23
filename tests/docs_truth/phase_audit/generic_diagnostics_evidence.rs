use super::super::*;

#[test]
fn phase5_generic_diagnostics_pin_codes_in_unit_tests() {
    let generic_diagnostics = read("tests/generic_diagnostics.rs");
    let function_inference =
        read("tests/generic_diagnostics/inference_conflicts/functions/basic.rs");
    let method_type_args = read("tests/generic_diagnostics/method_type_args.rs");
    let method_bounds = read("tests/generic_diagnostics/call_site_bounds/methods.rs");
    let generic_bound_validation = read("src/typechecker/generic_bound_validation.rs");

    assert!(
        generic_diagnostics.contains("fn assert_diagnostic_code_and_message("),
        "generic diagnostics tests should have a focused helper for checking code plus message"
    );

    for (source, code) in [
        (function_inference.as_str(), "E5000"),
        (method_type_args.as_str(), "E5001"),
        (method_type_args.as_str(), "E5002"),
        (method_bounds.as_str(), "E6004"),
    ] {
        let normalized = source.split_whitespace().collect::<String>();
        assert!(
            normalized.contains(&format!(
                r#"assert_diagnostic_code_and_message(&errors,"{code}""#
            )),
            "Phase 5 generic diagnostics unit tests should pin diagnostic code {code}"
        );
    }

    assert!(
        !generic_bound_validation.contains("E6012"),
        "generic behavior-bound arity should use public arity code E5001, not stale internal code E6012"
    );
}
