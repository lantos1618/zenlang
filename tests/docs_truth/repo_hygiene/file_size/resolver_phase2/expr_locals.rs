use super::*;

#[test]
fn resolver_phase2_expr_local_tests_live_in_focused_modules() {
    let root = read("tests/resolver_phase2/expr_locals.rs");
    let closures = read("tests/resolver_phase2/expr_locals/closures.rs");
    let local_symbols = read("tests/resolver_phase2/expr_locals/local_symbols.rs");
    let patterns = read("tests/resolver_phase2/expr_locals/patterns.rs");
    let value_references = read("tests/resolver_phase2/expr_locals/value_references.rs");
    let variant_expressions = read("tests/resolver_phase2/expr_locals/variant_expressions.rs");

    assert!(
        root.lines().count() < 60,
        "resolver_phase2 expr_locals.rs should only route focused expression-local modules"
    );
    for module in [
        r#"#[path = "expr_locals/closures.rs"]"#,
        r#"#[path = "expr_locals/local_symbols.rs"]"#,
        r#"#[path = "expr_locals/patterns.rs"]"#,
        r#"#[path = "expr_locals/value_references.rs"]"#,
        r#"#[path = "expr_locals/variant_expressions.rs"]"#,
    ] {
        assert!(
            root.contains(module),
            "resolver_phase2 expr_locals.rs should include focused module path `{module}`"
        );
    }

    for test_name in [
        "resolver_records_parameter_and_local_symbols",
        "resolver_records_top_level_expr_locals",
        "resolver_records_same_name_locals_in_distinct_scopes",
        "resolver_rejects_duplicate_bindings_in_same_scope",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "local symbol resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            local_symbols.contains(&format!("fn {test_name}")),
            "local_symbols.rs should keep local symbol resolver test: {test_name}"
        );
    }

    for test_name in [
        "resolver_rejects_unknown_unqualified_function_calls",
        "resolver_rejects_unknown_local_identifier_references",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "value reference resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            value_references.contains(&format!("fn {test_name}")),
            "value_references.rs should keep value reference resolver test: {test_name}"
        );
    }

    for test_name in [
        "resolver_rejects_unknown_enum_variant_expressions",
        "resolver_rejects_missing_enum_variant_payload_expressions",
        "resolver_rejects_unexpected_enum_variant_payload_expressions",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "variant expression resolver test should move out of expr_locals.rs: {test_name}"
        );
        assert!(
            variant_expressions.contains(&format!("fn {test_name}")),
            "variant_expressions.rs should keep variant expression resolver test: {test_name}"
        );
    }

    assert!(
        closures.contains("fn resolver_records_closure_locals"),
        "closures.rs should keep closure local resolver tests"
    );
    assert!(
        patterns.contains("fn resolver_records_pattern_locals"),
        "patterns.rs should keep pattern local resolver tests"
    );
}
