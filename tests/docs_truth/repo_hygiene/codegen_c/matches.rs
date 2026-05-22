use super::*;

#[test]
fn codegen_c_enum_match_emission_lives_in_focused_helper() {
    let root = read("src/codegen/c/matches.rs");
    let enum_match = read("src/codegen/c/matches/enum_match.rs");

    assert!(
        !root.contains("fn emit_enum_match"),
        "C match root should not own enum-specific switch emission"
    );
    assert!(
        enum_match.contains("fn emit_enum_match"),
        "enum match switch emission should live in focused helper"
    );
    assert!(
        root.contains("mod enum_match;"),
        "C match root should load focused enum-match helper"
    );
    assert!(
        root.lines().count() < 170,
        "matches.rs should stay focused on match-kind routing and shared match helpers"
    );
}
