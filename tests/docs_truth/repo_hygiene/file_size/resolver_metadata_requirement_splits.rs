use super::super::*;

#[test]
fn resolver_metadata_requirement_helper_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_metadata/metadata_requirements.rs");
    let helpers = read("src/typechecker/tests/resolver_metadata/metadata_requirements/helpers.rs");

    for test_name in [
        "method_key_formats_type_qualified_method_name",
        "resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks",
        "resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "metadata_requirements.rs should not own resolver metadata helper test: {test_name}"
        );
        assert!(
            helpers.contains(&format!("fn {test_name}")),
            "resolver metadata helper tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 200,
        "metadata_requirements.rs should stay focused on required resolver metadata completeness tests"
    );
    assert!(
        root.contains("mod helpers;"),
        "metadata_requirements.rs should include the focused helpers module"
    );
}
