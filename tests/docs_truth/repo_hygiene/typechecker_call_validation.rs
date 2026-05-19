use super::*;

#[test]
fn typechecker_generic_call_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let call_validation = read("src/typechecker/expressions/call_validation.rs");
    let generic_call_validation = read("src/typechecker/expressions/generic_call_validation.rs");

    for helper in [
        "resolve_generic_function_call",
        "check_call_signature_with_substitutions",
        "report_inference_conflicts",
        "explicit_type_arg_substitutions",
        "generic_type_annotation_arities_valid",
    ] {
        assert!(
            !call_validation.contains(&format!("fn {helper}")),
            "call_validation.rs should not own generic call helper: {helper}"
        );
        assert!(
            generic_call_validation.contains(&format!("fn {helper}")),
            "generic call helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("mod generic_call_validation;"),
        "expression checker root should include focused generic call validation module"
    );
}
