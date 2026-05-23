use super::super::*;

#[test]
fn phase_plan_evidence_audits_live_in_focused_modules() {
    let phase_plan = read("tests/docs_truth/phase_audit/phase_plan.rs");
    let phase_audit = read("tests/docs_truth/phase_audit.rs");
    let generated_c_evidence = read("tests/docs_truth/phase_audit/generated_c_evidence.rs");
    let generic_diagnostics_evidence =
        read("tests/docs_truth/phase_audit/generic_diagnostics_evidence.rs");

    for test_name in [
        "nested_generic_result_generated_c_pins_definition_counts",
        "multi_file_nested_generic_method_generated_c_pins_definition_counts",
        "local_nested_generic_method_generated_c_pins_definition_counts",
        "imported_transitive_worklist_generated_c_pins_definition_counts",
        "scoped_imported_generic_ufc_generated_c_pins_recovery_evidence",
    ] {
        assert!(
            !phase_plan.contains(&format!("fn {test_name}")),
            "phase_plan.rs should not own generated-C evidence test: {test_name}"
        );
        assert!(
            generated_c_evidence.contains(&format!("fn {test_name}")),
            "generated-C evidence test should live in focused module: {test_name}"
        );
    }

    assert!(
        !phase_plan.contains("fn phase5_generic_diagnostics_pin_codes_in_unit_tests"),
        "phase_plan.rs should not own generic diagnostic evidence tests"
    );
    assert!(
        generic_diagnostics_evidence
            .contains("fn phase5_generic_diagnostics_pin_codes_in_unit_tests"),
        "generic diagnostic evidence should live in focused module"
    );

    assert!(
        phase_plan.lines().count() < 120,
        "phase_plan.rs should stay focused on docs/PHASE_PLAN.md shape"
    );
    assert!(
        phase_audit.contains("mod generated_c_evidence;"),
        "phase_audit.rs should include generated-C evidence audits"
    );
    assert!(
        phase_audit.contains("mod generic_diagnostics_evidence;"),
        "phase_audit.rs should include generic diagnostic evidence audits"
    );
}
