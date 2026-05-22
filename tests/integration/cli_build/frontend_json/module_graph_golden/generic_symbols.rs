use super::assert_symbols_fixture_matches;

#[test]
fn emit_json_symbols_generic_method_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_method.zen",
        "tests/fixtures/ir_json/symbols_generic_method.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_option_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_enum_option.zen",
        "tests/fixtures/ir_json/symbols_generic_option.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_result_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_result_enum.zen",
        "tests/fixtures/ir_json/symbols_generic_result.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_result_method_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_result_enum_method.zen",
        "tests/fixtures/ir_json/symbols_generic_result_method.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_enum_method_nested_result_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_enum_method_nested_result.zen",
        "tests/fixtures/ir_json/symbols_generic_enum_method_nested_result.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_type_impl_methods_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_type_impl_methods.zen",
        "tests/fixtures/ir_json/symbols_generic_type_impl_methods.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_self_method_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_method_self.zen",
        "tests/fixtures/ir_json/symbols_generic_self_method.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_method_worklist_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_method_worklist.zen",
        "tests/fixtures/ir_json/symbols_generic_method_worklist.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_ufc_function_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/generic_ufc_function.zen",
        "tests/fixtures/ir_json/symbols_generic_ufc_function.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_behavior_association_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/behavior_json_generic_association.zen",
        "tests/fixtures/ir_json/symbols_generic_behavior_association.golden.json",
    );
}

#[test]
fn emit_json_symbols_generic_behavior_bound_ufcs_schema_matches_golden() {
    assert_symbols_fixture_matches(
        "tests/zen/behavior_json_generic_bound_ufcs.zen",
        "tests/fixtures/ir_json/symbols_generic_behavior_bound_ufcs.golden.json",
    );
}
