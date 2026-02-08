use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use inkwell::context::Context;
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::OptimizationLevel;
use zen::compiler::Compiler;
use zen::lexer::Lexer;
use zen::parser::Parser;

/// Global counter for unique test IDs (thread-safe)
static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Result of running compiled code
#[derive(Debug)]
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Compile Zen source code to a temporary executable and run it.
/// Returns the exit code and captured stdout/stderr.
pub fn compile_and_run(source: &str) -> Result<RunResult, String> {
    // Initialize LLVM
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| format!("LLVM init failed: {}", e))?;

    let context = Context::create();
    let compiler = Compiler::new(&context);

    // Parse
    let lexer = Lexer::new(source);
    let mut parser = Parser::new(lexer);
    let program = parser
        .parse_program()
        .map_err(|e| format!("Parse error: {}", e))?;

    // Compile to LLVM module
    let module = compiler
        .get_module(&program)
        .map_err(|e| format!("Compilation error: {}", e))?;

    // Create temp file paths with unique IDs to avoid conflicts in parallel tests
    let test_id = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let thread_id = std::thread::current().id();
    let obj_path = format!("/tmp/zen_test_{:?}_{}.o", thread_id, test_id);
    let exe_path = format!("/tmp/zen_test_{:?}_{}", thread_id, test_id);

    // Get target machine
    let target_triple = TargetMachine::get_default_triple();
    let target =
        Target::from_triple(&target_triple).map_err(|e| format!("Failed to get target: {}", e))?;

    let target_machine = target
        .create_target_machine(
            &target_triple,
            "generic",
            "",
            OptimizationLevel::None,
            RelocMode::Default,
            CodeModel::Default,
        )
        .ok_or_else(|| "Failed to create target machine".to_string())?;

    // Write object file
    target_machine
        .write_to_file(&module, FileType::Object, Path::new(&obj_path))
        .map_err(|e| format!("Failed to write object file: {}", e))?;

    // Link
    let link_status = Command::new("cc")
        .arg(&obj_path)
        .arg("-o")
        .arg(&exe_path)
        .arg("-no-pie")
        .arg("-lm")
        .status()
        .map_err(|e| format!("Failed to link: {}", e))?;

    if !link_status.success() {
        fs::remove_file(&obj_path).ok();
        return Err("Linking failed".to_string());
    }

    // Clean up object file
    fs::remove_file(&obj_path).ok();

    // Verify executable exists
    if !Path::new(&exe_path).exists() {
        return Err(format!("Executable was not created at {}", exe_path));
    }

    // Run the executable
    let output = Command::new(&exe_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("Failed to run executable: {}", e))?;

    // Clean up executable
    fs::remove_file(&exe_path).ok();

    // Handle signals (e.g., segfault = SIGSEGV = signal 11)
    let exit_code = output.status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            if let Some(signal) = output.status.signal() {
                return -(signal as i32); // Return negative signal number
            }
        }
        -1
    });

    Ok(RunResult {
        exit_code,
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// Helper to compile and run, asserting success
pub fn run_expecting_success(source: &str) -> RunResult {
    match compile_and_run(source) {
        Ok(result) => {
            if result.exit_code < 0 {
                let signal = -result.exit_code;
                let signal_name = match signal {
                    11 => "SIGSEGV (segmentation fault)",
                    6 => "SIGABRT (abort)",
                    8 => "SIGFPE (floating point exception)",
                    _ => "unknown signal",
                };
                panic!(
                    "Program crashed with signal {} ({})!\nstdout: {}\nstderr: {}",
                    signal, signal_name, result.stdout, result.stderr
                );
            }
            result
        }
        Err(e) => panic!("Compilation/run failed: {}", e),
    }
}
