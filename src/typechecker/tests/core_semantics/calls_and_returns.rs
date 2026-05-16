use super::*;

#[test]
fn unknown_function_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![Declaration::Function {
            name: "main".into(),
            type_params: Vec::new(),
            params: Vec::new(),
            return_type: Some(AstType::Void),
            body: Expression::Block {
                statements: vec![ast::Statement::Expression {
                    expr: Expression::FunctionCall {
                        name: "nonexistent".into(),
                        module: None,
                        type_args: Vec::new(),
                        args: Vec::new(),
                        span: Span::dummy(),
                    },
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
    let result = tc.check_program(&program);
    assert!(result.is_err());
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|d| d.message.contains("undefined function")));
}

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
                expr: Some(Box::new(Expression::Return {
                    value: Some(Box::new(Expression::StringLiteral {
                        value: "hello".into(),
                        span: Span::dummy(),
                    })),
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

#[test]
fn function_call_wrong_arity_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "add".into(),
                type_params: Vec::new(),
                params: vec![
                    ast::Param {
                        name: "a".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                    ast::Param {
                        name: "b".into(),
                        ty: AstType::I32,
                        mutable: false,
                        span: Span::dummy(),
                    },
                ],
                return_type: Some(AstType::I32),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: Some(Box::new(Expression::Return {
                        value: Some(Box::new(Expression::Identifier {
                            name: "a".into(),
                            span: Span::dummy(),
                        })),
                        span: Span::dummy(),
                    })),
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "add".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::IntLiteral {
                                value: 1,
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("wrong arity should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("function `add` expects 2 arguments, found 1")),
        "expected arity diagnostic, got {errors:?}"
    );
}

#[test]
fn function_call_argument_type_mismatch_is_error() {
    use crate::ast::{Expression, Program};
    let mut tc = TypeChecker::new();
    let program = Program {
        declarations: vec![
            Declaration::Function {
                name: "takes_i32".into(),
                type_params: Vec::new(),
                params: vec![ast::Param {
                    name: "value".into(),
                    ty: AstType::I32,
                    mutable: false,
                    span: Span::dummy(),
                }],
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: Vec::new(),
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
            Declaration::Function {
                name: "main".into(),
                type_params: Vec::new(),
                params: Vec::new(),
                return_type: Some(AstType::Void),
                body: Expression::Block {
                    statements: vec![ast::Statement::Expression {
                        expr: Expression::FunctionCall {
                            name: "takes_i32".into(),
                            module: None,
                            type_args: Vec::new(),
                            args: vec![Expression::StringLiteral {
                                value: "bad".into(),
                                span: Span::dummy(),
                            }],
                            span: Span::dummy(),
                        },
                        span: Span::dummy(),
                    }],
                    expr: None,
                    span: Span::dummy(),
                },
                public: false,
                span: Span::dummy(),
            },
        ],
        file_id: 0,
    };

    let errors = tc
        .check_program(&program)
        .expect_err("argument type mismatch should fail");
    assert!(
        errors.iter().any(|d| d
            .message
            .contains("argument 1 for `takes_i32` expects `i32`, found `str`")),
        "expected argument type diagnostic, got {errors:?}"
    );
}
