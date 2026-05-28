use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_behavior_association_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/behavior_json_generic_association.zen",
        "generic_behavior_association",
        "generic behavior association input",
    );
}

#[test]
fn emit_json_mir_generic_behavior_bound_ufcs_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/behavior_json_generic_bound_ufcs.zen",
        "generic_behavior_bound_ufcs",
        "generic behavior-bound UFCS input",
    );
}
