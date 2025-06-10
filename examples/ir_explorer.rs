//! IR Explorer - Development tool for examining LLVM IR output
//! 
//! This is not a test, but rather a utility for debugging and exploring
//! what LLVM IR the compiler generates for various Lynlang constructs.
//! 
//! Run with: cargo run --example ir_explorer

use inkwell::context::Context;
use lynlang::ast::{self, Declaration, ExternalFunction, Expression, Function, Statement, Type};
use lynlang::compiler::Compiler;

fn main() {
    let context = Context::create();
    
    // Test 1: Nested conditionals
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program::from_functions(vec![ast::Function {
            name: "test_nested_ifs".to_string(),
            args: vec![("x".to_string(), Type::Int64)],
            return_type: Type::Int64,
            body: vec![
                Statement::VariableDeclaration {
                    name: "result".to_string(),
                    type_: Type::Int64,
                    initializer: None,
                },
                Statement::Expression(Expression::Conditional {
                    scrutinee: Box::new(Expression::Identifier("x".to_string())),
                    arms: vec![
                        (
                            Expression::Integer64(1),
                            Expression::Integer64(10),
                        ),
                        (
                            Expression::Integer64(2),
                            Expression::Integer64(20),
                        ),
                        (
                            Expression::Integer64(3),
                            Expression::Integer64(30),
                        ),
                    ],
                }),
                Statement::Return(Expression::Identifier("result".to_string())),
            ],
        }]);

        compiler.compile_program(&program).unwrap();
        println!("Nested conditionals IR:");
        println!("{}", compiler.module.print_to_string().to_string());
        println!("\n---\n");
    }

    // Test 2: String with null terminator
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program::from_functions(vec![ast::Function {
            name: "test_string".to_string(),
            args: vec![],
            return_type: Type::String,
            body: vec![Statement::Return(Expression::String("Hello, World!".to_string()))],
        }]);

        compiler.compile_program(&program).unwrap();
        println!("String operations IR:");
        let ir = compiler.module.print_to_string().to_string();
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

    // Test C FFI with printf
    {
        let mut compiler = Compiler::new(&context);
        let program = ast::Program {
            declarations: vec![
                Declaration::ExternalFunction(ExternalFunction {
                    name: "printf".to_string(),
                    args: vec![Type::String], // First arg is format string
                    return_type: Type::Int64,
                    is_varargs: true, // printf is variadic
                }),
                Declaration::Function(Function {
                    name: "main".to_string(),
                    args: vec![],
                    return_type: Type::Int64,
                    body: vec![
                        Statement::Expression(Expression::FunctionCall {
                            name: "printf".to_string(),
                            args: vec![Expression::String("Hello from Lynlang!\n".to_string())],
                        }),
                        Statement::Return(Expression::Integer64(0)),
                    ],
                }),
            ],
        };

        compiler.compile_program(&program).unwrap();
        println!("C FFI (printf) IR:");
        let ir = compiler.module.print_to_string().to_string();
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
} 