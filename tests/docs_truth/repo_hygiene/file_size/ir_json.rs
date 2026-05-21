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

#[test]
fn layout_json_scalar_type_layouts_live_in_focused_helper() {
    let context = read("src/ir_json/layout/context.rs");
    let scalar_types = read("src/ir_json/layout/context/scalar_types.rs");

    for helper in [
        "fn seed_builtin_layouts",
        "fn layout_builtin_type",
        "fn cache_compound_layout",
        "fn layout_by_name",
    ] {
        assert!(
            !context.contains(helper),
            "layout context should not own scalar type layout helper: {helper}"
        );
        assert!(
            scalar_types.contains(helper),
            "scalar type layout helper should live in focused helper: {helper}"
        );
    }

    for builtin in ["StaticString", "String", "dynamic_string", "static_string"] {
        assert!(
            scalar_types.contains(builtin),
            "focused builtin layout helper should own runtime layout literal: {builtin}"
        );
    }

    assert!(
        context.contains("mod scalar_types;"),
        "layout context should include the focused scalar type layout helper"
    );
    assert!(
        context.lines().count() < 170,
        "layout context should stay below the next focused size threshold"
    );
}
