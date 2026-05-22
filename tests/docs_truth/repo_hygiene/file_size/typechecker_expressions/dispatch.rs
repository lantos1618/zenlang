use super::*;

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
