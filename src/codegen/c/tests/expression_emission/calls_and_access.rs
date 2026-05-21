use super::*;

#[test]
fn emit_variable() {
    let mut e = CEmitter::new();
    let expr = texpr(TypedExprKind::Variable("count".into()), Type::I32);
    assert_eq!(e.emit_expr_inline(&expr), "count");
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
