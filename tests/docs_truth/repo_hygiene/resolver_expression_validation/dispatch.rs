use super::*;

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
