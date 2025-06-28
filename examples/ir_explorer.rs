//! IR Explorer - Development tool for examining LLVM IR output
//! 
//! This is not a test, but rather a utility for debugging and exploring
//! what LLVM IR the compiler generates for various Zen constructs.
//! 
//! Run with: cargo run --example ir_explorer

use inkwell::context::Context;
use zen::ast::{self, Declaration, ExternalFunction, Expression, Function, Statement, AstType, VariableDeclarationType};
use zen::compiler::Compiler;

fn main() {
    let context = Context::create();
    
    // Test 1: Simple arithmetic and function calls
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program::from_functions(vec![ast::Function {
            name: "test_arithmetic".to_string(),
            args: vec![("x".to_string(), AstType::I64), ("y".to_string(), AstType::I64)],
            return_type: AstType::I64,
            body: vec![
                Statement::VariableDeclaration {
                    name: "result".to_string(),
                    type_: Some(AstType::I64),
                    initializer: Some(Expression::BinaryOp {
                        left: Box::new(Expression::Identifier("x".to_string())),
                        op: ast::BinaryOperator::Add,
                        right: Box::new(Expression::Identifier("y".to_string())),
                    }),
                    is_mutable: false,
                    declaration_type: VariableDeclarationType::ExplicitImmutable,
                },
                Statement::Return(Expression::Identifier("result".to_string())),
            ],
            is_async: false,
        }]);

        let ir = compiler.compile_llvm(&program).unwrap();
        println!("Simple arithmetic IR:");
        println!("{}", ir);
        println!("\n---\n");
    }

    // Test 2: String with null terminator
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program::from_functions(vec![ast::Function {
            name: "test_string".to_string(),
            args: vec![],
            return_type: AstType::String,
            body: vec![Statement::Return(Expression::String("Hello, World!".to_string()))],
            is_async: false,
        }]);

        let ir = compiler.compile_llvm(&program).unwrap();
        println!("String operations IR:");
        println!("{}", ir);
        
        // Check if string contains the expected pattern
        if ir.contains("[13 x i8]") {
            println!("✓ Found [13 x i8]");
        } else {
            println!("✗ Did not find [13 x i8]");
            // Look for what pattern we actually have
            if ir.contains("[14 x i8]") {
                println!("  Found [14 x i8] instead (with null terminator)");
            }
        }
    }

    // Test 3: C FFI with printf
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program {
            declarations: vec![
                Declaration::ExternalFunction(ExternalFunction {
                    name: "printf".to_string(),
                    args: vec![AstType::String], // First arg is format string
                    return_type: AstType::I64,
                    is_varargs: true, // printf is variadic
                }),
                Declaration::Function(Function {
                    name: "main".to_string(),
                    args: vec![],
                    return_type: AstType::I64,
                    body: vec![
                        Statement::Expression(Expression::FunctionCall {
                            name: "printf".to_string(),
                            args: vec![Expression::String("Hello from Zen!\n".to_string())],
                        }),
                        Statement::Return(Expression::Integer64(0)),
                    ],
                    is_async: false,
                }),
            ],
        };

        let ir = compiler.compile_llvm(&program).unwrap();
        println!("C FFI (printf) IR:");
        println!("{}", ir);
        
        // Check what we actually get
        if ir.contains("declare") {
            println!("✓ Found declare statement");
        }
        if ir.contains("@printf") {
            println!("✓ Found @printf");
        }
        if ir.contains("...") {
            println!("✓ Found varargs (...)");
        }
    }

    // Test 4: Math library FFI
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program {
            declarations: vec![
                Declaration::ExternalFunction(ExternalFunction {
                    name: "sqrt".to_string(),
                    args: vec![AstType::F64],
                    return_type: AstType::F64,
                    is_varargs: false,
                }),
                Declaration::Function(Function {
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
        println!("Math library FFI (sqrt) IR:");
        println!("{}", ir);
        
        // Check what we actually get
        if ir.contains("declare double @sqrt(double)") {
            println!("✓ Found sqrt declaration");
        }
        if ir.contains("call double @sqrt") {
            println!("✓ Found sqrt function call");
        }
    }
} 