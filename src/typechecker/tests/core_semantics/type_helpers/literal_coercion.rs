use super::*;

#[test]
fn literal_coercion_in_var_decl() {
    use crate::ast::{Expression, Program, Statement};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![Statement::VarDecl {
                    name: "x".into(),
                    ty: Some(AstType::I64),
                    value: Expression::IntLiteral {
                        value: 42,
                        span: Span::dummy(),
                    },
                    mutable: false,
                    constant: false,
                    span: Span::dummy(),
                }],
                expr: None,
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program).unwrap();
    // The variable should have type I64 (coerced from I32 literal)
    let body = &result.functions[0].body;
    match &body.statements[0].kind {
        TypedStatementKind::VarDecl { ty, .. } => assert_eq!(*ty, Type::I64),
        _ => panic!("expected VarDecl"),
    }
}
