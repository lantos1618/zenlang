use super::super::*;

#[test]
fn typechecker_closure_expression_checking_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let simple_forms = read("src/typechecker/expressions/simple_forms.rs");
    let closures = read("src/typechecker/expressions/closure_forms.rs");

    assert!(
        !simple_forms.contains("fn check_closure_expr"),
        "simple_forms.rs should not own closure expression checking"
    );
    assert!(
        closures.contains("fn check_closure_expr"),
        "closure_forms.rs should own closure expression checking"
    );
    assert!(
        simple_forms.lines().count() < 180,
        "simple_forms.rs should stay focused on scalar/block/string/defer expression forms"
    );
    assert!(
        root.contains("mod closure_forms;"),
        "expression checking module should include focused closure expression helper"
    );
}

#[test]
fn typechecker_expression_dispatch_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let dispatch = read("src/typechecker/expressions/dispatch.rs");

    assert!(
        root.lines().count() < 80,
        "expressions.rs should stay focused on module wiring and shared imports"
    );
    assert!(
        root.contains("mod dispatch;"),
        "expression checking module should include focused dispatch helper"
    );
    assert!(
        !root.contains("pub(crate) fn check_expr"),
        "top-level expression module should not own expression dispatch"
    );
    assert!(
        dispatch.contains("pub(crate) fn check_expr"),
        "dispatch.rs should own expression dispatch"
    );
    assert!(
        dispatch.contains("Expression::FunctionCall"),
        "dispatch.rs should route expression variants"
    );
}

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

#[test]
fn typechecker_generic_method_resolution_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions/method_call_support.rs");
    let generic_methods =
        read("src/typechecker/expressions/method_call_support/generic_methods.rs");

    assert!(
        root.lines().count() < 170,
        "method_call_support.rs should stay focused on method dispatch and UFC routing"
    );
    assert!(
        root.contains("mod generic_methods;"),
        "method call support should include focused generic method resolution helper"
    );
    assert!(
        !root.contains("fn resolve_generic_method_call"),
        "method-call dispatch should not own generic method specialization resolution"
    );
    assert!(
        generic_methods.contains("fn resolve_generic_method_call"),
        "generic method specialization resolution should live in focused helper"
    );
    assert!(
        generic_methods.contains("infer_method_type_args")
            && generic_methods.contains("specialize_generic_method"),
        "generic method helper should own inference and specialization flow"
    );
}

#[test]
fn typechecker_return_flow_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/expressions.rs");
    let call_validation = read("src/typechecker/expressions/call_validation.rs");
    let return_flow = read("src/typechecker/expressions/return_flow.rs");

    assert!(
        call_validation.lines().count() < 190,
        "call_validation.rs should stay focused on call and method validation"
    );
    for helper in [
        "fn block_satisfies_return",
        "fn block_definitely_returns",
        "fn expr_definitely_returns",
    ] {
        assert!(
            !call_validation.contains(helper),
            "call validation should not own return-flow helper: {helper}"
        );
        assert!(
            return_flow.contains(helper),
            "return_flow.rs should own return-flow helper: {helper}"
        );
    }
    assert!(
        root.contains("mod return_flow;"),
        "expression checking module should include focused return-flow helper"
    );
}
