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
