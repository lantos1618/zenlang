use super::*;

#[test]
fn test_hello() {
    run_test("hello");
}

#[test]
fn test_arithmetic() {
    run_test("arithmetic");
}

#[test]
fn test_structs() {
    run_test("structs");
}

#[test]
fn test_enums() {
    run_test("enums");
}

#[test]
fn test_duplicate_enum_variant_names() {
    run_test("duplicate_enum_variant_names");
}

#[test]
fn test_ufc() {
    run_test("ufc");
}

#[test]
fn test_conditionals() {
    run_test("conditionals");
}

#[test]
fn test_loops() {
    run_test("loops");
}

#[test]
fn test_strings() {
    run_test("strings");
}

#[test]
fn test_functions() {
    run_test("functions");
}

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
fn test_generic_method() {
    run_test("generic_method");
}

#[test]
fn test_generic_method_self() {
    run_test("generic_method_self");
}

#[test]
fn test_generic_method_worklist() {
    run_test("generic_method_worklist");
}

#[test]
fn test_generic_method_nested_result() {
    run_test("generic_method_nested_result");
}

#[test]
fn test_type_impl_methods() {
    run_test("type_impl_methods");
}

#[test]
fn test_generic_result_enum() {
    run_test("generic_result_enum");
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
fn test_generic_ufc_function() {
    run_test("generic_ufc_function");
}

#[test]
fn test_behavior_json_explicit_impl() {
    run_test("behavior_json_explicit_impl");
}

#[test]
fn test_behavior_default_method_dispatch() {
    run_test("behavior_default_method_dispatch");
}

#[test]
fn test_behavior_generic_default_method() {
    run_test("behavior_generic_default_method");
}

#[test]
fn test_behavior_inherited_default_method() {
    run_test("behavior_inherited_default_method");
}

#[test]
fn test_behavior_json_generic_dispatch() {
    run_test("behavior_json_generic_dispatch");
}

#[test]
fn test_behavior_json_generic_association() {
    run_test("behavior_json_generic_association");
}

#[test]
fn test_behavior_json_generic_bound() {
    run_test("behavior_json_generic_bound");
}

#[test]
fn test_behavior_json_generic_bound_ufcs() {
    run_test("behavior_json_generic_bound_ufcs");
}

#[test]
fn test_behavior_generic_parent_inheritance() {
    run_test("behavior_generic_parent_inheritance");
}

#[test]
fn test_behavior_inherited_generic_dispatch() {
    run_test("behavior_inherited_generic_dispatch");
}
