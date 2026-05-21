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
fn ast_expression_parts_live_in_focused_helper() {
    let expressions = read("src/ast/expressions.rs");
    let parts = read("src/ast/expressions/expression_parts.rs");

    for helper in ["StringPart", "MatchArm"] {
        assert!(
            !expressions.contains(&format!("pub enum {helper}"))
                && !expressions.contains(&format!("pub struct {helper}")),
            "expression AST root should not own recursive helper type: {helper}"
        );
        assert!(
            parts.contains(&format!("pub enum {helper}"))
                || parts.contains(&format!("pub struct {helper}")),
            "expression AST helper type should live in focused helper: {helper}"
        );
    }

    assert!(
        expressions.contains("mod expression_parts;"),
        "expression AST root should include focused expression parts module"
    );
    assert!(
        expressions.contains("pub use expression_parts::{MatchArm, StringPart};"),
        "expression AST root should preserve public re-exports for expression parts"
    );
    assert!(
        expressions.lines().count() < 210,
        "expressions.rs should stay focused on the Expression enum shell"
    );
}

#[test]
fn ast_expression_span_helper_lives_in_focused_helper() {
    let expressions = read("src/ast/expressions.rs");
    let span = read("src/ast/expressions/span.rs");

    assert!(
        !expressions.contains("pub fn span(&self) -> Span"),
        "expression AST root should not own span dispatch"
    );
    assert!(
        span.contains("pub fn span(&self) -> Span"),
        "expression span dispatch should live in focused helper"
    );
    assert!(
        expressions.contains("mod span;"),
        "expression AST root should include focused span helper"
    );
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

#[test]
fn typed_ast_declaration_parts_live_in_focused_helper() {
    let typed = read("src/ast/typed.rs");
    let declarations = read("src/ast/typed/declarations.rs");

    for helper in [
        "TypedFunction",
        "TypedParam",
        "TypedTypeDef",
        "TypeDefKind",
        "TypedVariant",
        "TypedGlobal",
        "TypedProgram",
    ] {
        assert!(
            !typed.contains(&format!("pub struct {helper}"))
                && !typed.contains(&format!("pub enum {helper}")),
            "typed AST root should not own declaration/program helper: {helper}"
        );
        assert!(
            declarations.contains(&format!("pub struct {helper}"))
                || declarations.contains(&format!("pub enum {helper}")),
            "typed AST declaration/program helper should live in focused helper: {helper}"
        );
    }

    assert!(
        typed.contains("mod declarations;"),
        "typed AST root should include focused declaration helper"
    );
    assert!(
        typed.contains("pub use declarations::{"),
        "typed AST root should preserve public re-exports for declarations"
    );
    assert!(
        typed.lines().count() < 170,
        "typed.rs should stay focused on typed expression, statement, and block shells"
    );
}
