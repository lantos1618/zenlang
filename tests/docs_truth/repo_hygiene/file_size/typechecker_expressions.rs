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
