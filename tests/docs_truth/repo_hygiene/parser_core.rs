use super::*;

#[test]
fn parser_core_navigation_and_lookahead_live_in_focused_helpers() {
    let core = read("src/parser/core.rs");
    let navigation = read("src/parser/navigation.rs");
    let lookahead = read("src/parser/lookahead.rs");
    let root = read("src/parser/mod.rs");

    for helper in [
        "peek",
        "peek_span",
        "peek_skip_newlines",
        "peek_ahead",
        "advance",
        "skip_newlines",
        "expect",
        "expect_gt",
        "expect_identifier",
        "at_eof",
        "prev_span",
        "skip_newlines_if_continuation",
    ] {
        assert!(
            !core.contains(&format!("fn {helper}")),
            "parser core should not own token navigation helper: {helper}"
        );
        assert!(
            navigation.contains(&format!("fn {helper}")),
            "parser token navigation should live in focused helper: {helper}"
        );
    }

    for helper in [
        "is_import",
        "is_struct_def",
        "is_enum_def",
        "colon_is_followed_by_identifier",
        "is_struct_pattern",
    ] {
        assert!(
            !core.contains(&format!("fn {helper}")),
            "parser core should not own lookahead predicate: {helper}"
        );
        assert!(
            lookahead.contains(&format!("fn {helper}")),
            "parser lookahead predicates should live in focused helper: {helper}"
        );
    }

    for module_name in ["navigation", "lookahead"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "parser root should include focused helper module: {module_name}"
        );
    }
}

#[test]
fn parser_pratt_infix_lives_in_focused_helper() {
    let expressions = read("src/parser/expressions.rs");
    let infix = read("src/parser/expressions/infix.rs");

    for helper in [
        "fn parse_infix_or_range_expr",
        "enum InfixParse",
        "fn binary_op_for_token",
    ] {
        assert!(
            !expressions.contains(helper),
            "parser expression root should not own Pratt infix helper: {helper}"
        );
        assert!(
            infix.contains(helper),
            "Pratt infix helper should live in focused helper: {helper}"
        );
    }

    for forbidden in [
        "REMOVED_AS_CAST_L_BP",
        "REMOVED_INFIX_AS_CAST_MESSAGE",
        "Token::DotDot",
        "Token::DotDotEq",
    ] {
        assert!(
            !expressions.contains(forbidden),
            "parser expression root should not own infix/range detail: {forbidden}"
        );
    }

    assert!(
        expressions.contains("mod infix;"),
        "parser expression root should include the focused infix module"
    );
    assert!(
        expressions.lines().count() < 220,
        "parser expression root should stay focused on Pratt dispatch"
    );
}
