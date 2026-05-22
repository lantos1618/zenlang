use super::super::*;

#[test]
fn generic_single_file_fixture_tests_live_in_focused_helper() {
    let root = read("tests/integration/single_file_fixtures.rs");
    let generic = read("tests/integration/single_file_fixtures/generic.rs");

    for test_name in [
        "test_generic_identity",
        "test_generic_enum_method_nested_result",
        "test_generic_result_enum_multi_specialization",
        "test_generic_recursive_method",
        "test_generic_ufc_dedup",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "single_file_fixtures.rs should not own generic fixture test: {test_name}"
        );
        assert!(
            generic.contains(&format!("fn {test_name}")),
            "generic single-file fixture tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "single_file_fixtures.rs should stay focused on core and behavior fixture tests"
    );
    assert!(
        root.contains("#[path = \"single_file_fixtures/generic.rs\"]"),
        "single_file_fixtures.rs should include the focused generic fixture module by path"
    );
}
