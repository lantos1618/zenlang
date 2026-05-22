use super::*;

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
fn test_behavior_distinct_generic_specialization_dispatch() {
    run_test("behavior_distinct_generic_specialization_dispatch");
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
fn test_behavior_generic_parent_type_arg_inheritance() {
    run_test("behavior_generic_parent_type_arg_inheritance");
}

#[test]
fn test_behavior_inherited_generic_dispatch() {
    run_test("behavior_inherited_generic_dispatch");
}
