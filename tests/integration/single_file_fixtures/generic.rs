use super::*;

#[test]
fn test_generic_identity() {
    run_test("generic_identity");
}

#[test]
fn test_generic_struct() {
    run_test("generic_struct");
}

#[test]
fn test_generic_enum_option() {
    run_test("generic_enum_option");
}

#[test]
fn test_generic_enum_method() {
    run_test("generic_enum_method");
}

#[test]
fn test_generic_enum_multi_specialization() {
    run_test("generic_enum_multi_specialization");
}

#[test]
fn test_generic_enum_nested_payload_inference() {
    run_test("generic_enum_nested_payload_inference");
}

#[test]
fn test_generic_method() {
    run_test("generic_method");
}

#[test]
fn test_generic_method_self() {
    run_test("generic_method_self");
}

#[test]
fn test_generic_enum_method_self_renamed_params() {
    run_test("generic_enum_method_self_renamed_params");
}

#[test]
fn test_generic_method_worklist() {
    run_test("generic_method_worklist");
}

#[test]
fn test_generic_method_method_worklist() {
    run_test("generic_method_method_worklist");
}

#[test]
fn test_generic_method_nested_result() {
    run_test("generic_method_nested_result");
}

#[test]
fn test_generic_enum_method_nested_result() {
    run_test("generic_enum_method_nested_result");
}

#[test]
fn test_generic_type_impl_methods() {
    run_test("generic_type_impl_methods");
}

#[test]
fn test_generic_result_enum() {
    run_test("generic_result_enum");
}

#[test]
fn test_generic_result_enum_method() {
    run_test("generic_result_enum_method");
}

#[test]
fn test_generic_result_enum_multi_specialization() {
    run_test("generic_result_enum_multi_specialization");
}

#[test]
fn test_generic_nested_result_enum() {
    run_test("generic_nested_result_enum");
}

#[test]
fn test_generic_vec() {
    run_test("generic_vec");
}

#[test]
fn test_generic_worklist() {
    run_test("generic_worklist");
}

#[test]
fn test_generic_worklist_dedup() {
    run_test("generic_worklist_dedup");
}

#[test]
fn test_generic_recursive_function() {
    run_test("generic_recursive_function");
}

#[test]
fn test_generic_recursive_method() {
    run_test("generic_recursive_method");
}

#[test]
fn test_generic_ufc_function() {
    run_test("generic_ufc_function");
}

#[test]
fn test_generic_ufc_dedup() {
    run_test("generic_ufc_dedup");
}
