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
