extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, ConditionalArm, Pattern, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_conditional_expression() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::Return(Expression::Conditional {
                    scrutinee: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Integer64(1)),
                        op: BinaryOperator::Equals,
                        right: Box::new(Expression::Integer64(1)),
                    }),
                    arms: vec![
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(1)), 
                            guard: None, 
                            body: Expression::Integer64(42) 
                        },
                        ConditionalArm { 
                            pattern: Pattern::Literal(Expression::Integer64(0)), 
                            guard: None, 
                            body: Expression::Integer64(0) 
                        },
                    ],
                }),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
} 