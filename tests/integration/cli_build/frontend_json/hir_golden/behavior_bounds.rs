use super::assert_hir_golden;

#[test]
fn emit_json_hir_generic_behavior_association_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/behavior_json_generic_association.zen",
        "tests/fixtures/ir_json/hir_generic_behavior_association.golden.json",
        "generic behavior association input",
    );
}

#[test]
fn emit_json_hir_generic_behavior_bound_ufcs_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/behavior_json_generic_bound_ufcs.zen",
        "tests/fixtures/ir_json/hir_generic_behavior_bound_ufcs.golden.json",
        "generic behavior-bound UFCS input",
    );
}
