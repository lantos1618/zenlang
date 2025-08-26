use inkwell::context::Context;
use std::process::Command;
use std::fs;
use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType, VariableDeclarationType, StringPart};
use zen::compiler::Compiler;

#[test]
fn test_simple_string_interpolation() {
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
                            Expression::StringInterpolation {
                                parts: vec![
                                    StringPart::Literal("The answer is: ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("x".to_string())),
                                    StringPart::Literal("\n".to_string()),
                                ],
                            }
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
    let ir_file = "/tmp/test_interpolation.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile and run
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_interpolation.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_interpolation", "/tmp/test_interpolation.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let output = Command::new("/tmp/test_interpolation")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "The answer is: 42\n", "Interpolation output doesn't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_interpolation.s");
    let _ = fs::remove_file("/tmp/test_interpolation");
}

#[test]
fn test_multiple_interpolations() {
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
                        name: "name".to_string(),
                        type_: Some(AstType::String),
                        initializer: Some(Expression::String("Zen".to_string())),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::VariableDeclaration {
                        name: "version".to_string(),
                        type_: Some(AstType::I32),
                        initializer: Some(Expression::Integer32(1)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::StringInterpolation {
                                parts: vec![
                                    StringPart::Literal("Language: ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("name".to_string())),
                                    StringPart::Literal(", Version: ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("version".to_string())),
                                    StringPart::Literal("\n".to_string()),
                                ],
                            }
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
    let ir_file = "/tmp/test_multi_interp.ll";
    fs::write(ir_file, &ir).expect("Failed to write IR file");
    
    // Compile and run
    let output = Command::new("llc-17")
        .args(["-o", "/tmp/test_multi_interp.s", ir_file])
        .output()
        .expect("Failed to compile IR with llc");
    assert!(output.status.success(), "LLC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let output = Command::new("gcc")
        .args(["-no-pie", "-o", "/tmp/test_multi_interp", "/tmp/test_multi_interp.s"])
        .output()
        .expect("Failed to compile assembly with gcc");
    assert!(output.status.success(), "GCC compilation failed: {}", String::from_utf8_lossy(&output.stderr));
    
    let output = Command::new("/tmp/test_multi_interp")
        .output()
        .expect("Failed to run executable");
    
    // Verify the output
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout, "Language: Zen, Version: 1\n", "Multiple interpolation output doesn't match expected");
    assert!(output.status.success(), "Program didn't exit successfully");
    
    // Clean up
    let _ = fs::remove_file(ir_file);
    let _ = fs::remove_file("/tmp/test_multi_interp.s");
    let _ = fs::remove_file("/tmp/test_multi_interp");
}

#[test]
fn test_parse_string_interpolation() {
    use zen::lexer::Lexer;
    use zen::parser::Parser;
    
    let input = r#"
    main = () i32 {
        x := 42
        message := "The answer is $(x)"
        return 0
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.name, "main");
        assert_eq!(func.body.len(), 3);
        
        // Check the second statement (message assignment)
        if let Statement::VariableDeclaration { name, initializer, .. } = &func.body[1] {
            assert_eq!(name, "message");
            
            // Check that it parsed as a string interpolation
            if let Some(Expression::StringInterpolation { parts }) = initializer {
                assert_eq!(parts.len(), 2);
                
                // First part should be literal "The answer is "
                if let StringPart::Literal(s) = &parts[0] {
                    assert_eq!(s, "The answer is ");
                } else {
                    panic!("Expected literal string part");
                }
                
                // Second part should be interpolation of x
                if let StringPart::Interpolation(Expression::Identifier(id)) = &parts[1] {
                    assert_eq!(id, "x");
                } else {
                    panic!("Expected interpolated identifier");
                }
            } else {
                panic!("Expected StringInterpolation expression");
            }
        } else {
            panic!("Expected variable declaration for 'message'");
        }
    } else {
        panic!("Expected Function declaration");
    }
}

#[test]
fn test_nested_expression_interpolation() {
    use zen::lexer::Lexer;
    use zen::parser::Parser;
    
    let input = r#"
    main = () i32 {
        x := 10
        y := 5
        result := "$(x) + $(y) = $(x + y)"
        return 0
    }
    "#;
    
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    
    let program = parser.parse_program().unwrap();
    assert_eq!(program.declarations.len(), 1);
    
    if let Declaration::Function(func) = &program.declarations[0] {
        assert_eq!(func.body.len(), 4);
        
        // Check the result assignment
        if let Statement::VariableDeclaration { name, initializer, .. } = &func.body[2] {
            assert_eq!(name, "result");
            
            if let Some(Expression::StringInterpolation { parts }) = initializer {
                assert_eq!(parts.len(), 5);
                
                // Check the third interpolation (x + y)
                if let StringPart::Interpolation(expr) = &parts[4] {
                    if let Expression::BinaryOp { left, op, right } = expr {
                        assert!(matches!(op, ast::BinaryOperator::Add));
                        assert!(matches!(**left, Expression::Identifier(ref n) if n == "x"));
                        assert!(matches!(**right, Expression::Identifier(ref n) if n == "y"));
                    } else {
                        panic!("Expected binary operation in interpolation");
                    }
                } else {
                    panic!("Expected interpolated expression");
                }
            } else {
                panic!("Expected StringInterpolation expression");
            }
        } else {
            panic!("Expected variable declaration for 'result'");
        }
    } else {
        panic!("Expected Function declaration");
    }
}