extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_function_call() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "add".to_string(),
                args: vec![
                    ("a".to_string(), AstType::I64),
                    ("b".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            },
            ast::Function { 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: "add".to_string(),
                    args: vec![
                        Expression::Integer64(40),
                        Expression::Integer64(2),
                    ],
                })],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_function_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "add".to_string(),
                args: vec![
                    ("a".to_string(), AstType::I64),
                    ("b".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOperator::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                })],
            },
            ast::Function { 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "op".to_string(),
                        type_: Some(AstType::Function {
                            args: vec![AstType::I64, AstType::I64],
                            return_type: Box::new(AstType::I64),
                        }),
                        initializer: Some(Expression::Identifier("add".to_string())),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::FunctionCall {
                        name: "op".to_string(),
                        args: vec![
                            Expression::Integer64(40),
                            Expression::Integer64(2),
                        ],
                    }),
                ],
            },
        ]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_recursive_function() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { 
                is_async: false,
                name: "factorial".to_string(),
                args: vec![
                    ("n".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("n".to_string())),
                        op: BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(0)),
                    }),
                    arms: vec![
                        ast::ConditionalArm { 
                            pattern: ast::Pattern::Literal(Expression::Integer64(1)), 
                            guard: None, 
                            body: Expression::Integer64(1) 
                        },
                        ast::ConditionalArm { 
                            pattern: ast::Pattern::Literal(Expression::Integer64(0)), 
                            guard: None, 
                            body: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("n".to_string())),
                                op: BinaryOperator::Multiply,
                                right: Box::new(Expression::FunctionCall {
                                    name: "factorial".to_string(),
                                    args: vec![Expression::BinaryOp {
                                        left: Box::new(Expression::Identifier("n".to_string())),
                                        op: BinaryOperator::Subtract,
                                        right: Box::new(Expression::Integer64(1)),
                                    }],
                                }),
                            }
                        },
                    ],
                })],
            },
            ast::Function { 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: "factorial".to_string(),
                    args: vec![Expression::Integer64(5)],
                })],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 120);
    });
} 