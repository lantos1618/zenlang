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
