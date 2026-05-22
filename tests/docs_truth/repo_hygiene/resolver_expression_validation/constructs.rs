use super::*;

#[test]
fn resolver_aggregate_expression_validation_lives_in_focused_helper() {
    let constructs = read("src/resolver/expression_validation_constructs.rs");
    let aggregates = read("src/resolver/expression_validation_constructs/aggregate_literals.rs");

    for helper in [
        "StructLiteralRef",
        "EnumVariantRef",
        "validate_struct_literal_refs",
        "validate_enum_variant_refs",
    ] {
        assert!(
            !constructs.contains(&format!("struct {helper}"))
                && !constructs.contains(&format!("fn {helper}")),
            "general resolver expression constructs should not own aggregate helper: {helper}"
        );
        assert!(
            aggregates.contains(&format!("struct {helper}"))
                || aggregates.contains(&format!("fn {helper}")),
            "aggregate expression validation should live in focused helper: {helper}"
        );
    }

    assert!(
        constructs.contains("mod aggregate_literals;"),
        "resolver expression construct helpers should load aggregate literal validation"
    );
}

#[test]
fn resolver_scoped_construct_validation_lives_in_focused_helper() {
    let constructs = read("src/resolver/expression_validation_constructs.rs");
    let scoped = read("src/resolver/expression_validation_constructs/scoped_constructs.rs");

    assert!(
        constructs.lines().count() < 135,
        "general resolver expression constructs should stay focused on shared argument and match-arm traversal"
    );

    for helper in [
        "BlockRef",
        "ClosureRef",
        "validate_child_scope_expr_refs",
        "validate_block_refs",
        "validate_closure_refs",
    ] {
        assert!(
            !constructs.contains(&format!("struct {helper}"))
                && !constructs.contains(&format!("fn {helper}")),
            "general resolver expression constructs should not own scoped construct helper: {helper}"
        );
        assert!(
            scoped.contains(&format!("struct {helper}"))
                || scoped.contains(&format!("fn {helper}")),
            "scoped expression construct validation should live in focused helper: {helper}"
        );
    }

    assert!(
        constructs.contains("mod scoped_constructs;"),
        "resolver expression construct helpers should load scoped construct validation"
    );
}
