use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType, StringPart, VariableDeclarationType};
use std::process::Command;
use std::fs;
use tempfile::TempDir;

/// Helper to compile Zen code to executable and capture its output
pub struct ExecutionHelper {
    temp_dir: TempDir,
}

impl ExecutionHelper {
    pub fn new() -> Self {
        ExecutionHelper {
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
        }
    }

    /// Compile Zen AST to executable and run it
    pub fn compile_ast_and_run(
        &self,
        program: &zen::ast::Program,
    ) -> Result<CapturedOutput, String> {
        let context = inkwell::context::Context::create();
        let compiler = zen::compiler::Compiler::new(&context);

        // Compile to LLVM IR
        let ir = compiler
            .compile_llvm(program)
            .map_err(|e| format!("Compilation failed: {:?}", e))?;

        // Write IR to file
        let ir_path = self.temp_dir.path().join("test.ll");
        fs::write(&ir_path, ir).map_err(|e| format!("Failed to write IR: {}", e))?;

        // Run LLVM IR directly with lli (LLVM interpreter)
        // Try different versions of lli
        let lli_commands = ["lli-18", "lli-17", "lli-20", "lli"];
        let mut run_output = None;
        let mut last_error = String::new();

        for lli_cmd in &lli_commands {
            match Command::new(lli_cmd).arg(&ir_path).output() {
                Ok(output) => {
                    run_output = Some(output);
                    break;
                }
                Err(e) => {
                    last_error = format!("Failed to run {}: {}", lli_cmd, e);
                    continue;
                }
            }
        }

        let run_output = run_output
            .ok_or_else(|| format!("No working lli found. Last error: {}", last_error))?;

        Ok(CapturedOutput {
            stdout: String::from_utf8_lossy(&run_output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&run_output.stderr).to_string(),
            exit_code: run_output.status.code().unwrap_or(-1),
        })
    }
}

#[derive(Debug)]
pub struct CapturedOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CapturedOutput {
    /// Assert that stdout contains a specific string
    pub fn assert_stdout_contains(&self, s: &str) {
        assert!(
            self.stdout.contains(s),
            "Expected stdout to contain '{}', but got: '{}'",
            s,
            self.stdout
        );
    }

    /// Assert that the exit code matches expected
    pub fn assert_exit_code(&self, expected: i32) {
        assert_eq!(
            self.exit_code, expected,
            "Expected exit code {}, got {}",
            expected, self.exit_code
        );
    }
}

#[test]
fn test_string_interpolation_integer_output() {
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

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Verify the interpolation works
    output.assert_stdout_contains("The answer is: 42");
    output.assert_exit_code(0);
    
    println!("✓ String interpolation with integer verified!");
}

#[test]
fn test_string_interpolation_string_var_output() {
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
                    Statement::VariableDeclaration {
                        name: "name".to_string(),
                        type_: Some(AstType::String),
                        initializer: Some(Expression::String("Zen".to_string())),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::StringInterpolation {
                                parts: vec![
                                    StringPart::Literal("Hello from ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("name".to_string())),
                                    StringPart::Literal(" language!\n".to_string()),
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

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Verify the interpolation works with string variable
    output.assert_stdout_contains("Hello from Zen language!");
    output.assert_exit_code(0);
    
    println!("✓ String interpolation with string variable verified!");
}

#[test]
fn test_string_interpolation_multiple_vars_output() {
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
                    Statement::VariableDeclaration {
                        name: "x".to_string(),
                        type_: Some(AstType::I32),
                        initializer: Some(Expression::Integer32(10)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::VariableDeclaration {
                        name: "y".to_string(),
                        type_: Some(AstType::I32),
                        initializer: Some(Expression::Integer32(20)),
                        is_mutable: false,
                        declaration_type: VariableDeclarationType::ExplicitImmutable,
                    },
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![
                            Expression::StringInterpolation {
                                parts: vec![
                                    StringPart::Literal("x = ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("x".to_string())),
                                    StringPart::Literal(", y = ".to_string()),
                                    StringPart::Interpolation(Expression::Identifier("y".to_string())),
                                    StringPart::Literal(", sum = ".to_string()),
                                    StringPart::Interpolation(Expression::BinaryOp {
                                        left: Box::new(Expression::Identifier("x".to_string())),
                                        op: ast::BinaryOperator::Add,
                                        right: Box::new(Expression::Identifier("y".to_string())),
                                    }),
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

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Verify multiple interpolations and expression evaluation
    output.assert_stdout_contains("x = 10, y = 20, sum = 30");
    output.assert_exit_code(0);
    
    println!("✓ String interpolation with multiple variables and expressions verified!");
}