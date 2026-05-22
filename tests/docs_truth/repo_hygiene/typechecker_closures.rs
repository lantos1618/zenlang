use super::*;

#[test]
fn typechecker_closure_expression_checking_lives_with_closure_analysis() {
    let simple_forms = read("src/typechecker/expressions/simple_forms.rs");
    let closures = read("src/typechecker/closures.rs");
    let expressions = read("src/typechecker/expressions.rs");

    assert!(
        !simple_forms.contains("fn check_closure_expr("),
        "simple_forms.rs should not own closure expression checking"
    );
    assert!(
        closures.contains("fn check_closure_expr("),
        "closure expression checking should live with closure capture analysis"
    );
    assert!(
        !expressions.contains("use super::closures::collect_captures;"),
        "expressions.rs should not import closure capture analysis for simple forms"
    );
    assert!(
        simple_forms.lines().count() < 200,
        "simple_forms.rs should stay focused on non-closure simple expression forms"
    );
}
