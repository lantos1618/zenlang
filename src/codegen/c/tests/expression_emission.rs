use super::*;
use crate::ast::expressions::BinaryOp;

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
    assert_eq!(e.emit_expr_inline(&expr), "zen_str_from_literal(\"hi\")");
}

#[test]
fn emit_variable() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::Variable("count".into()), Type::I32);
    assert_eq!(e.emit_expr_inline(&expr), "count");
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
    use crate::ast::expressions::UnaryOp;
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
fn emit_function_call() {
    let mut e = CEmitter::new();
    let args = vec![
        texpr(TypedExprKind::IntLiteral(1), Type::I32),
        texpr(TypedExprKind::IntLiteral(2), Type::I32),
    ];
    let expr = texpr(
        TypedExprKind::FunctionCall {
            function: "add".into(),
            args,
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&expr), "add(1LL, 2LL)");
}

#[test]
fn emit_field_access() {
    let mut e = CEmitter::new();
    let obj = Box::new(texpr(
        TypedExprKind::Variable("p".into()),
        Type::Struct {
            name: "Point".into(),
            fields: vec![],
        },
    ));
    let expr = texpr(
        TypedExprKind::FieldAccess {
            object: obj,
            field: "x".into(),
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&expr), "p.x");
}

#[test]
fn emit_field_access_through_ptr() {
    let mut e = CEmitter::new();
    let obj = Box::new(texpr(
        TypedExprKind::Variable("p".into()),
        Type::Ptr(Box::new(Type::Struct {
            name: "Point".into(),
            fields: vec![],
        })),
    ));
    let expr = texpr(
        TypedExprKind::FieldAccess {
            object: obj,
            field: "x".into(),
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&expr), "p->x");
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

#[test]
fn emit_ref_deref() {
    let mut e = CEmitter::new();
    let var = texpr(TypedExprKind::Variable("x".into()), Type::I32);
    let r = texpr(
        TypedExprKind::Ref(Box::new(var.clone())),
        Type::Ptr(Box::new(Type::I32)),
    );
    assert_eq!(e.emit_expr_inline(&r), "(&x)");

    let d = texpr(
        TypedExprKind::Deref(Box::new(texpr(
            TypedExprKind::Variable("p".into()),
            Type::Ptr(Box::new(Type::I32)),
        ))),
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&d), "(*p)");
}

#[test]
fn emit_index_access() {
    let mut e = CEmitter::new();
    let obj = Box::new(texpr(
        TypedExprKind::Variable("arr".into()),
        Type::Slice(Box::new(Type::I32)),
    ));
    let idx = Box::new(texpr(TypedExprKind::IntLiteral(3), Type::I32));
    let expr = texpr(
        TypedExprKind::IndexAccess {
            object: obj,
            index: idx,
        },
        Type::I32,
    );
    assert_eq!(e.emit_expr_inline(&expr), "arr[3LL]");
}

#[test]
fn emit_struct_literal() {
    let mut e = CEmitter::new();
    let expr = texpr(
        TypedExprKind::StructLiteral {
            type_name: "Point".into(),
            fields: vec![
                ("x".into(), texpr(TypedExprKind::IntLiteral(1), Type::I32)),
                ("y".into(), texpr(TypedExprKind::IntLiteral(2), Type::I32)),
            ],
        },
        Type::Struct {
            name: "Point".into(),
            fields: vec![],
        },
    );
    assert_eq!(e.emit_expr_inline(&expr), "(Point){ .x = 1LL, .y = 2LL }");
}

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
