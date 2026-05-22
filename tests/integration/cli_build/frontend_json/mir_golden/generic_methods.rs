use super::assert_mir_golden;

#[test]
fn emit_json_mir_generic_method_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_method.zen",
        "tests/fixtures/ir_json/mir_generic_method.golden.json",
        "generic method input",
    );
}

#[test]
fn emit_json_mir_generic_type_impl_methods_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_type_impl_methods.zen",
        "tests/fixtures/ir_json/mir_generic_type_impl_methods.golden.json",
        "generic type impl methods input",
    );
}

#[test]
fn emit_json_mir_generic_self_method_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_method_self.zen",
        "tests/fixtures/ir_json/mir_generic_self_method.golden.json",
        "generic Self method input",
    );
}

#[test]
fn emit_json_mir_generic_method_worklist_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_method_worklist.zen",
        "tests/fixtures/ir_json/mir_generic_method_worklist.golden.json",
        "generic method worklist input",
    );
}

#[test]
fn emit_json_mir_generic_method_method_worklist_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_method_method_worklist.zen",
        "tests/fixtures/ir_json/mir_generic_method_method_worklist.golden.json",
        "generic method-to-method worklist input",
    );
}

#[test]
fn emit_json_mir_multi_file_type_method_worklist_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_type_method_worklist/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_type_method_worklist.golden.json",
        "multi-file type method worklist input",
    );
}

#[test]
fn emit_json_mir_generic_recursive_method_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_recursive_method.zen",
        "tests/fixtures/ir_json/mir_generic_recursive_method.golden.json",
        "generic recursive method input",
    );
}

#[test]
fn emit_json_mir_generic_method_nested_result_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_method_nested_result.zen",
        "tests/fixtures/ir_json/mir_generic_method_nested_result.golden.json",
        "generic method nested result input",
    );
}

#[test]
fn emit_json_mir_generic_enum_method_nested_result_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_enum_method_nested_result.zen",
        "tests/fixtures/ir_json/mir_generic_enum_method_nested_result.golden.json",
        "generic enum method nested result input",
    );
}

#[test]
fn emit_json_mir_multi_file_generic_method_nested_result_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_type_method_nested_result_dependency/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_generic_method_nested_result.golden.json",
        "multi-file generic method nested result input",
    );
}

#[test]
fn emit_json_mir_multi_file_generic_result_method_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/multi_file_generic_result_enum_method/main.zen",
        "tests/fixtures/ir_json/mir_multi_file_generic_result_method.golden.json",
        "multi-file generic Result method input",
    );
}

#[test]
fn emit_json_mir_generic_result_method_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_result_enum_method.zen",
        "tests/fixtures/ir_json/mir_generic_result_method.golden.json",
        "generic Result method program input",
    );
}

#[test]
fn emit_json_mir_nested_generic_result_schema_matches_golden() {
    assert_mir_golden(
        "tests/zen/generic_nested_result_enum.zen",
        "tests/fixtures/ir_json/mir_nested_generic_result.golden.json",
        "nested generic Result program input",
    );
}
