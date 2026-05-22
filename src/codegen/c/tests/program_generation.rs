use super::*;
use crate::ast::expressions::BinaryOp;

mod aggregates;
mod functions_and_runtime;

fn make_simple_program() -> TypedProgram {
    TypedProgram {
        functions: vec![TypedFunction {
            name: "add".into(),
            params: vec![
                TypedParam {
                    name: "a".into(),
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                },
                TypedParam {
                    name: "b".into(),
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                },
            ],
            return_type: Type::I32,
            body: TypedBlock {
                statements: vec![],
                expr: Some(Box::new(TypedExpression {
                    kind: TypedExprKind::BinaryOp {
                        op: BinaryOp::Add,
                        left: Box::new(TypedExpression {
                            kind: TypedExprKind::Variable("a".into()),
                            ty: Type::I32,
                            span: crate::error::Span::dummy(),
                        }),
                        right: Box::new(TypedExpression {
                            kind: TypedExprKind::Variable("b".into()),
                            ty: Type::I32,
                            span: crate::error::Span::dummy(),
                        }),
                    },
                    ty: Type::I32,
                    span: crate::error::Span::dummy(),
                })),
                ty: Type::I32,
                span: crate::error::Span::dummy(),
            },
            defers: vec![],
            span: crate::error::Span::dummy(),
        }],
        types: vec![],
        globals: vec![],
        entry_point: None,
    }
}
