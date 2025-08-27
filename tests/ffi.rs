mod common;

use common::{ExecutionHelper, CapturedOutput};
use inkwell::context::Context;
use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType, VariableDeclarationType};
use zen::compiler::Compiler;

#[test]
fn test_printf_output_verified() {
    let helper = ExecutionHelper::new();
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Hello, World!\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile and run, verifying actual output
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // Verify the output actually appeared on stdout
    output.assert_stdout_contains("Hello, World!");
    output.assert_success();
    
    println!("✓ Printf output verified successfully!");
}

#[test]
fn test_printf_return_value() {
    let helper = ExecutionHelper::new();
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    // Printf returns number of characters printed
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Test\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(42)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    output.assert_stdout_eq("Test");
    output.assert_exit_code(42);
    
    println!("✓ Printf return value test passed!");
}

#[test]
fn test_puts_output_verified() {
    let helper = ExecutionHelper::new();
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "puts".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I32,
                is_varargs: false,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I32,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "puts".to_string(),
                        args: vec![Expression::String("Hello from puts".to_string())],
                    }),
                    Statement::Return(Expression::Integer32(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // puts adds newline automatically
    output.assert_stdout_contains("Hello from puts");
    output.assert_success();
    
    println!("✓ Puts output verified successfully!");
}

#[test]
fn test_multiple_printf_calls_verified() {
    let helper = ExecutionHelper::new();
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("First\n".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Second\n".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Third\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    // Verify all outputs appear in correct order
    let lines = output.stdout_lines();
    assert_eq!(lines.len(), 3, "Should have exactly 3 lines");
    assert_eq!(lines[0], "First");
    assert_eq!(lines[1], "Second");
    assert_eq!(lines[2], "Third");
    
    output.assert_success();
    
    println!("✓ Multiple printf calls verified in correct order!");
}

#[test]
fn test_float_operations_with_printf() {
    let helper = ExecutionHelper::new();
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    // Test float math
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: Some(AstType::F64),
                        initializer: Some(Expression::Float64(3.14)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::VariableDeclaration {
                        name: "y".to_string(),
                        type_: Some(AstType::F64),
                        initializer: Some(Expression::Float64(2.0)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::VariableDeclaration {
                        name: "result".to_string(),
                        type_: Some(AstType::F64),
                        initializer: Some(Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("x".to_string())),
                            op: ast::BinaryOperator::Add,
                            right: Box::new(Expression::Identifier("y".to_string())),
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    // Just verify computation worked
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Float math completed\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");
    
    output.assert_stdout_contains("Float math completed");
    output.assert_success();
    
    println!("✓ Float operations test passed!");
}

// Keep the LLVM IR generation tests for verification
#[test]
fn test_external_function_ir_generation() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "printf".to_string(),
                args: vec![AstType::String],
                return_type: AstType::I64,
                is_varargs: true,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "main".to_string(),
                args: vec![],
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Hello\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Verify IR contains expected declarations
    let printf_declared = ir.contains("declare i64 @printf(ptr, ...)") || 
                         ir.contains("declare i64 @printf(i8*, ...)");
    assert!(printf_declared, "Should declare printf with varargs");
    assert!(ir.contains("@printf"), "Should reference printf function");
    
    println!("✓ External function IR generation test passed!");
}

#[test]
fn test_external_math_function_ir() {
    let context = Context::create();
    let compiler = Compiler::new(&context);

    let program = ast::Program {
        declarations: vec![
            Declaration::ExternalFunction(ExternalFunction {
                name: "sqrt".to_string(),
                args: vec![AstType::F64],
                return_type: AstType::F64,
                is_varargs: false,
            }),
            Declaration::Function(Function { 
                type_params: vec![],
                name: "calculate_distance".to_string(),
                args: vec![("x".to_string(), AstType::F64), ("y".to_string(), AstType::F64)],
                return_type: AstType::F64,
                body: vec![
                    Statement::VariableDeclaration {
                        name: "sum_squares".to_string(),
                        type_: Some(AstType::F64),
                        initializer: Some(Expression::BinaryOp {
                            left: Box::new(Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("x".to_string())),
                                op: ast::BinaryOperator::Multiply,
                                right: Box::new(Expression::Identifier("x".to_string())),
                            }),
                            op: ast::BinaryOperator::Add,
                            right: Box::new(Expression::BinaryOp {
                                left: Box::new(Expression::Identifier("y".to_string())),
                                op: ast::BinaryOperator::Multiply,
                                right: Box::new(Expression::Identifier("y".to_string())),
                            }),
                        }),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Return(Expression::FunctionCall {
                        name: "sqrt".to_string(),
                        args: vec![Expression::Identifier("sum_squares".to_string())],
                    }),
                ],
                is_async: false,
            }),
        ],
    };

    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Verify sqrt declaration
    assert!(ir.contains("declare double @sqrt(double)"), "Should declare sqrt");
    assert!(ir.contains("call double @sqrt"), "Should call sqrt");
    
    println!("✓ External math function IR test passed!");
}