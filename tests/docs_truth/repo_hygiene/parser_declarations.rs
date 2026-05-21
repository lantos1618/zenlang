use super::*;

#[test]
fn parser_declaration_suffixes_live_in_focused_helper() {
    let declarations = read("src/parser/declarations.rs");
    let suffixes = read("src/parser/declarations/suffix_forms.rs");

    for helper in [
        "parse_generic_declaration_suffix",
        "parse_type_suffix_declaration",
        "reject_gated_generic_association_target",
        "gated_association_call_span",
    ] {
        assert!(
            !declarations.contains(&format!("fn {helper}")),
            "parser declaration dispatch should not own suffix helper: {helper}"
        );
        assert!(
            suffixes.contains(&format!("fn {helper}")),
            "parser declaration suffix helper should live in suffix_forms.rs: {helper}"
        );
    }

    assert!(
        declarations.lines().count() < 150,
        "declarations.rs should stay focused on prefix declaration dispatch"
    );
    assert!(
        declarations.contains("mod suffix_forms;"),
        "parser declaration dispatch should load the focused suffix helper"
    );
}
