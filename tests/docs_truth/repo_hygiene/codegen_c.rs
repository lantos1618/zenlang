use super::*;

#[test]
fn codegen_c_function_emission_lives_in_focused_helper() {
    let types = read("src/codegen/c/types.rs");
    let functions = read("src/codegen/c/functions.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    for helper in [
        "emit_function_forward_decl",
        "emit_function",
        "format_params",
        "emit_global",
    ] {
        assert!(
            !types.contains(&format!("fn {helper}")),
            "C type emission should not own function/global helper: {helper}"
        );
        assert!(
            functions.contains(&format!("fn {helper}")),
            "C function/global emission should live in focused helper: {helper}"
        );
    }

    assert!(
        c_mod.contains("mod functions;"),
        "C codegen should load focused function/global emission helper"
    );
}

#[test]
fn codegen_c_expression_operator_spelling_lives_in_focused_helper() {
    let emit = read("src/codegen/c/emit.rs");
    let operators = read("src/codegen/c/operators.rs");
    let literals = read("src/codegen/c/literals.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    assert!(
        emit.lines().count() < 240,
        "C expression emission should stay focused on expression routing"
    );
    for helper in ["fn c_binary_op", "fn c_unary_op"] {
        assert!(
            !emit.contains(helper),
            "C expression emitter should not own operator spelling helper: {helper}"
        );
        assert!(
            operators.contains(helper),
            "C operator spelling should live in focused helper: {helper}"
        );
    }
    assert!(
        c_mod.contains("mod operators;"),
        "C codegen should load focused operator spelling helper"
    );
    for helper in [
        "emit_struct_literal",
        "emit_enum_variant_literal",
        "emit_array_literal",
    ] {
        assert!(
            !emit.contains(&format!("fn {helper}")),
            "C expression emitter should route aggregate literals to focused helper: {helper}"
        );
        assert!(
            literals.contains(&format!("fn {helper}")),
            "C aggregate literal emission should live in focused helper: {helper}"
        );
    }
    assert!(
        c_mod.contains("mod literals;"),
        "C codegen should load focused aggregate literal helper"
    );
}

#[test]
fn generated_c_test_support_splits_definition_and_call_scanning() {
    let root = read("tests/integration/support/generated_c.rs");
    let definitions = read("tests/integration/support/generated_c/definitions.rs");
    let calls = read("tests/integration/support/generated_c/calls.rs");

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
        calls.contains("pub fn undefined_generated_c_calls"),
        "calls.rs should own generated C call/definition consistency scanning"
    );
    assert!(
        calls.contains("fn c_function_pointer_bindings"),
        "calls.rs should own generated C function-pointer binding detection"
    );
}
