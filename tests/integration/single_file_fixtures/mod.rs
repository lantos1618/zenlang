use super::*;
mod generic;

macro_rules! fixture_tests {
    ($(fn $name:ident() => run_test($fixture:literal);)+) => {$(
        #[test]
        fn $name() {
            run_test($fixture);
        }
    )+};
}

fixture_tests! {
    fn test_hello() => run_test("hello");
    fn test_arithmetic() => run_test("arithmetic");
    fn test_structs() => run_test("structs");
    fn test_enums() => run_test("enums");
    fn test_duplicate_enum_variant_names() => run_test("duplicate_enum_variant_names");
    fn test_ufc() => run_test("ufc");
    fn test_conditionals() => run_test("conditionals");
    fn test_loops() => run_test("loops");
    fn test_loop_control() => run_test("loop_control");
    fn test_strings() => run_test("strings");
    fn test_functions() => run_test("functions");
    fn test_type_impl_methods() => run_test("type_impl_methods");
    fn test_behavior_json_explicit_impl() => run_test("behavior_json_explicit_impl");
    fn test_behavior_default_method_dispatch() => run_test("behavior_default_method_dispatch");
    fn test_behavior_generic_default_method() => run_test("behavior_generic_default_method");
    fn test_behavior_inherited_default_method() => run_test("behavior_inherited_default_method");
    fn test_behavior_json_generic_dispatch() => run_test("behavior_json_generic_dispatch");
    fn test_behavior_json_generic_association() => run_test("behavior_json_generic_association");
    fn test_behavior_generic_target_association() => run_test("behavior_generic_target_association");
    fn test_behavior_generic_target_default_method() => run_test("behavior_generic_target_default_method");
    fn test_behavior_generic_target_distinct_behavior_args() => run_test("behavior_generic_target_distinct_behavior_args");
    fn test_behavior_distinct_generic_specialization_dispatch() => run_test("behavior_distinct_generic_specialization_dispatch");
    fn test_behavior_json_generic_bound() => run_test("behavior_json_generic_bound");
    fn test_behavior_json_generic_bound_ufcs() => run_test("behavior_json_generic_bound_ufcs");
    fn test_behavior_generic_parent_inheritance() => run_test("behavior_generic_parent_inheritance");
    fn test_behavior_generic_parent_type_arg_inheritance() => run_test("behavior_generic_parent_type_arg_inheritance");
    fn test_behavior_inherited_generic_dispatch() => run_test("behavior_inherited_generic_dispatch");
}
