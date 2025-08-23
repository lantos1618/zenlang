extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_undefined_variable() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "test_undefined".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Identifier("x".to_string()))],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        if let Err(CompileError::UndeclaredVariable(name, _)) = result {
            assert_eq!(name, "x");
        } else {
            panic!("Expected UndeclaredVariable error");
        }
    });
}

#[test]
fn test_undefined_function() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::FunctionCall {
                name: "undefined_func".to_string(),
                args: vec![],
            })],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        if let Err(CompileError::UndeclaredFunction(name, _)) = result {
            assert_eq!(name, "undefined_func");
        } else {
            panic!("Expected UndeclaredFunction error");
        }
    });
}

#[test]
fn test_type_mismatch() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "test_type_mismatch".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::BinaryOp {
                left: Box::new(Expression::Integer64(42)),
                op: BinaryOperator::Add,
                right: Box::new(Expression::String("hello".to_string())),
            })],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        if let Err(CompileError::TypeMismatch { expected, found, .. }) = result {
            assert_eq!(expected, "i64");
            assert!(found.contains("String"));
        } else {
            panic!("Expected TypeMismatch error");
        }
    });
}

#[test]
fn test_invalid_function_type() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![
                ("x".to_string(), AstType::Function {
                    args: vec![AstType::I64],
                    return_type: Box::new(AstType::I64),
                }),
            ],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(42))],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
    });
} 