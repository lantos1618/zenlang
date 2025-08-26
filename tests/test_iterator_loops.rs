use zen::ast::{Declaration, ExternalFunction, Function, Statement, Expression, AstType, LoopKind, VariableDeclarationType};
use zen::compiler::Compiler;
use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to compile and run Zen code
struct ExecutionHelper {
    temp_dir: TempDir,
}

impl ExecutionHelper {
    fn new() -> Self {
        ExecutionHelper {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    fn compile_and_run(&self, program: &zen::ast::Program) -> Result<i64, String> {
        let context = inkwell::context::Context::create();
        let compiler = Compiler::new(&context);

        // Compile to LLVM IR
        let ir = compiler
            .compile_llvm(program)
            .map_err(|e| format!("Compilation failed: {:?}", e))?;

        // Write IR to file
        let ir_path = self.temp_dir.path().join("test.ll");
        fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

        // Try to run with lli
        let lli_commands = ["lli-18", "lli-17", "lli-20", "lli"];
        
        for lli_cmd in &lli_commands {
            if let Ok(output) = Command::new(lli_cmd).arg(&ir_path).output() {
                let exit_code = output.status.code().unwrap_or(-1) as i64;
                return Ok(exit_code);
            }
        }
        
        Err("No working lli found".to_string())
    }
}

#[test]
fn test_loop_iterator_array_literal() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // sum := 0
    // loop x in [1, 2, 3, 4, 5] {
    //     sum ::= sum + x  
    // }
    // return sum
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::VariableDeclaration {
                name: "sum".to_string(),
                type_: None,
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::InferredMutable,
            },
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![
                        Expression::Integer64(1),
                        Expression::Integer64(2),
                        Expression::Integer64(3),
                        Expression::Integer64(4),
                        Expression::Integer64(5),
                    ]),
                },
                label: None,
                body: vec![
                    Statement::VariableAssignment {
                        name: "sum".to_string(),
                        value: Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("sum".to_string())),
                            op: zen::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Identifier("x".to_string())),
                        },
                    },
                ],
            },
            Statement::Return(Expression::Identifier("sum".to_string())),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, Declaration::Function(main_function)],
    };
    
    let result = helper.compile_and_run(&program);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    assert_eq!(result.unwrap(), 15); // 1+2+3+4+5 = 15
}

#[test]
fn test_loop_iterator_empty_array() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // count := 0
    // loop x in [] {
    //     count ::= count + 1
    // }
    // return count
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::VariableDeclaration {
                name: "count".to_string(),
                type_: None,
                initializer: Some(Expression::Integer64(0)),
                is_mutable: true,
                declaration_type: VariableDeclarationType::InferredMutable,
            },
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![]),
                },
                label: None,
                body: vec![
                    Statement::VariableAssignment {
                        name: "count".to_string(),
                        value: Expression::BinaryOp {
                            left: Box::new(Expression::Identifier("count".to_string())),
                            op: zen::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Integer64(1)),
                        },
                    },
                ],
            },
            Statement::Return(Expression::Identifier("count".to_string())),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, Declaration::Function(main_function)],
    };
    
    let result = helper.compile_and_run(&program);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
    assert_eq!(result.unwrap(), 0); // Should not enter loop body
}

#[test]
fn test_loop_iterator_with_printf() {
    let helper = ExecutionHelper::new();
    
    // Malloc declaration for arrays
    let malloc_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "malloc".to_string(),
        args: vec![AstType::I64],
        return_type: AstType::Pointer(Box::new(AstType::Void)),
        is_varargs: false,
    });
    
    // Iterate and print each element
    // loop x in [10, 20, 30] {
    //     printf("value: %d\n", x)
    // }
    // return 0
    
    // Printf declaration
    let printf_decl = Declaration::ExternalFunction(ExternalFunction {
        name: "printf".to_string(),
        args: vec![AstType::String],
        return_type: AstType::I32,
        is_varargs: true,
    });
    
    let main_function = Function {
        name: "main".to_string(),
        type_params: vec![],
        args: vec![],
        return_type: AstType::I64,
        body: vec![
            Statement::Loop {
                kind: LoopKind::Iterator {
                    variable: "x".to_string(),
                    iterable: Expression::ArrayLiteral(vec![
                        Expression::Integer64(10),
                        Expression::Integer64(20),
                        Expression::Integer64(30),
                    ]),
                },
                label: None,
                body: vec![
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::String("value: %lld\n".to_string()),
                            Expression::Identifier("x".to_string()),
                        ],
                    }),
                ],
            },
            Statement::Return(Expression::Integer64(0)),
        ],
        is_async: false,
    };
    
    let program = zen::ast::Program {
        declarations: vec![malloc_decl, printf_decl, Declaration::Function(main_function)],
    };
    
    // Just verify it compiles - we're not checking output in this test
    let result = helper.compile_and_run(&program);
    assert!(result.is_ok(), "Failed to compile: {:?}", result.err());
}