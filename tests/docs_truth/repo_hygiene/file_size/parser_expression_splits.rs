use super::super::*;

#[test]
fn parser_expression_tests_live_in_focused_helpers() {
    let root = read("src/parser/tests/expressions.rs");
    let module_dir = "src/parser/tests/expressions";

    for (module, test_name) in [
        ("control_flow", "parse_pattern_match"),
        ("control_flow", "parse_loop_control_param_expr"),
        (
            "enum_variants",
            "parse_shorthand_enum_variant_expr_and_pattern",
        ),
        ("forms", "parse_struct_literal"),
        ("forms", "parse_range_expr"),
        (
            "generic_disambiguation",
            "speculative_generic_lookahead_preserves_shift_right_tokens",
        ),
        (
            "generic_disambiguation",
            "type_argument_lists_require_commas_between_args",
        ),
    ] {
        let focused = read(format!("{module_dir}/{module}.rs"));
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "expressions.rs should not own parser expression test: {test_name}"
        );
        assert!(
            root.contains(&format!("mod {module};")),
            "expressions.rs should include focused module `{module}`"
        );
        assert!(
            focused.contains(&format!("fn {test_name}")),
            "parser expression test should live in focused module `{module}`: {test_name}"
        );
    }

    assert!(
        !root.contains("#[test]"),
        "expressions.rs should stay as a router and not define tests directly"
    );
    assert!(
        root.lines().count() < 80,
        "expressions.rs should stay as a small router for parser expression tests"
    );
}
