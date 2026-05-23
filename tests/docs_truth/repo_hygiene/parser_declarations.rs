use super::*;

#[test]
fn parser_generic_declarations_live_in_focused_helper() {
    let declarations = read("src/parser/declarations.rs");
    let generic = read("src/parser/declarations/generic.rs");

    for helper in [
        "parse_generic_declaration",
        "reject_gated_generic_association_target",
        "gated_association_call_span",
    ] {
        assert!(
            !declarations.contains(&format!("fn {helper}")),
            "parser declaration dispatch should not own generic declaration helper: {helper}"
        );
        assert!(
            generic.contains(&format!("fn {helper}")),
            "generic declaration parsing should live in the focused helper: {helper}"
        );
    }

    assert!(
        declarations.contains("mod generic;"),
        "parser declaration dispatch should load the focused generic-declarations helper"
    );
    assert!(
        declarations.lines().count() < 190,
        "parser declaration dispatch should stay small after generic declarations move out"
    );
}

#[test]
fn parser_declaration_tests_share_single_declaration_helper() {
    let declaration_tests = read("src/parser/tests/declarations.rs");
    let parser_tests = read("src/parser/tests.rs");

    assert!(
        parser_tests.contains("fn parse_single_decl("),
        "parser tests should expose a helper for single-declaration fixtures"
    );
    assert!(
        !declaration_tests.contains("assert_eq!(prog.declarations.len(), 1)"),
        "parser declaration tests should not repeat trivial one-declaration assertions"
    );
}
