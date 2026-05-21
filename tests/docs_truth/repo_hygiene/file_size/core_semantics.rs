use super::super::*;

#[test]
fn core_semantics_module_call_tests_live_in_focused_helper() {
    let mixed = read("src/typechecker/tests/core_semantics/enum_assignment_and_modules.rs");
    let module_calls = read("src/typechecker/tests/core_semantics/module_calls.rs");
    let module = read("src/typechecker/tests/core_semantics.rs");

    assert!(
        mixed.lines().count() < 220,
        "enum_assignment_and_modules.rs should stay focused on enum, assignment, conversion, and fallthrough semantics"
    );
    assert!(
        !mixed.contains("unknown_root_std_module_call_is_error"),
        "std module-call tests should live in module_calls.rs"
    );
    assert!(
        module_calls.contains("unknown_root_std_module_call_is_error"),
        "module_calls.rs should cover rejected std module calls"
    );
    assert!(
        module_calls.contains("known_root_std_runtime_standins_remain_allowed"),
        "module_calls.rs should cover temporary std runtime stand-ins"
    );
    assert!(
        module.contains("mod module_calls;"),
        "core_semantics.rs should include the focused module_calls module"
    );
}
