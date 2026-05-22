use super::*;

#[test]
fn match_semantics_tests_stay_split_by_match_subject() {
    let root = read("src/typechecker/tests/core_semantics/match_semantics.rs");
    let bool_matches = read("src/typechecker/tests/core_semantics/match_semantics/bool_matches.rs");
    let enum_matches = read("src/typechecker/tests/core_semantics/match_semantics/enum_matches.rs");

    assert!(
        root.lines().count() < 80,
        "match_semantics.rs should only route focused match semantics tests"
    );
    for module in ["mod bool_matches;", "mod enum_matches;"] {
        assert!(
            root.contains(module),
            "match_semantics.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn enum_match_missing_variant_is_error"),
        "enum match tests should live in enum_matches.rs"
    );
    assert!(
        enum_matches.contains("fn enum_match_payload_shape_is_checked"),
        "enum_matches.rs should cover enum payload validation"
    );
    assert!(
        bool_matches.contains("fn bool_match_missing_arm_is_error_for_value_match"),
        "bool_matches.rs should cover bool exhaustiveness"
    );
    assert!(
        bool_matches.contains("fn match_arm_return_does_not_force_never_result_type"),
        "bool_matches.rs should cover match result typing"
    );
}
