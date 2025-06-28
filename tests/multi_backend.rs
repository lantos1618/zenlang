//! Multi-Backend Interface Tests
//! 
//! These tests verify that the Zen compiler architecture properly separates
//! frontend (parsing) from backend (code generation), allowing different
//! backends to be plugged in.

use zen::ast::{self, AstType, Expression, Statement};
use zen::compiler::Compiler;
use inkwell::context::Context;

#[test]
fn test_multi_backend_interface() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // Create a simple test program
    let program = ast::Program::from_functions(vec![
        ast::Function {
            name: "main".to_string(),
            args: vec![],
            return_type: AstType::I64,
            body: vec![Statement::Return(Expression::Integer64(42))],
            is_async: false,
        }
    ]);
    
    // Test that the LLVM backend works through the clean interface
    let result = compiler.compile_llvm(&program);
    assert!(result.is_ok(), "LLVM backend should handle basic compilation");
    
    let output = result.unwrap();
    assert!(output.contains("define i64 @main()"), "Should produce LLVM IR");
    assert!(output.contains("ret i64 42"), "Should contain return instruction");
}

#[test]
fn test_frontend_backend_separation() {
    let context = Context::create();
    let compiler = Compiler::new(&context);
    
    // Test that frontend (parsing) is completely separate from backend (codegen)
    let source = "main = () i64 { 42 }";
    
    // Parse using frontend only
    let lexer = zen::lexer::Lexer::new(source);
    let mut parser = zen::parser::Parser::new(lexer);
    let program = parser.parse_program().unwrap();
    
    // Verify frontend worked correctly
    assert_eq!(program.declarations.len(), 1);
    if let ast::Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.return_type, AstType::I64);
    } else {
        panic!("Expected Function declaration");
    }
    
    // Now test backend separately
    let result = compiler.compile_llvm(&program);
    assert!(result.is_ok(), "Backend should work with parsed program");
    
    let output = result.unwrap();
    assert!(output.contains("define i64 @main()"), "Backend should produce LLVM IR");
} 