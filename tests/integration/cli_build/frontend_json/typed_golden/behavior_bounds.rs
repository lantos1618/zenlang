use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_behavior_association_schema_matches_golden() {
    assert_typed_golden(
        "behavior_json_generic_association.zen",
        "generic_behavior_association",
        "generic behavior association",
    );
}

#[test]
fn emit_json_typed_generic_behavior_bound_ufcs_schema_matches_golden() {
    assert_typed_golden(
        "behavior_json_generic_bound_ufcs.zen",
        "generic_behavior_bound_ufcs",
        "generic behavior-bound UFCS",
    );
}
