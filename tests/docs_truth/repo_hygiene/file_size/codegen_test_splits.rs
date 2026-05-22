use super::super::*;

#[test]
fn codegen_c_expression_literal_tests_live_in_focused_helper() {
    let root = read("src/codegen/c/tests/expression_emission.rs");
    let literals = read("src/codegen/c/tests/expression_emission/literals.rs");

    for test_name in [
        "emit_int_literal",
        "emit_float_literal",
        "emit_bool_literal",
        "emit_string_literal",
        "emit_variable",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "expression_emission.rs should not own primitive expression test: {test_name}"
        );
        assert!(
            literals.contains(&format!("fn {test_name}")),
            "primitive expression emission tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 210,
        "expression_emission.rs should stay focused on compound expression and statement emission"
    );
    assert!(
        root.contains("#[path = \"expression_emission/literals.rs\"]"),
        "expression_emission.rs should include the focused literal emission module by path"
    );
}
