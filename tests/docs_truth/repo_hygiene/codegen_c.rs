use super::*;

#[path = "codegen_c/generated_c_facade.rs"]
mod generated_c_facade;
#[path = "codegen_c/generated_c_support.rs"]
mod generated_c_support;

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
        operators.contains("op.symbol()"),
        "C operator spelling should reuse AST operator symbols instead of repeating a C-only table"
    );
    for duplicated_table_entry in ["BinaryOp::Add =>", "UnaryOp::Neg =>"] {
        assert!(
            !operators.contains(duplicated_table_entry),
            "C operator helper should not duplicate AST operator spelling: {duplicated_table_entry}"
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
fn codegen_c_does_not_lower_gated_enum_payload_mutation() {
    let lowering = read("src/codegen/c/intrinsics.rs");
    let spelling = read("src/codegen/c/intrinsics/names/spelling.rs");
    let gated = read("src/typechecker/gated_intrinsics/spelling.rs");

    assert!(
        !lowering.contains("memcpy((uint8_t*)({}) + sizeof(int32_t), {}, 0)"),
        "C codegen should not keep a zero-byte set_payload lowering placeholder"
    );
    assert!(
        !spelling.contains(r#"SetPayload => "set_payload""#),
        "set_payload should not be listed as an available C backend intrinsic until layout sizes exist"
    );
    assert!(
        gated.contains(r#"SetPayload => "set_payload""#),
        "set_payload should stay visible as a gated typechecker intrinsic"
    );
}
