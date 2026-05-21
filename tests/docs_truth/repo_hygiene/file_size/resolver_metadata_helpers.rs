use super::super::*;

#[test]
fn resolver_behavior_ref_key_helpers_live_in_focused_helper() {
    let root = read("src/resolver/metadata_helpers.rs");
    let behavior_refs = read("src/resolver/metadata_helpers/behavior_refs.rs");

    assert!(
        root.lines().count() < 180,
        "metadata_helpers.rs should stay focused on value signatures and aggregate metadata"
    );
    assert!(
        root.contains("mod behavior_refs;"),
        "metadata helpers should include the focused behavior-ref helper"
    );
    assert!(
        root.contains("pub(super) use behavior_refs::{"),
        "metadata helpers should re-export behavior-ref helpers for resolver modules"
    );

    for helper in [
        "fn behavior_ref_display",
        "fn resolver_method_key",
        "fn resolver_behavior_impl_method_key",
        "fn behavior_ref_symbol_suffix",
        "fn sanitize_symbol_part",
    ] {
        assert!(
            !root.contains(helper),
            "metadata_helpers.rs should not own behavior-ref/key helper: {helper}"
        );
        assert!(
            behavior_refs.contains(helper),
            "behavior-ref/key helper should live in behavior_refs.rs: {helper}"
        );
    }
}
