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
