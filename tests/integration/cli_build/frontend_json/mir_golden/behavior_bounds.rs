use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_behavior_association_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/behavior_json_generic_association.zen",
        "tests/fixtures/ir_json/mir_generic_behavior_association.golden.json",
        "generic behavior association input",
    );
}

#[test]
fn emit_json_mir_generic_behavior_bound_ufcs_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/behavior_json_generic_bound_ufcs.zen",
        "tests/fixtures/ir_json/mir_generic_behavior_bound_ufcs.golden.json",
        "generic behavior-bound UFCS input",
    );
}
