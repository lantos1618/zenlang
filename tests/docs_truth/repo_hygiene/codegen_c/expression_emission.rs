use super::*;

#[test]
fn codegen_c_expression_operator_spelling_lives_in_focused_helper() {
    let emit = read("src/codegen/c/emit.rs");
    let statements = read("src/codegen/c/emit/statements.rs");
    let operators = read("src/codegen/c/operators.rs");
    let literals = read("src/codegen/c/literals.rs");
    let c_mod = read("src/codegen/c/mod.rs");

    assert!(
        emit.lines().count() < 170,
        "C expression emission should stay focused on inline expression routing"
    );
    assert!(
        emit.contains("mod statements;"),
        "C expression emission should load focused statement emission helper"
    );
    for helper in ["emit_block_body", "emit_statement", "emit_expr_to_stmt"] {
        assert!(
            !emit.contains(&format!("fn {helper}")),
            "C inline expression emitter should not own statement helper: {helper}"
        );
        assert!(
            statements.contains(&format!("fn {helper}")),
            "C statement emission should live in focused helper: {helper}"
        );
    }
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
