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
