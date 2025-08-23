extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_string_concatenation() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { type_params: vec![], 
                is_async: false,
                name: "concat_strings".to_string(),
                args: vec![
                    ("s1".to_string(), AstType::String),
                    ("s2".to_string(), AstType::String),
                ],
                return_type: AstType::String,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("s1".to_string())),
                    op: BinaryOperator::StringConcat,
                    right: Box::new(Expression::Identifier("s2".to_string())),
                })],
            },
            ast::Function { type_params: vec![], 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "result".to_string(),
                        type_: Some(AstType::String),
                        initializer: Some(Expression::FunctionCall {
                            name: "concat_strings".to_string(),
                            args: vec![
                                Expression::String("Hello, ".to_string()),
                                Expression::String("World!".to_string()),
                            ],
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::Integer64(0)),
                ],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 0);
    });
}

#[test]
fn test_string_comparison() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![
            ast::Function { type_params: vec![], 
                is_async: false,
                name: "compare_strings".to_string(),
                args: vec![
                    ("s1".to_string(), AstType::String),
                    ("s2".to_string(), AstType::String),
                ],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("s1".to_string())),
                    op: BinaryOperator::Equals,
                    right: Box::new(Expression::Identifier("s2".to_string())),
                })],
            },
            ast::Function { type_params: vec![], 
                is_async: false,
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![Statement::Return(Expression::FunctionCall {
                    name: "compare_strings".to_string(),
                    args: vec![
                        Expression::String("hello".to_string()),
                        Expression::String("hello".to_string()),
                    ],
                })],
            },
        ]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 1); // Should return true (1) for equal strings
    });
}

#[test]
fn test_string_length() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "s".to_string(),
                    type_: Some(AstType::String),
                    initializer: Some(Expression::String("hello".to_string())),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::StringLength(Box::new(Expression::Identifier("s".to_string())))),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 5);
    });
}

#[test]
fn test_string_literal_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(42))],
        }]);

        test_context.compile(&program).unwrap();
        let ir = test_context.get_ir().unwrap();
        assert!(ir.contains("ret i64 42"), "IR should contain 'ret i64 42'\nIR was:\n{}", ir);
    });
} 