use super::*;

#[test]
fn typechecker_resolver_validation_entry_helpers_stay_split_by_surface() {
    let root =
        read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation/entry_helpers.rs");
    let post_pass = read(
        "tests/docs_truth/repo_hygiene/typechecker_resolver_validation/entry_helpers/post_pass.rs",
    );
    let local_helpers =
        read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation/entry_helpers/local_helpers.rs");
    let type_entries =
        read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation/entry_helpers/type_entries.rs");
    let metadata_helpers =
        read("tests/docs_truth/repo_hygiene/typechecker_resolver_validation/entry_helpers/metadata_helpers.rs");

    assert!(
        root.lines().count() < 80,
        "entry_helpers.rs should route focused resolver-validation guard surfaces"
    );
    for module in [
        "mod local_helpers;",
        "mod metadata_helpers;",
        "mod post_pass;",
        "mod type_entries;",
    ] {
        assert!(
            root.contains(module),
            "entry_helpers.rs should include focused module `{module}`"
        );
    }
    assert!(
        !root.contains("fn typechecker_resolver_validation_post_pass_lives_in_focused_helper"),
        "post-pass guard should live in the focused post_pass module"
    );

    assert!(
        post_pass.contains("fn typechecker_resolver_validation_post_pass_lives_in_focused_helper"),
        "post-pass guard should live in post_pass.rs"
    );
    assert!(
        local_helpers
            .contains("fn typechecker_resolver_entry_local_helpers_live_in_focused_helper")
            && local_helpers.contains(
                "fn typechecker_resolver_pattern_local_traversal_lives_in_focused_helper"
            ),
        "local and pattern traversal guards should live in local_helpers.rs"
    );
    assert!(
        type_entries.contains(
            "fn typechecker_resolver_entry_type_declaration_helpers_live_in_focused_helper"
        ),
        "type entry guard should live in type_entries.rs"
    );
    assert!(
        metadata_helpers
            .contains("fn typechecker_resolver_variant_metadata_lives_in_focused_helper")
            && metadata_helpers
                .contains("fn typechecker_resolver_absence_diagnostics_live_in_focused_helper")
            && metadata_helpers.contains(
                "fn typechecker_resolver_import_absence_metadata_lives_in_focused_helper"
            ),
        "metadata guards should live in metadata_helpers.rs"
    );
}
