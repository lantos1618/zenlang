use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_method_schema_matches_golden() {
    assert_typed_golden(
        "generic_method.zen",
        "tests/fixtures/ir_json/typed_generic_method.golden.json",
        "generic method",
    );
}

#[test]
fn emit_json_typed_generic_type_impl_methods_schema_matches_golden() {
    assert_typed_golden(
        "generic_type_impl_methods.zen",
        "tests/fixtures/ir_json/typed_generic_type_impl_methods.golden.json",
        "generic type impl methods",
    );
}

#[test]
fn emit_json_typed_generic_self_method_schema_matches_golden() {
    assert_typed_golden(
        "generic_method_self.zen",
        "tests/fixtures/ir_json/typed_generic_self_method.golden.json",
        "generic Self method",
    );
}

#[test]
fn emit_json_typed_generic_method_worklist_schema_matches_golden() {
    assert_typed_golden(
        "generic_method_worklist.zen",
        "tests/fixtures/ir_json/typed_generic_method_worklist.golden.json",
        "generic method worklist",
    );
}

#[test]
fn emit_json_typed_generic_method_nested_result_schema_matches_golden() {
    assert_typed_golden(
        "generic_method_nested_result.zen",
        "tests/fixtures/ir_json/typed_generic_method_nested_result.golden.json",
        "generic method nested Result",
    );
}
