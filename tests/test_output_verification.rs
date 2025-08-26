use zen::ast::{self, Declaration, ExternalFunction, Function, Statement, Expression, AstType};
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
fn test_printf_output_verification() {
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
                        args: vec![Expression::String("Hello from verified test!\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    // Compile and run the program, capturing output
    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Verify the output
    output.assert_stdout_contains("Hello from verified test!");
    output.assert_exit_code(0);
    
    println!("✓ Printf output verified successfully!");
}

#[test]
fn test_puts_output_verification() {
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
                        args: vec![Expression::String("Testing puts output".to_string())],
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
    output.assert_stdout_contains("Testing puts output");
    output.assert_exit_code(0);
    
    println!("✓ Puts output verified successfully!");
}

#[test]
fn test_multiple_printf_calls() {
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
                        args: vec![Expression::String("First line\n".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Second line\n".to_string())],
                    }),
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Third line\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(42)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Verify all outputs appear
    output.assert_stdout_contains("First line");
    output.assert_stdout_contains("Second line");
    output.assert_stdout_contains("Third line");
    output.assert_exit_code(42);
    
    // Verify order
    let stdout = &output.stdout;
    let first_pos = stdout.find("First line").expect("First line not found");
    let second_pos = stdout.find("Second line").expect("Second line not found");
    let third_pos = stdout.find("Third line").expect("Third line not found");
    
    assert!(first_pos < second_pos, "First should come before second");
    assert!(second_pos < third_pos, "Second should come before third");
    
    println!("✓ Multiple printf calls verified successfully!");
}

#[test]
fn test_printf_with_format_specifiers() {
    // This test will be expanded once we have proper format string support
    // For now, just test that we can pass format strings even if not fully processed
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
                    // For now just test plain strings work
                    // TODO: Add integer passing once varargs fully work
                    Statement::Expression(Expression::FunctionCall {
                        name: "printf".to_string(),
                        args: vec![Expression::String("Number: %d\n".to_string())],
                    }),
                    Statement::Return(Expression::Integer64(0)),
                ],
                is_async: false,
            }),
        ],
    };

    let output = helper.compile_ast_and_run(&program)
        .expect("Failed to compile and run program");

    // Just verify it doesn't crash for now
    // Format specifiers without args will print as-is or garbage
    output.assert_exit_code(0);
    
    println!("✓ Printf with format specifiers doesn't crash!");
}