use super::*;

#[test]
fn return_type_mismatch_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "foo".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::I32),
            body: Expression::Block {
                statements: Vec::new(),
                expr: Some(Box::new(Expression::StringLiteral {
                    value: "hello".into(),
                    span: Span::dummy(),
                })),
                span: Span::dummy(),
            },
            public: false,
            span: Span::dummy(),
        }],
        file_id: 0,
    };
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("return type mismatch")));
}
