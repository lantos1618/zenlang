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

    for helper in ["check_match_exhaustiveness", "check_enum_match_patterns"] {
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
        "match_validation.rs should stay focused on match kind routing"
    );
    assert!(
        patterns.contains("mod match_validation_enum;"),
        "pattern helpers should load focused enum match validation"
    );
}

#[test]
fn enum_match_metadata_helpers_live_in_focused_helper() {
    let enums = read("src/typechecker/patterns/match_validation_enum.rs");
    let metadata = read("src/typechecker/patterns/match_validation_enum/metadata.rs");

    for helper in [
        "EnumVariantPayloads",
        "enum_variants_for_match",
        "enum_variant_payloads_for_match",
        "enum_variant_name_from_pattern",
        "explicit_enum_variant_pattern",
    ] {
        assert!(
            !enums.contains(&format!("type {helper}")) && !enums.contains(&format!("fn {helper}")),
            "enum match diagnostics should not own metadata helper: {helper}"
        );
        assert!(
            metadata.contains(&format!("type {helper}"))
                || metadata.contains(&format!("fn {helper}")),
            "enum match metadata helper should live in focused helper: {helper}"
        );
    }

    assert!(
        enums.contains("mod metadata;"),
        "enum match diagnostics should include focused metadata helper"
    );
    assert!(
        enums.lines().count() < 140,
        "enum match diagnostics should stay focused on validation flow"
    );
}
