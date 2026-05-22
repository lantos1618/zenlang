use super::*;

#[test]
fn typechecker_resolver_entry_local_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let locals = read("src/typechecker/resolver_validation/entry_locals.rs");

    for helper in [
        "require_resolver_callable_locals",
        "require_resolver_scoped_expr_locals",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own local helper: {helper}"
        );
        assert!(
            locals.contains(&format!("fn {helper}")),
            "resolver entry local helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 260,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_locals.rs\");"),
        "resolver validation should include focused entry-local helpers"
    );
}

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
