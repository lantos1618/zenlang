use super::*;

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
