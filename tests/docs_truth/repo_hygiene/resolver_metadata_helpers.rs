use super::*;

#[test]
fn resolver_behavior_ref_helpers_live_in_focused_module() {
    let root = read("src/resolver/metadata_helpers.rs");
    let behavior_refs = read("src/resolver/metadata_helpers/behavior_refs.rs");

    for helper in [
        "fn behavior_ref_display(",
        "fn resolver_behavior_impl_method_key(",
        "fn resolver_behavior_impl_method_key_with_target_args(",
        "fn behavior_ref_symbol_suffix(",
        "fn sanitize_symbol_part(",
    ] {
        assert!(
            !root.contains(helper),
            "resolver metadata_helpers.rs should not own behavior-ref helper `{helper}`"
        );
        assert!(
            behavior_refs.contains(helper),
            "resolver behavior-ref metadata helper should live in focused module: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "metadata_helpers.rs should stay focused on non-behavior-ref metadata construction"
    );
    assert!(
        root.contains("mod behavior_refs;"),
        "metadata_helpers.rs should include focused behavior-ref helpers"
    );
}
