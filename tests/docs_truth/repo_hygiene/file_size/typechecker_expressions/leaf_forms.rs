use super::*;

#[test]
fn typechecker_leaf_expression_forms_live_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let dispatch = read("src/typechecker/expressions/dispatch.rs");
    let leaf_forms = read("src/typechecker/expressions/leaf_forms.rs");

    for helper in [
        "fn check_int_literal_expr",
        "fn check_float_literal_expr",
        "fn check_static_string_literal_expr",
        "fn check_bool_literal_expr",
        "fn check_binary_expr",
        "fn check_break_expr",
        "fn check_continue_expr",
        "fn check_loop_control_expr",
        "fn check_unary_expr",
        "fn check_range_expr",
        "fn check_error_expr",
    ] {
        assert!(
            !dispatch.contains(helper),
            "expression dispatch should not own leaf expression helper: {helper}"
        );
        assert!(
            leaf_forms.contains(helper),
            "leaf expression helper should live in focused helper: {helper}"
        );
    }

    for typed_leaf in [
        "TypedExprKind::IntLiteral",
        "TypedExprKind::FloatLiteral",
        "TypedExprKind::StringLiteral",
        "TypedExprKind::BoolLiteral",
        "TypedExprKind::BinaryOp",
        "TypedExprKind::Break",
        "TypedExprKind::Continue",
        "TypedExprKind::LoopControl",
        "TypedExprKind::UnaryOp",
        "TypedExprKind::Error",
    ] {
        assert!(
            !dispatch.contains(typed_leaf),
            "expression dispatch should route leaf forms instead of constructing: {typed_leaf}"
        );
        assert!(
            leaf_forms.contains(typed_leaf),
            "leaf expression helper should construct typed leaf form: {typed_leaf}"
        );
    }

    assert!(
        root.contains("mod leaf_forms;"),
        "expression checking module should include focused leaf expression helper"
    );
    assert!(
        dispatch.lines().count() < 150,
        "dispatch.rs should stay focused on expression routing"
    );
}
