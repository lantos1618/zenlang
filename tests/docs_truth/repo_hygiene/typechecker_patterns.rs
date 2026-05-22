use super::*;

#[test]
fn bool_match_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/patterns/match_validation.rs");
    let bools = read("src/typechecker/patterns/match_validation_bool.rs");
    let patterns = read("src/typechecker/patterns.rs");

    for helper in [
        "check_bool_match_patterns",
        "missing_bool_match_values",
        "missing_bool_match_fix_replacement",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "match validation root should not own bool-specific helper: {helper}"
        );
        assert!(
            bools.contains(&format!("fn {helper}")),
            "bool match validation should live in focused helper: {helper}"
        );
    }

    assert!(
        patterns.contains("mod match_validation_bool;"),
        "pattern helpers should load focused bool match validation"
    );
}

#[test]
fn enum_match_validation_lives_in_focused_helper() {
    let root = read("src/typechecker/patterns/match_validation.rs");
    let enums = read("src/typechecker/patterns/match_validation_enum.rs");
    let patterns = read("src/typechecker/patterns.rs");

    for helper in [
        "check_match_exhaustiveness",
        "check_enum_match_patterns",
        "enum_variants_for_match",
        "enum_variant_payloads_for_match",
        "explicit_enum_variant_pattern",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "match validation root should not own enum-specific helper: {helper}"
        );
        assert!(
            enums.contains(&format!("fn {helper}")),
            "enum match validation should live in focused helper: {helper}"
        );
    }

    assert!(
        root.lines().count() < 80,
        "match validation root should stay focused on match kind classification"
    );
    assert!(
        patterns.contains("mod match_validation_enum;"),
        "pattern helpers should load focused enum match validation"
    );
}
