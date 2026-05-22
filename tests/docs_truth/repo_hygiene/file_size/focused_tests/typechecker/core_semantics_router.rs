use super::*;

#[test]
fn core_semantics_docs_truth_guards_stay_split_by_surface() {
    let root =
        read("tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/core_semantics.rs");
    let declaration_validation =
        read("tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/core_semantics/declaration_validation.rs");
    let gated_intrinsics =
        read("tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/core_semantics/gated_intrinsics.rs");
    let literals_and_types =
        read("tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/core_semantics/literals_and_types.rs");
    let match_semantics =
        read("tests/docs_truth/repo_hygiene/file_size/focused_tests/typechecker/core_semantics/match_semantics.rs");

    assert!(
        root.lines().count() < 80,
        "core_semantics.rs should only route focused core semantics guard modules"
    );
    for module in [
        "mod declaration_validation;",
        "mod gated_intrinsics;",
        "mod literals_and_types;",
        "mod match_semantics;",
    ] {
        assert!(
            root.contains(module),
            "core_semantics.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn declaration_validation_precollection_tasks_live_in_focused_helper"),
        "declaration validation guards should live in declaration_validation.rs"
    );
    assert!(
        declaration_validation
            .contains("fn declaration_validation_resolver_replay_tests_stay_split_by_task_kind"),
        "declaration_validation.rs should cover declaration validation guard splits"
    );
    assert!(
        gated_intrinsics.contains("fn intrinsic_gate_tests_stay_split_by_effect_family"),
        "gated_intrinsics.rs should cover intrinsic gate guard splits"
    );
    assert!(
        literals_and_types.contains("fn type_helper_tests_stay_split_by_semantic_surface"),
        "literals_and_types.rs should cover literal and type helper guard splits"
    );
    assert!(
        match_semantics.contains("fn match_semantics_tests_stay_split_by_match_subject"),
        "match_semantics.rs should cover match semantics guard splits"
    );
}
