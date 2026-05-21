use super::*;

#[test]
fn typechecker_generic_call_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let call_validation = read("src/typechecker/expressions/call_validation.rs");
    let generic_call_validation = read("src/typechecker/expressions/generic_call_validation.rs");
    let type_args = read("src/typechecker/expressions/generic_call_validation/type_args.rs");

    for helper in [
        "resolve_generic_function_call",
        "check_call_signature_with_substitutions",
        "report_inference_conflicts",
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

    for helper in [
        "explicit_type_arg_substitutions",
        "explicit_type_args_valid",
        "generic_type_annotation_arities_valid",
    ] {
        assert!(
            !generic_call_validation.contains(&format!("fn {helper}")),
            "generic call resolver should not own generic type-argument helper: {helper}"
        );
        assert!(
            type_args.contains(&format!("fn {helper}")),
            "generic type-argument helper should live in type_args.rs: {helper}"
        );
    }

    assert!(
        generic_call_validation.lines().count() < 150,
        "generic_call_validation.rs should stay focused on generic call resolution and shared diagnostics"
    );
    assert!(
        generic_call_validation.contains("mod type_args;"),
        "generic call validation should include focused type-argument helper"
    );
    assert!(
        root.contains("mod generic_call_validation;"),
        "expression checker root should include focused generic call validation module"
    );
}
