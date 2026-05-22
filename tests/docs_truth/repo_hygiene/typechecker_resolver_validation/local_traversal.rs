use super::*;

#[test]
fn typechecker_resolver_pattern_local_traversal_lives_in_focused_helper() {
    let traversal = read("src/typechecker/resolver_validation/local_traversal.rs");
    let patterns = read("src/typechecker/resolver_validation/pattern_locals.rs");

    for helper in [
        "require_resolver_pattern_expr_locals",
        "require_resolver_pattern_locals",
        "require_resolver_pattern_binding",
    ] {
        assert!(
            !traversal.contains(&format!("fn {helper}")),
            "resolver local traversal should not own pattern-local helper: {helper}"
        );
        assert!(
            patterns.contains(&format!("fn {helper}")),
            "resolver pattern-local traversal should live in focused helper: {helper}"
        );
    }

    let root = read("src/typechecker/resolver_validation.rs");
    assert!(
        root.contains("include!(\"resolver_validation/pattern_locals.rs\");"),
        "resolver validation should include focused pattern-local traversal"
    );
}

#[test]
fn typechecker_resolver_local_scope_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let traversal = read("src/typechecker/resolver_validation/local_traversal.rs");
    let helpers = read("src/typechecker/resolver_validation/local_scope_helpers.rs");

    for helper in [
        "require_resolver_parameter_locals",
        "require_resolver_child_expr_locals",
        "require_resolver_block_locals",
        "require_resolver_closure_locals",
        "require_resolver_var_decl_local",
    ] {
        assert!(
            !traversal.contains(&format!("fn {helper}")),
            "resolver local traversal should not own local-scope helper: {helper}"
        );
        assert!(
            helpers.contains(&format!("fn {helper}")),
            "resolver local-scope helper should live in focused helper: {helper}"
        );
    }

    assert!(
        traversal.lines().count() < 190,
        "resolver local traversal should stay focused on expression and statement dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/local_scope_helpers.rs\");"),
        "resolver validation should include focused local-scope helpers"
    );
}
