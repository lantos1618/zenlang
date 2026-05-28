use super::assert_symbols_fixture_matches;

#[test]
fn emit_json_symbols_generic_schemas_match_golden() {
    for (source, fixture) in [
        (
            "tests/zen/generic_method.zen",
            "tests/fixtures/ir_json/symbols_generic_method.golden.json",
        ),
        (
            "tests/zen/generic_enum_option.zen",
            "tests/fixtures/ir_json/symbols_generic_option.golden.json",
        ),
        (
            "tests/zen/generic_result_enum.zen",
            "tests/fixtures/ir_json/symbols_generic_result.golden.json",
        ),
        (
            "tests/zen/generic_result_enum_method.zen",
            "tests/fixtures/ir_json/symbols_generic_result_method.golden.json",
        ),
        (
            "tests/zen/generic_enum_method_nested_result.zen",
            "tests/fixtures/ir_json/symbols_generic_enum_method_nested_result.golden.json",
        ),
        (
            "tests/zen/generic_type_impl_methods.zen",
            "tests/fixtures/ir_json/symbols_generic_type_impl_methods.golden.json",
        ),
        (
            "tests/zen/generic_method_self.zen",
            "tests/fixtures/ir_json/symbols_generic_self_method.golden.json",
        ),
        (
            "tests/zen/generic_method_worklist.zen",
            "tests/fixtures/ir_json/symbols_generic_method_worklist.golden.json",
        ),
        (
            "tests/zen/generic_ufc_function.zen",
            "tests/fixtures/ir_json/symbols_generic_ufc_function.golden.json",
        ),
        (
            "tests/zen/behavior_json_generic_association.zen",
            "tests/fixtures/ir_json/symbols_generic_behavior_association.golden.json",
        ),
        (
            "tests/zen/behavior_json_generic_bound_ufcs.zen",
            "tests/fixtures/ir_json/symbols_generic_behavior_bound_ufcs.golden.json",
        ),
    ] {
        assert_symbols_fixture_matches(source, fixture);
    }
}
