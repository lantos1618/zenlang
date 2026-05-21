use super::*;

#[test]
fn emit_var_decl_mutable_vs_const() {
    let mut e = CEmitter::new();
    let stmt_const = TypedStatement {
        kind: TypedStatementKind::VarDecl {
            name: "x".into(),
            ty: Type::I32,
            value: texpr(TypedExprKind::IntLiteral(42), Type::I32),
            mutable: false,
        },
        span: dummy(),
    };
    e.emit_statement(&stmt_const);
    assert!(e.output.contains("const int32_t x = 42LL;"));

    let mut e = CEmitter::new();
    let stmt_mut = TypedStatement {
        kind: TypedStatementKind::VarDecl {
            name: "y".into(),
            ty: Type::I32,
            value: texpr(TypedExprKind::IntLiteral(0), Type::I32),
            mutable: true,
        },
        span: dummy(),
    };
    e.emit_statement(&stmt_mut);
    assert!(e.output.contains("int32_t y = 0LL;"));
    assert!(!e.output.contains("const"));
}
