use super::*;

#[test]
fn resolver_call_expression_validation_lives_in_focused_helper() {
    let validation = read("src/resolver/expression_validation.rs");
    let calls = read("src/resolver/expression_validation/calls.rs");

    for helper in [
        "validate_function_call_expr_refs",
        "validate_identifier_expr_refs",
        "validate_method_call_expr_refs",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "resolver expression dispatch should not own call helper: {helper}"
        );
        assert!(
            calls.contains(&format!("fn {helper}")),
            "resolver call expression validation should live in focused helper: {helper}"
        );
    }

    assert!(
        validation.contains("mod calls;"),
        "resolver expression validation should load focused call validation"
    );
}
