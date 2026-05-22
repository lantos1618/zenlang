use super::*;

#[test]
fn generated_c_test_support_splits_definition_and_call_scanning() {
    let root = read("tests/integration/support/generated_c.rs");
    let definitions = read("tests/integration/support/generated_c/definitions.rs");
    let calls = read("tests/integration/support/generated_c/calls.rs");
    let call_scan = read("tests/integration/support/generated_c/calls/scan.rs");
    let call_signatures = read("tests/integration/support/generated_c/calls/signatures.rs");
    let function_pointers =
        read("tests/integration/support/generated_c/calls/function_pointers.rs");

    for module in [
        "#[path = \"generated_c/calls.rs\"]",
        "#[path = \"generated_c/definitions.rs\"]",
    ] {
        assert!(
            root.contains(module),
            "generated_c.rs should load focused support module `{module}`"
        );
    }
    assert!(
        !root.contains("fn c_function_definitions"),
        "generated_c.rs should not own generated C definition scanning"
    );
    assert!(
        definitions.contains("pub(super) fn c_function_definitions"),
        "definitions.rs should own generated C definition scanning"
    );
    assert!(
        calls.lines().count() < 80,
        "calls.rs should route focused generated C call-check helpers"
    );
    for module in [
        "#[path = \"calls/function_pointers.rs\"]",
        "#[path = \"calls/scan.rs\"]",
        "#[path = \"calls/signatures.rs\"]",
    ] {
        assert!(
            calls.contains(module),
            "calls.rs should load focused generated-C call helper `{module}`"
        );
    }
    for helper in [
        "fn c_function_pointer_bindings",
        "fn generated_c_calls_on_line",
        "fn is_any_c_function_signature_line",
    ] {
        assert!(
            !calls.contains(helper),
            "calls.rs should route helper `{helper}` to a focused generated-C call submodule"
        );
    }
    assert!(
        call_scan.contains("pub fn undefined_generated_c_calls"),
        "calls.rs should own generated C call/definition consistency scanning"
    );
    assert!(
        call_scan.contains("fn generated_c_calls_on_line"),
        "scan.rs should own generated C call extraction"
    );
    assert!(
        function_pointers.contains("pub(super) fn c_function_pointer_bindings"),
        "function_pointers.rs should own generated C function-pointer binding detection"
    );
    assert!(
        call_signatures.contains("pub fn has_c_call_outside_signature"),
        "signatures.rs should own generated C signature filtering"
    );
}
