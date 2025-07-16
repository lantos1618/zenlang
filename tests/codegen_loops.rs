extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_loop_construct() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![
                ("n".to_string(), AstType::I64),
            ],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "sum".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(0)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "i".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(0)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Loop {
                    condition: Some(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("i".to_string())),
                        op: BinaryOperator::LessThan,
                        right: Box::new(Expression::Identifier("n".to_string())),
                    }),

                    label: None,
                    body: vec![
                        Statement::VariableAssignment {
                            name: "sum".to_string(),
                            value: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("sum".to_string())),
                                op: BinaryOperator::Add,
                                right: Box::new(Expression::Identifier("i".to_string())),
                            },
                        },
                        Statement::VariableAssignment {
                            name: "i".to_string(),
                            value: Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("i".to_string())),
                                op: BinaryOperator::Add,
                                right: Box::new(Expression::Integer64(1)),
                            },
                        },
                    ],
                },
                Statement::Return(Expression::Identifier("sum".to_string())),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 10); // Sum of 0+1+2+3+4 = 10
    });
} 