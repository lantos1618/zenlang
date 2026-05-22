use super::super::*;

mod behavior_descriptors;
mod constructors;
mod diagnostics;
mod local_traversal;

#[test]
fn resolver_validation_support_guards_stay_split_by_surface() {
    let root = read("tests/docs_truth/repo_hygiene/file_size/resolver_validation_support.rs");
    let behavior_descriptors = read(
        "tests/docs_truth/repo_hygiene/file_size/resolver_validation_support/behavior_descriptors.rs",
    );
    let local_traversal = read(
        "tests/docs_truth/repo_hygiene/file_size/resolver_validation_support/local_traversal.rs",
    );
    let constructors =
        read("tests/docs_truth/repo_hygiene/file_size/resolver_validation_support/constructors.rs");
    let diagnostics =
        read("tests/docs_truth/repo_hygiene/file_size/resolver_validation_support/diagnostics.rs");

    assert!(
        root.lines().count() < 80,
        "resolver_validation_support.rs should route focused file-size guard modules"
    );
    for module_name in [
        "behavior_descriptors",
        "local_traversal",
        "constructors",
        "diagnostics",
    ] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "resolver_validation_support.rs should include focused guard module: {module_name}"
        );
    }
    assert!(
        behavior_descriptors
            .contains("fn resolver_behavior_ref_validation_descriptor_lives_in_focused_helper"),
        "behavior descriptor guards should live in resolver_validation_support/behavior_descriptors.rs"
    );
    assert!(
        local_traversal
            .contains("fn expected_local_traversal_support_stays_split_by_responsibility"),
        "expected local traversal guards should live in resolver_validation_support/local_traversal.rs"
    );
    assert!(
        constructors.contains("fn resolver_backed_type_info_constructors_live_in_focused_helper"),
        "constructor/signature guards should live in resolver_validation_support/constructors.rs"
    );
    assert!(
        diagnostics.contains("fn resolver_count_diagnostics_live_in_focused_helper"),
        "diagnostic guards should live in resolver_validation_support/diagnostics.rs"
    );
}
