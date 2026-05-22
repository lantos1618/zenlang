use super::*;

#[test]
fn behavior_impl_resolver_refs_live_in_focused_helper() {
    let support = read("src/typechecker/behavior_impl_support.rs");
    let refs = read("src/typechecker/behavior_impl_support/resolver_refs.rs");

    for helper in [
        "resolver_impl_ref_for",
        "resolver_behavior_ref_for",
        "behavior_ref_parts",
        "pop_resolver_behavior_ref",
        "pop_resolver_behavior_ref_from_queue",
        "resolver_behavior_impl_ref_for_peek",
        "peek_resolver_behavior_ref",
        "resolver_behavior_ref_queue_index",
        "named_queue_index",
        "named_queue_index_preserving_future_front",
        "resolver_behavior_impl_ref_parts",
    ] {
        assert!(
            !support.contains(&format!("fn {helper}")),
            "behavior impl support root should not own resolver behavior-ref helper: {helper}"
        );
        assert!(
            refs.contains(&format!("fn {helper}")),
            "resolver behavior-ref helper should live in focused module: {helper}"
        );
    }

    assert!(
        support.contains("mod resolver_refs;"),
        "behavior impl support should load focused resolver behavior-ref helper"
    );
    assert!(
        support.lines().count() < 90,
        "behavior_impl_support.rs should stay focused on support module routing and non-ref helpers"
    );
}
