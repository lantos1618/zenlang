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
