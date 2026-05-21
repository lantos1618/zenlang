use super::*;
use crate::ast::expressions::{BinaryOp, UnaryOp};

#[test]
fn emit_int_literal() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::IntLiteral(42), Type::I32);
    assert_eq!(e.emit_expr_inline(&expr), "42LL");
}

#[test]
fn emit_float_literal() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::FloatLiteral(3.14), Type::F64);
    assert_eq!(e.emit_expr_inline(&expr), "3.14");
}

#[test]
fn emit_bool_literal() {
    let mut e = CEmitter::new();
    assert_eq!(
        e.emit_expr_inline(&texpr(TypedExprKind::BoolLiteral(true), Type::Bool)),
        "true"
    );
    assert_eq!(
        e.emit_expr_inline(&texpr(TypedExprKind::BoolLiteral(false), Type::Bool)),
        "false"
    );
}

#[test]
fn emit_string_literal() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::StringLiteral("hi".into()), Type::Str);
    assert_eq!(
        e.emit_expr_inline(&expr),
        "(zen_str){ .ptr = \"hi\", .len = sizeof(\"hi\") - 1 }"
    );
}

#[test]
fn emit_binary_ops() {
    let mut e = CEmitter::new();
    let left = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
    let right = Box::new(texpr(TypedExprKind::IntLiteral(1), Type::I32));
    let expr = texpr(
        TypedExprKind::BinaryOp {
            op: BinaryOp::Sub,
            left,
            right,
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&expr), "(x - 1LL)");
}

#[test]
fn emit_unary_ops() {
    let mut e = CEmitter::new();

    let operand = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
    let neg = texpr(
        TypedExprKind::UnaryOp {
            op: UnaryOp::Neg,
            operand,
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&neg), "(-x)");

    let operand = Box::new(texpr(TypedExprKind::Variable("b".into()), Type::Bool));
    let not = texpr(
        TypedExprKind::UnaryOp {
            op: UnaryOp::Not,
            operand,
        },
        Type::Bool,
    );
    assert_eq!(e.emit_expr_inline(&not), "(!b)");
}

#[test]
fn emit_cast() {
    let mut e = CEmitter::new();
    let inner = Box::new(texpr(TypedExprKind::Variable("x".into()), Type::I32));
    let expr = texpr(
        TypedExprKind::Cast {
            expr: inner,
            to_type: Type::I64,
            from_type: Type::I32,
        },
        Type::I64,
    );
    assert_eq!(e.emit_expr_inline(&expr), "((int64_t)x)");
}
