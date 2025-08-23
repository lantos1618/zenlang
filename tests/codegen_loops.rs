extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;
use std::time::{Duration, Instant};
use std::process;

// Helper function to run tests with timeout
fn run_with_timeout<F>(timeout_secs: u64, test_fn: F) -> Result<(), Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<(), Box<dyn std::error::Error>>,
{
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);
    
    // Run the test
    let result = test_fn();
    
    // Check if we exceeded timeout
    if start.elapsed() > timeout {
        eprintln!("Test timed out after {} seconds - killing process", timeout_secs);
        process::exit(1);
    }
    
    result
}

#[test]
fn test_comparison_operator() {
    run_with_timeout(5, || {
        test_context!(|test_context: &mut TestContext| {
            let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
                name: "main".to_string(),
                args: vec![
                    ("n".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(3)),
                        op: BinaryOperator::LessThan,
                        right: Box::new(Expression::Integer64(5)),
                    }),
                ],
            }]);

            test_context.compile(&program).unwrap();
            let result = test_context.run().unwrap();
            assert_eq!(result, 1); // 3 < 5 should be true (1)
        });
        Ok(())
    }).unwrap();
}

#[test]
fn test_simple_binary_op() {
    run_with_timeout(5, || {
        test_context!(|test_context: &mut TestContext| {
            let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
                name: "main".to_string(),
                args: vec![
                    ("n".to_string(), AstType::I64),
                ],
                return_type: AstType::I64,
                body: vec![
                    Statement::Return(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(5)),
                        op: BinaryOperator::Add,
                        right: Box::new(Expression::Integer64(3)),
                    }),
                ],
            }]);

            test_context.compile(&program).unwrap();
            let result = test_context.run().unwrap();
            assert_eq!(result, 8);
        });
        Ok(())
    }).unwrap();
}

#[test]
fn test_loop_construct() {
    run_with_timeout(5, || {
        test_context!(|test_context: &mut TestContext| {
            let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::VariableDeclaration { 
                        name: "sum".to_string(),
                        type_: Some(AstType::I64),
                        initializer: Some(Expression::Integer64(0)),
                        is_mutable: true,
                        declaration_type: VariableDeclarationType::ExplicitMutable,
                    },
                    Statement::VariableDeclaration { 
                        name: "i".to_string(),
                        type_: Some(AstType::I64),
                        initializer: Some(Expression::Integer64(0)),
                        is_mutable: true,
                        declaration_type: VariableDeclarationType::ExplicitMutable,
                    },
                    Statement::Loop {
                        condition: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("i".to_string())),
                            op: BinaryOperator::LessThan,
                            right: Box::new(Expression::Integer64(5)),
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
        Ok(())
    }).unwrap();
} 