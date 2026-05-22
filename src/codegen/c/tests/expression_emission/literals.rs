use super::*;

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
fn emit_variable() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::Variable("count".into()), Type::I32);
    assert_eq!(e.emit_expr_inline(&expr), "count");
}
