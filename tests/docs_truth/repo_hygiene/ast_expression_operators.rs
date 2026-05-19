use super::*;

#[test]
fn ast_expression_operator_spelling_lives_in_focused_helper() {
    let expressions = read("src/ast/expressions.rs");
    let operators = read("src/ast/expressions/operators.rs");

    for helper in ["BinaryOp", "UnaryOp", "LoopControlAction"] {
        assert!(
            !expressions.contains(&format!("pub enum {helper}")),
            "expression AST root should not own operator/control spelling enum: {helper}"
        );
        assert!(
            operators.contains(&format!("pub enum {helper}")),
            "operator/control spelling enum should live in focused helper: {helper}"
        );
    }

    for required in [
        "mod operators;",
        "pub use operators::{BinaryOp, LoopControlAction, UnaryOp};",
    ] {
        assert!(
            expressions.contains(required),
            "expression AST root should re-export focused operator helper: {required}"
        );
    }
}
