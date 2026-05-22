use super::*;

#[test]
fn behavior_impl_signature_collection_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let focused = read("src/typechecker/behavior_impl_signature_collection.rs");

    for helper in [
        "collect_impl_method_signature",
        "collect_resolver_backed_impl_method_template",
        "collect_resolver_behavior_impl_method_signatures",
        "collect_behavior_default_method_signatures",
        "should_skip_behavior_default_synthesis",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "typechecker root should not own behavior impl signature helper: {helper}"
        );
        assert!(
            focused.contains(&format!("fn {helper}")),
            "behavior impl signature collection should live in focused helper: {helper}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "typechecker root should stay focused on module wiring and shared imports"
    );
    assert!(
        root.contains("mod behavior_impl_signature_collection;"),
        "typechecker root should include focused behavior impl signature collection"
    );
}
