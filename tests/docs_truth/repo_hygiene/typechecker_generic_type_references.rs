use super::*;

#[test]
fn generic_type_reference_symbol_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/generic_type_reference_walker.rs");
    let type_refs = read("src/typechecker/generic_type_reference_walker/type_refs.rs");

    for helper in [
        "validate_named_type_ref_bounds",
        "validate_generic_type_ref_with_args",
        "is_known_named_type",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "generic_type_reference_walker.rs should not own type-symbol helper: {helper}"
        );
        assert!(
            type_refs.contains(&format!("fn {helper}")),
            "generic type-symbol validation should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("mod type_refs;"),
        "generic type-reference walker should load the focused type_refs helper"
    );
    assert!(
        root.lines().count() < 170,
        "generic_type_reference_walker.rs should stay focused on recursive AstType traversal"
    );
}
