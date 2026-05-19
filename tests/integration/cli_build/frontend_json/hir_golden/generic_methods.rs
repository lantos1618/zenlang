use super::assert_hir_golden;

#[test]
fn emit_json_hir_generic_method_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_method.zen",
        "tests/fixtures/ir_json/hir_generic_method.golden.json",
        "generic method input",
    );
}

#[test]
fn emit_json_hir_generic_type_impl_methods_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_type_impl_methods.zen",
        "tests/fixtures/ir_json/hir_generic_type_impl_methods.golden.json",
        "generic type impl methods input",
    );
}

#[test]
fn emit_json_hir_generic_self_method_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_method_self.zen",
        "tests/fixtures/ir_json/hir_generic_self_method.golden.json",
        "generic Self method input",
    );
}

#[test]
fn emit_json_hir_generic_method_worklist_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_method_worklist.zen",
        "tests/fixtures/ir_json/hir_generic_method_worklist.golden.json",
        "generic method worklist input",
    );
}

#[test]
fn emit_json_hir_generic_result_method_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_result_enum_method.zen",
        "tests/fixtures/ir_json/hir_generic_result_method.golden.json",
        "generic Result method program input",
    );
}

#[test]
fn emit_json_hir_nested_generic_result_schema_matches_golden() {
    assert_hir_golden(
        "tests/zen/generic_nested_result_enum.zen",
        "tests/fixtures/ir_json/hir_nested_generic_result.golden.json",
        "nested generic Result program input",
    );
}
