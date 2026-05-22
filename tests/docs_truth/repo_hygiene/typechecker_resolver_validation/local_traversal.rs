use super::*;

#[test]
fn typechecker_resolver_statement_local_traversal_lives_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let traversal = read("src/typechecker/resolver_validation/local_traversal.rs");
    let statements = read("src/typechecker/resolver_validation/statement_locals.rs");

    for helper in [
        "require_resolver_statement_locals",
        "require_resolver_var_decl_local",
    ] {
        assert!(
            !traversal.contains(&format!("fn {helper}")),
            "resolver expression traversal should not own statement-local helper: {helper}"
        );
        assert!(
            statements.contains(&format!("fn {helper}")),
            "resolver statement-local traversal should live in focused helper: {helper}"
        );
    }

    assert!(
        traversal.lines().count() < 210,
        "resolver local traversal should stay focused on expression and child-scope traversal"
    );
    assert!(
        root.contains("include!(\"resolver_validation/statement_locals.rs\");"),
        "resolver validation should include focused statement-local traversal"
    );
}

#[test]
fn typechecker_resolver_expr_local_traversal_stays_split_by_surface() {
    let root = read("src/typechecker/resolver_validation/local_traversal.rs");
    let scopes = read("src/typechecker/resolver_validation/local_traversal/scopes.rs");
    let expressions = read("src/typechecker/resolver_validation/local_traversal/expressions.rs");

    assert!(
        root.lines().count() < 40,
        "resolver local traversal root should only route focused traversal helpers"
    );
    for include in [
        "include!(\"local_traversal/scopes.rs\");",
        "include!(\"local_traversal/expressions.rs\");",
    ] {
        assert!(
            root.contains(include),
            "resolver local traversal should include focused helper: {include}"
        );
    }
    assert!(
        !root.contains("fn require_resolver_expr_locals"),
        "resolver local traversal root should not own expression traversal bodies"
    );
    assert!(
        scopes.contains("fn require_resolver_child_expr_locals"),
        "child-scope traversal helpers should live in local_traversal/scopes.rs"
    );
    assert!(
        expressions.contains("fn require_resolver_expr_locals"),
        "expression traversal should live in local_traversal/expressions.rs"
    );
}
