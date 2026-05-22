use super::*;

#[test]
fn expected_local_traversal_support_stays_split_by_responsibility() {
    let root = read("src/typechecker/resolver_validation_support/expected_local_traversal.rs");
    let expressions =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/expressions.rs");
    let statements =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/statements.rs");
    let patterns =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/patterns.rs");
    let bindings =
        read("src/typechecker/resolver_validation_support/expected_local_traversal/bindings.rs");

    assert!(
        root.lines().count() < 80,
        "expected_local_traversal.rs should only include focused traversal helpers"
    );
    for include in [
        "include!(\"expected_local_traversal/bindings.rs\");",
        "include!(\"expected_local_traversal/expressions.rs\");",
        "include!(\"expected_local_traversal/patterns.rs\");",
        "include!(\"expected_local_traversal/statements.rs\");",
    ] {
        assert!(
            root.contains(include),
            "expected local traversal root should include focused helper: {include}"
        );
    }
    assert!(
        !root.contains("fn expected_resolver_statement_locals"),
        "expected local traversal root should not own statement traversal bodies"
    );
    assert!(
        expressions.contains("fn expected_resolver_expr_locals"),
        "expressions.rs should cover expression traversal"
    );
    assert!(
        statements.contains("fn expected_resolver_statement_locals"),
        "statements.rs should cover statement traversal"
    );
    assert!(
        patterns.contains("fn expected_resolver_pattern_locals"),
        "patterns.rs should cover pattern traversal"
    );
    assert!(
        bindings.contains("fn expected_resolver_local"),
        "bindings.rs should cover shared local binding helpers"
    );
}
