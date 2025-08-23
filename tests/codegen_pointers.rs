extern crate test_utils;

use zen::ast::{self, AstType, Expression, Statement, BinaryOperator, VariableDeclarationType};
use test_utils::TestContext;
use zen::error::CompileError;
use inkwell::context::Context;
use inkwell::OptimizationLevel;
use inkwell::execution_engine::JitFunction;
use test_utils::test_context;

// Helper function to compile and execute a program
fn compile_and_run<'ctx>(test_context: &mut TestContext<'ctx>, program: &ast::Program) -> i64 {
    test_context.compile(program).unwrap();
    let execution_engine = test_context.module().unwrap().create_jit_execution_engine(OptimizationLevel::None).unwrap();
    let jit_function: JitFunction<unsafe extern "C" fn() -> i64> = unsafe { execution_engine.get_function("main").unwrap() };
    let result = unsafe { jit_function.call() };
    drop(execution_engine); // Explicitly drop before returning
    result
}

#[test]
fn test_pointer_operations() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "p".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("p".to_string())))),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 42);
    });
}

#[test]
fn test_pointer_arithmetic() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![
                ("arr".to_string(), AstType::Pointer(Box::new(AstType::I64))),
            ],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::PointerOffset {
                        pointer: Box::new(Expression::Identifier("arr".to_string())),
                        offset: Box::new(Expression::Integer64(1)),
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
}

#[test]
fn test_pointer_assignment() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::I64))),
                    initializer: Some(Expression::AddressOf(Box::new(Expression::Identifier("x".to_string())))),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::PointerAssignment {
                    pointer: Expression::Identifier("ptr".to_string()),
                    value: Expression::Integer64(100),
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("ptr".to_string())))),
            ],
        }]);

        test_context.compile(&program).unwrap();
        let result = test_context.run().unwrap();
        assert_eq!(result, 100);
    });
}

#[test]
fn test_invalid_dereferencing_non_pointer() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "x".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::Integer64(42)),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Dereference(Box::new(Expression::Identifier("x".to_string())))),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_err());
        if let Err(CompileError::TypeMismatch { expected, found, .. }) = result {
            assert_eq!(expected, "pointer");
            assert!(found.contains("IntType"));
        } else {
            panic!("Expected TypeMismatch error");
        }
    });
}

#[test]
fn test_void_pointer_declaration() {
    test_context!(|test_context: &mut TestContext| {
        let program = ast::Program::from_functions(vec![ast::Function { type_params: vec![], is_async: false, 
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration { 
                    name: "ptr".to_string(),
                    type_: Some(AstType::Pointer(Box::new(AstType::Void))),
                    initializer: None,
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Integer64(0)),
            ],
        }]);

        let result = test_context.compile(&program);
        assert!(result.is_ok());
    });
} 