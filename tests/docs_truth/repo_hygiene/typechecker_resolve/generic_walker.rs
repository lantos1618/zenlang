use super::*;

#[test]
fn generic_type_reference_walker_bounds_live_in_focused_helper() {
    let root = read("src/typechecker/generic_type_reference_walker.rs");
    let expressions = read("src/typechecker/generic_type_reference_walker/expressions.rs");
    let statements = read("src/typechecker/generic_type_reference_walker/statements.rs");
    let type_refs = read("src/typechecker/generic_type_reference_walker/type_refs.rs");
    let type_ref_bounds = read("src/typechecker/generic_type_reference_walker/type_ref_bounds.rs");

    assert!(
        root.lines().count() < 160,
        "generic_type_reference_walker.rs should stay focused on public traversal entry points"
    );
    assert!(
        root.contains("mod type_refs;"),
        "generic type-reference walker should include the focused type_refs helper"
    );
    assert!(
        !root.contains("fn validate_generic_type_ref_bounds_with_unknowns"),
        "recursive generic type-ref bound validation should live in type_refs.rs"
    );
    assert!(
        type_refs.contains("pub(super) fn validate_generic_type_ref_bounds_with_unknowns"),
        "type_refs.rs should own recursive generic type-ref bound validation"
    );
    assert!(
        type_refs.lines().count() < 90,
        "type_refs.rs should stay focused on recursive type-shape traversal"
    );
    for helper in [
        "validate_named_type_ref_bounds",
        "validate_parameterized_type_ref_bounds",
        "is_known_named_type",
    ] {
        assert!(
            !type_refs.contains(&format!("fn {helper}")),
            "recursive type-shape traversal should not own type-ref bound helper: {helper}"
        );
        assert!(
            type_ref_bounds.contains(&format!("fn {helper}")),
            "type_ref_bounds.rs should own type-ref bound helper: {helper}"
        );
    }
    assert!(
        root.contains("mod type_ref_bounds;"),
        "generic type-reference walker should include focused type-ref bounds helper"
    );
    assert!(
        expressions.lines().count() < 170,
        "generic expression type-reference traversal should not own statement traversal"
    );
    assert!(
        root.contains("mod statements;"),
        "generic type-reference walker should include the focused statements helper"
    );
    assert!(
        !expressions.contains("fn validate_generic_statement_type_references"),
        "generic expression type-reference traversal should not own statement traversal"
    );
    assert!(
        statements.contains("fn validate_generic_statement_type_references"),
        "statements.rs should own generic statement type-reference traversal"
    );
}
