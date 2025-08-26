use inkwell::context::Context;
use std::process::Command;
use std::fs;
use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType, VariableDeclarationType};
use zen::compiler::Compiler;

#[test]
fn test_printf_output_captured() {
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
                        args: vec![Expression::String("Test output from Zen\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile to LLVM IR
    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Write IR to a temporary file
    let ir_file = "/tmp/test_printf.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile IR to executable using LLC and GCC
    // First convert LLVM IR to assembly
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_printf.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Then assemble and link with gcc
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_printf", "/tmp/test_printf.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Run the executable and capture output
    let output = Command::new("/tmp/test_printf")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "Test output from Zen\n", "Printf output doesn't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_printf.s");
    let _ = fs::remove_file("/tmp/test_printf");
}

#[test]
fn test_puts_output_captured() {
    let context = Context::create();
    let compiler = Compiler::new(&context);

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
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "puts".to_string(),
                        args: vec![Expression::String("Hello from puts".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile to LLVM IR
    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Write IR to a temporary file
    let ir_file = "/tmp/test_puts.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile IR to executable using LLC and GCC
    // First convert LLVM IR to assembly
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_puts.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Then assemble and link with gcc
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_puts", "/tmp/test_puts.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Run the executable and capture output
    let output = Command::new("/tmp/test_puts")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output (puts adds a newline automatically)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "Hello from puts\n", "Puts output doesn't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_puts.s");
    let _ = fs::remove_file("/tmp/test_puts");
}

#[test]
fn test_printf_with_format_args() {
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
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: Some(AstType::I32),
                        initializer: Some(Expression::Integer32(42)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::String("Number: %d\n".to_string()),
                            Expression::Identifier("x".to_string()),
                        ],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile to LLVM IR
    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Write IR to a temporary file
    let ir_file = "/tmp/test_printf_fmt.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile IR to executable using LLC and GCC
    // First convert LLVM IR to assembly
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_printf_fmt.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Then assemble and link with gcc
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_printf_fmt", "/tmp/test_printf_fmt.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Run the executable and capture output
    let output = Command::new("/tmp/test_printf_fmt")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "Number: 42\n", "Printf format output doesn't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_printf_fmt.s");
    let _ = fs::remove_file("/tmp/test_printf_fmt");
}

#[test]
fn test_multiple_external_calls() {
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
                return_type: AstType::I64,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("First: printf\n".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "puts".to_string(),
                        args: vec![Expression::String("Second: puts".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Third: printf again\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile to LLVM IR
    let ir = compiler.compile_llvm(&program).unwrap();
    
    // Write IR to a temporary file
    let ir_file = "/tmp/test_multi_extern.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile IR to executable using LLC and GCC
    // First convert LLVM IR to assembly
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_multi_extern.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Then assemble and link with gcc
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_multi_extern", "/tmp/test_multi_extern.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    // Run the executable and capture output
    let output = Command::new("/tmp/test_multi_extern")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let expected = "First: printf\nSecond: puts\nThird: printf again\n";
    assert_eq!(stdout, expected, "Multiple external call outputs don't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_multi_extern.s");
    let _ = fs::remove_file("/tmp/test_multi_extern");
}