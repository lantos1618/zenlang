use super::*;

#[test]
fn typechecker_resolver_entry_type_declaration_helpers_live_in_focused_helper() {
    let root = read("src/typechecker/resolver_validation.rs");
    let entry = read("src/typechecker/resolver_validation/entry_symbols.rs");
    let type_entries = read("src/typechecker/resolver_validation/entry_types.rs");

    assert!(
        !entry.contains("require_resolver_struct_symbol")
            && !entry.contains("require_resolver_enum_symbol")
            && !entry.contains("require_resolver_variant_symbol")
            && !entry.contains("require_resolver_behavior_symbol"),
        "resolver entry traversal should not own detailed type-declaration symbol validation"
    );
    assert!(
        entry.contains("self.validate_resolver_type_declaration_entry("),
        "resolver entry traversal should delegate type-declaration work through one routing helper"
    );

    for helper in [
        "validate_resolver_type_declaration_entry",
        "validate_resolver_struct_entry",
        "validate_resolver_enum_entry",
        "validate_resolver_behavior_entry",
    ] {
        assert!(
            !entry.contains(&format!("fn {helper}")),
            "resolver entry traversal should not own type-declaration helper: {helper}"
        );
        assert!(
            type_entries.contains(&format!("fn {helper}")),
            "resolver type-declaration entry helper should live in focused helper: {helper}"
        );
    }

    assert!(
        entry.lines().count() < 170,
        "resolver entry traversal should stay focused on declaration dispatch"
    );
    assert!(
        root.contains("include!(\"resolver_validation/entry_types.rs\");"),
        "resolver validation should include focused type-declaration entry helpers"
    );
}
