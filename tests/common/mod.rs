use zen::ast;
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

    /// Compile Zen AST to executable and run it, capturing output
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

    /// Get the path to the temp directory for custom file operations
    pub fn temp_dir_path(&self) -> &std::path::Path {
        self.temp_dir.path()
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

    /// Assert stdout equals expected string exactly
    pub fn assert_stdout_eq(&self, expected: &str) {
        assert_eq!(
            self.stdout.trim(),
            expected.trim(),
            "Stdout mismatch.\nExpected: '{}'\nActual: '{}'",
            expected,
            self.stdout
        );
    }

    /// Assert that stderr contains a specific string
    pub fn assert_stderr_contains(&self, s: &str) {
        assert!(
            self.stderr.contains(s),
            "Expected stderr to contain '{}', but got: '{}'",
            s,
            self.stderr
        );
    }

    /// Assert stderr is empty
    pub fn assert_stderr_empty(&self) {
        assert!(
            self.stderr.is_empty(),
            "Expected empty stderr, but got: '{}'",
            self.stderr
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

    /// Assert successful execution (exit code 0)
    pub fn assert_success(&self) {
        self.assert_exit_code(0);
    }

    /// Get lines from stdout
    pub fn stdout_lines(&self) -> Vec<&str> {
        self.stdout.lines().collect()
    }

    /// Check if stdout matches a regex pattern
    pub fn assert_stdout_matches(&self, pattern: &str) {
        let re = regex::Regex::new(pattern)
            .expect(&format!("Invalid regex pattern: {}", pattern));
        assert!(
            re.is_match(&self.stdout),
            "Stdout does not match pattern '{}'. Got: '{}'",
            pattern,
            self.stdout
        );
    }
}

/// Macro to quickly create test programs with less boilerplate
#[macro_export]
macro_rules! test_program {
    (extern { $($extern:tt)* } main { $($body:tt)* }) => {{
        use zen::ast::{Program, Declaration};
        Program {
            declarations: vec![
                $($extern)*,
                Declaration::Function(Function {
                    type_params: vec![],
                    name: "main".to_string(),
                    args: vec![],
                    return_type: AstType::I32,
                    body: vec![$($body)*],
                    is_async: false,
                })
            ]
        }
    }};
}