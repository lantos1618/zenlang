extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

#[test]
fn test_simple_return() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_simple_program(42);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_simple_return_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_simple_program(42);
        test_context.compile(&program).unwrap();
        let ir = test_context.get_ir().unwrap();
        assert!(ir.contains("ret i64 42"), "IR should contain 'ret i64 42'\nIR was:\n{}", ir);
    });
}

#[test]
fn test_binary_operations_execution() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_binary_op_program(40, BinaryOperator::Add, 2);
        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_binary_operations_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_binary_op_program(40, BinaryOperator::Add, 2);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_variable_declaration() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_variable_program("x", 42);
        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_variable_declaration_ir() {
    test_context!(|test_context: &mut TestContext| {
        let program = TestContext::create_variable_program("x", 42);
        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
} 