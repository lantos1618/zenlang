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

#[test]
fn typed_ast_expression_parts_live_in_focused_helper() {
    let typed = read("src/ast/typed.rs");
    let parts = read("src/ast/typed/expression_parts.rs");

    for helper in [
        "MatchKind",
        "TypedMatchArm",
        "TypedPattern",
        "Capture",
        "TypedStringPart",
    ] {
        assert!(
            !typed.contains(&format!("pub struct {helper}"))
                && !typed.contains(&format!("pub enum {helper}")),
            "typed AST root should not own expression part helper: {helper}"
        );
        assert!(
            parts.contains(&format!("pub struct {helper}"))
                || parts.contains(&format!("pub enum {helper}")),
            "typed AST expression part helper should live in focused helper: {helper}"
        );
    }

    assert!(
        typed.contains("mod expression_parts;"),
        "typed AST root should include focused expression parts module"
    );
    assert!(
        typed.contains("pub use expression_parts::{Capture, MatchKind, TypedMatchArm, TypedPattern, TypedStringPart};"),
        "typed AST root should preserve public re-exports for expression parts"
    );
    assert!(
        typed.lines().count() < 230,
        "typed.rs should stay focused on typed expression, declaration, and program shells"
    );
}
