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
fn typechecker_resolver_behavior_association_entries_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let behavior_entries =
        read("src/typechecker/resolver_validation/entry_behavior_associations.rs");

    for helper in [
        "validate_resolver_impl_block_entry_symbols",
        "validate_resolver_requires_entry_symbols",
        "validate_resolver_behavior_extends_entry_symbols",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own behavior-association helper: {helper}"
        );
        assert!(
            behavior_entries.contains(&format!("fn {helper}")),
            "resolver behavior-association entry helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 220,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_behavior_associations.rs\");"),
        "resolver validation should include focused behavior-association entry helpers"
    );
}
