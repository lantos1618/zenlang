use super::super::*;

#[test]
fn frontend_json_symbol_multi_specialization_tests_live_in_focused_helper() {
    let root = read("tests/integration/cli_build/frontend_json/module_graph/symbols/generic.rs");
    let multi_specialization = read(
        "tests/integration/cli_build/frontend_json/module_graph/symbols/generic/result_multi_specialization.rs",
    );

    for test_name in [
        "emit_json_symbols_reports_multi_file_generic_result_multi_specialization_surface",
        "emit_json_symbols_reports_multi_file_generic_result_error_multi_specialization_surface",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "generic.rs should not own Result multi-specialization symbol surface test: {test_name}"
        );
        assert!(
            multi_specialization.contains(&format!("fn {test_name}")),
            "Result multi-specialization symbol surface tests should live in focused module: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 190,
        "generic.rs should stay focused on generic symbol surfaces and shared helpers"
    );
    assert!(
        root.contains("mod result_multi_specialization;"),
        "generic.rs should include the focused result_multi_specialization module"
    );
}
