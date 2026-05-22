use super::*;

#[test]
fn typechecker_generic_method_resolution_lives_in_focused_helper() {
    let root = read("src/typechecker/expressions/method_call_support.rs");
    let generic_methods =
        read("src/typechecker/expressions/method_call_support/generic_methods.rs");

    assert!(
        root.lines().count() < 170,
        "method_call_support.rs should stay focused on method dispatch and UFC routing"
    );
    assert!(
        root.contains("mod generic_methods;"),
        "method call support should include focused generic method resolution helper"
    );
    assert!(
        !root.contains("fn resolve_generic_method_call"),
        "method-call dispatch should not own generic method specialization resolution"
    );
    assert!(
        generic_methods.contains("fn resolve_generic_method_call"),
        "generic method specialization resolution should live in focused helper"
    );
    assert!(
        generic_methods.contains("infer_method_type_args")
            && generic_methods.contains("specialize_generic_method"),
        "generic method helper should own inference and specialization flow"
    );
}
