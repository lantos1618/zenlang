use super::*;

#[test]
fn resolver_expression_traversal_lives_in_focused_helper() {
    let validation = read("src/resolver/expression_validation.rs");
    let traversal = read("src/resolver/expression_validation/traversal.rs");

    for helper in [
        "validate_binary_expr_refs",
        "validate_unary_expr_refs",
        "validate_index_expr_refs",
        "validate_string_interpolation_refs",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "resolver expression dispatch should not own traversal helper: {helper}"
        );
        assert!(
            traversal.contains(&format!("fn {helper}")),
            "resolver expression traversal should live in focused helper: {helper}"
        );
    }

    assert!(
        validation.contains("mod traversal;"),
        "resolver expression validation should load focused traversal validation"
    );
}

#[test]
fn resolver_control_flow_traversal_lives_in_focused_helper() {
    let traversal = read("src/resolver/expression_validation/traversal.rs");
    let control_flow = read("src/resolver/expression_validation/traversal/control_flow.rs");

    for helper in [
        "IfOrWhileExprRef",
        "RangeExprRef",
        "validate_if_or_while_expr_refs",
        "validate_range_expr_refs",
        "validate_defer_expr_refs",
    ] {
        assert!(
            !traversal.contains(&format!("struct {helper}"))
                && !traversal.contains(&format!("fn {helper}")),
            "general traversal helper should not own control-flow traversal helper: {helper}"
        );
        assert!(
            control_flow.contains(&format!("struct {helper}"))
                || control_flow.contains(&format!("fn {helper}")),
            "control-flow traversal helper should live in focused helper: {helper}"
        );
    }

    assert!(
        traversal.contains("mod control_flow;"),
        "resolver expression traversal should include focused control-flow traversal helper"
    );
    assert!(
        traversal.lines().count() < 145,
        "resolver expression traversal should stay focused on direct child traversal"
    );
}
