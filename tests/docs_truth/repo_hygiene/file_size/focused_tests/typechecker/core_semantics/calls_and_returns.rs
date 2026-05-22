use super::*;

#[test]
fn call_and_return_tests_stay_split_by_semantic_surface() {
    let root = read("src/typechecker/tests/core_semantics/calls_and_returns.rs");
    let function_calls =
        read("src/typechecker/tests/core_semantics/calls_and_returns/function_calls.rs");
    let returns = read("src/typechecker/tests/core_semantics/calls_and_returns/returns.rs");

    assert!(
        root.lines().count() < 80,
        "calls_and_returns.rs should only route focused call and return tests"
    );
    for module in ["mod function_calls;", "mod returns;"] {
        assert!(
            root.contains(module),
            "calls_and_returns.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "fn unknown_function_error",
        "fn function_call_wrong_arity_is_error",
        "fn return_type_mismatch_error",
    ] {
        assert!(
            !root.contains(test_name),
            "concrete call/return test `{test_name}` should live in a focused child module"
        );
    }
    assert!(
        function_calls.contains("fn unknown_function_error"),
        "function_calls.rs should cover unknown function diagnostics"
    );
    assert!(
        function_calls.contains("fn function_call_wrong_arity_is_error"),
        "function_calls.rs should cover function-call arity diagnostics"
    );
    assert!(
        function_calls.contains("fn function_call_argument_type_mismatch_is_error"),
        "function_calls.rs should cover function-call argument type diagnostics"
    );
    assert!(
        returns.contains("fn return_type_mismatch_error"),
        "returns.rs should cover return type diagnostics"
    );
}
