use super::super::*;

#[test]
fn layout_json_context_lives_in_focused_helper() {
    let root = read("src/ir_json/layout.rs");
    let context = read("src/ir_json/layout/context.rs");

    assert!(
        root.lines().count() < 80,
        "layout.rs should stay focused on JSON emission wiring"
    );
    assert!(
        root.contains("mod context;"),
        "layout.rs should include the focused layout context helper"
    );
    assert!(
        !root.contains("struct LayoutContext"),
        "layout.rs should not own layout context state"
    );
    for helper in [
        "fn new",
        "fn seed_builtin_layouts",
        "fn layout_type_def",
        "fn layout_struct",
        "fn layout_fields",
        "fn layout_type",
        "fn layout_named",
    ] {
        assert!(
            !root.contains(helper),
            "layout.rs should not own layout context helper: {helper}"
        );
        assert!(
            context.contains(helper),
            "layout context helper should live in context.rs: {helper}"
        );
    }
}
