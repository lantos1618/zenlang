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
fn resolver_call_expression_validation_lives_in_focused_helper() {
    let validation = read("src/resolver/expression_validation.rs");
    let calls = read("src/resolver/expression_validation/calls.rs");

    for helper in [
        "validate_function_call_expr_refs",
        "validate_identifier_expr_refs",
        "validate_method_call_expr_refs",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "resolver expression dispatch should not own call helper: {helper}"
        );
        assert!(
            calls.contains(&format!("fn {helper}")),
            "resolver call expression validation should live in focused helper: {helper}"
        );
    }

    assert!(
        validation.contains("mod calls;"),
        "resolver expression validation should load focused call validation"
    );
}

#[test]
fn resolver_expression_traversal_lives_in_focused_helper() {
    let validation = read("src/resolver/expression_validation.rs");
    let traversal = read("src/resolver/expression_validation/traversal.rs");

    for helper in [
        "validate_binary_expr_refs",
        "validate_unary_expr_refs",
        "validate_index_expr_refs",
        "validate_if_or_while_expr_refs",
        "validate_string_interpolation_refs",
        "validate_range_expr_refs",
        "validate_defer_expr_refs",
    ] {
        assert!(
            !validation.contains(&format!("fn {helper}")),
            "resolver expression dispatch should not own traversal helper: {helper}"
        );
        assert!(
            traversal.contains(&format!("fn {helper}")),
            "resolver expression traversal should live in focused helper: {helper}"
        );
    }

    assert!(
        validation.contains("mod traversal;"),
        "resolver expression validation should load focused traversal validation"
    );
}

#[test]
fn resolver_expression_dispatch_stays_as_category_router() {
    let validation = read("src/resolver/expression_validation.rs");
    let dispatch = read("src/resolver/expression_validation/dispatch.rs");
    let construct_dispatch = read("src/resolver/expression_validation/construct_dispatch.rs");
    let calls = read("src/resolver/expression_validation/calls.rs");
    let traversal = read("src/resolver/expression_validation/traversal.rs");
    let constructs = read("src/resolver/expression_validation_constructs.rs");

    assert!(
        validation.lines().count() < 180,
        "resolver expression validation should stay a compact dispatch router"
    );
    assert!(
        validation.contains("mod dispatch;"),
        "resolver expression validation should load focused expression dispatch"
    );
    assert!(
        validation.contains("mod construct_dispatch;"),
        "resolver expression validation should load focused construct dispatch"
    );
    assert!(
        dispatch.lines().count() < 220,
        "resolver expression category dispatch should stay compact"
    );
    assert!(
        construct_dispatch.lines().count() < 140,
        "resolver construct expression dispatch should stay compact"
    );
    assert!(
        dispatch.contains("fn validate_call_expr_refs"),
        "dispatch.rs should route call-like expressions through calls.rs"
    );
    assert!(
        dispatch.contains("fn validate_traversal_expr_refs"),
        "dispatch.rs should route traversal expressions through traversal.rs"
    );
    assert!(
        construct_dispatch.contains("fn validate_construct_expr_refs"),
        "construct_dispatch.rs should route construct/scoped expressions through expression_validation_constructs.rs"
    );
    assert!(
        !calls.contains("fn validate_call_expr_refs")
            && !traversal.contains("fn validate_traversal_expr_refs")
            && !constructs.contains("fn validate_construct_expr_refs"),
        "category dispatch should stay out of leaf validation helpers"
    );
}
