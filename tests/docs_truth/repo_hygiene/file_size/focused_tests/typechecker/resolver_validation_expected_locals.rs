use super::*;

#[test]
fn resolver_validation_expected_local_tests_stay_split_by_traversal_surface() {
    let root = read("src/typechecker/tests/resolver_validation/expected_locals.rs");
    let callables = read("src/typechecker/tests/resolver_validation/expected_locals/callables.rs");
    let closures = read("src/typechecker/tests/resolver_validation/expected_locals/closures.rs");
    let patterns = read("src/typechecker/tests/resolver_validation/expected_locals/patterns.rs");
    let scoped_exprs =
        read("src/typechecker/tests/resolver_validation/expected_locals/scoped_exprs.rs");
    let statements =
        read("src/typechecker/tests/resolver_validation/expected_locals/statements.rs");

    assert!(
        root.lines().count() < 80,
        "expected_locals.rs should only route focused expected-local test modules"
    );
    for module in [
        "mod callables;",
        "mod closures;",
        "mod patterns;",
        "mod scoped_exprs;",
        "mod statements;",
    ] {
        assert!(
            root.contains(module),
            "expected_locals.rs should include focused module `{module}`"
        );
    }

    assert_expected_local_tests_live_in(
        &root,
        &callables,
        &[
            "expected_resolver_impl_method_symbols_collect_value_symbols_and_locals",
            "expected_resolver_callable_locals_collect_params_and_body",
        ],
        "callables.rs",
    );
    assert_expected_local_tests_live_in(
        &root,
        &scoped_exprs,
        &[
            "expected_resolver_scoped_expr_locals_collects_block_bindings",
            "expected_resolver_child_expr_locals_collects_branch_bindings",
            "expected_resolver_block_locals_collects_statement_and_final_expr_bindings",
        ],
        "scoped_exprs.rs",
    );
    assert_expected_local_tests_live_in(
        &root,
        &statements,
        &["expected_resolver_statement_locals_preserve_mutable_handoff"],
        "statements.rs",
    );
    assert_expected_local_tests_live_in(
        &root,
        &closures,
        &["expected_resolver_closure_locals_collects_params_and_body_bindings"],
        "closures.rs",
    );
    assert_expected_local_tests_live_in(
        &root,
        &patterns,
        &[
            "expected_resolver_pattern_expr_locals_collects_pattern_and_body_bindings",
            "expected_resolver_pattern_locals_collects_struct_shorthand_bindings",
        ],
        "patterns.rs",
    );
}

fn assert_expected_local_tests_live_in(
    root: &str,
    focused_module: &str,
    test_names: &[&str],
    focused_path: &str,
) {
    for test_name in test_names {
        let fn_name = format!("fn {test_name}");
        assert!(
            !root.contains(&fn_name),
            "expected-local helper test should move out of the root module: {test_name}"
        );
        assert!(
            focused_module.contains(&fn_name),
            "{focused_path} should keep expected-local helper test: {test_name}"
        );
    }
}
