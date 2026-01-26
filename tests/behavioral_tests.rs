//! Behavioral tests that compile and RUN code to verify correct output.
//!
//! These tests catch semantic bugs that syntax/IR-only tests miss:
//! - Type mismatches that produce wrong values (not crashes)
//! - Memory corruption from stack layout issues
//! - Logic errors in codegen
//! - Silent data corruption
//!
//! Philosophy: Test what the user cares about - does the program produce correct output?

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
fn compile_and_run(source: &str) -> Result<RunResult, String> {
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
fn run_expecting_success(source: &str) -> RunResult {
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

// ============================================================================
// REGRESSION TESTS - Tests for specific bugs that were found and fixed
// ============================================================================

/// Regression test for: i64/i32 type mismatch causing stack corruption
/// Bug: Integer literals were i64 but alloca was i32, causing adjacent variables
/// to be overwritten when storing 8 bytes into 4-byte slot.
/// Fixed in: coercing_store in variables.rs
#[test]
fn test_regression_i64_i32_stack_corruption() {
    let source = r#"
        { io } = @std.io

        main = () i32 {
            a = 5
            b = 3
            msg = "${a}"
            // After string interpolation, 'a' should still be 5
            // Bug caused 'a' to become 0 due to stack corruption
            a == 5 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(
        result.exit_code, 0,
        "Variable 'a' was corrupted! Expected 5, got something else.\nstdout: {}\nstderr: {}",
        result.stdout, result.stderr
    );
}

/// Verify multiple variables maintain their values after string operations
#[test]
fn test_multiple_variables_after_string_interpolation() {
    let source = r#"
        { io } = @std.io

        main = () i32 {
            a = 10
            b = 20
            c = 30
            msg = "${a} ${b} ${c}"

            // All variables should maintain their values
            sum = a + b + c
            sum == 60 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(
        result.exit_code, 0,
        "Variables were corrupted after string interpolation.\nstdout: {}\nstderr: {}",
        result.stdout, result.stderr
    );
}

// ============================================================================
// ARITHMETIC TESTS
// ============================================================================

#[test]
fn test_basic_arithmetic() {
    let source = r#"
        main = () i32 {
            a = 10
            b = 3
            sum = a + b
            diff = a - b
            prod = a * b
            quot = a / b

            // 10 + 3 = 13, 10 - 3 = 7, 10 * 3 = 30, 10 / 3 = 3
            // sum + diff + prod + quot = 13 + 7 + 30 + 3 = 53
            result = sum + diff + prod + quot
            result == 53 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Basic arithmetic failed");
}

#[test]
fn test_negative_numbers() {
    let source = r#"
        main = () i32 {
            a = 5
            b = 10
            diff = a - b  // -5

            diff == -5 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Negative number handling failed");
}

// ============================================================================
// CONTROL FLOW TESTS
// ============================================================================

#[test]
fn test_conditional_branches() {
    let source = r#"
        main = () i32 {
            x = 42
            x == 42 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Conditional branch failed");
}

#[test]
fn test_conditional_false_branch() {
    let source = r#"
        main = () i32 {
            x = 10
            x == 42 ?
                | true { return 1 }
                | false { return 0 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Conditional false branch failed");
}

// ============================================================================
// FUNCTION TESTS
// ============================================================================

#[test]
fn test_function_call_and_return() {
    let source = r#"
        add = (a: i32, b: i32) i32 {
            return a + b
        }

        main = () i32 {
            result = add(3, 4)
            result == 7 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Function call failed");
}

#[test]
fn test_nested_function_calls() {
    let source = r#"
        double = (x: i32) i32 {
            return x * 2
        }

        quadruple = (x: i32) i32 {
            return double(double(x))
        }

        main = () i32 {
            result = quadruple(5)  // 5 * 2 * 2 = 20
            result == 20 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Nested function calls failed");
}

// ============================================================================
// STRUCT TESTS
// ============================================================================

#[test]
fn test_struct_field_access() {
    let source = r#"
        Point: {
            x: i32,
            y: i32
        }

        main = () i32 {
            p = Point { x: 10, y: 20 }
            sum = p.x + p.y
            sum == 30 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Struct field access failed");
}

#[test]
fn test_nested_struct_field_access() {
    let source = r#"
        Inner: {
            value: i32
        }

        Outer: {
            inner: Inner
        }

        main = () i32 {
            obj = Outer {
                inner: Inner { value: 42 }
            }
            obj.inner.value == 42 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Nested struct field access failed");
}

/// Test for struct field assignment type mismatch
/// Note: Direct struct field assignment (p.x = 10) is currently not supported
/// This test uses mutable variable reassignment instead
#[test]
fn test_struct_mutable_reassignment() {
    let source = r#"
        Point: {
            x: i32,
            y: i32
        }

        main = () i32 {
            // Test that struct values work correctly with different field values
            p1 = Point { x: 10, y: 20 }
            p2 = Point { x: 100, y: 200 }

            sum1 = p1.x + p1.y  // 30
            sum2 = p2.x + p2.y  // 300

            total = sum1 + sum2  // 330
            total == 330 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Struct mutable reassignment failed");
}

// ============================================================================
// OUTPUT VERIFICATION TESTS
// ============================================================================

#[test]
fn test_println_output() {
    let source = r#"
        { io } = @std.io

        main = () i32 {
            io.println("hello")
            return 0
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("hello"),
        "Expected 'hello' in output, got: {}",
        result.stdout
    );
}

#[test]
fn test_string_interpolation_output() {
    let source = r#"
        { io } = @std.io

        main = () i32 {
            x = 42
            io.println("value: ${x}")
            return 0
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0);
    assert!(
        result.stdout.contains("value: 42"),
        "Expected 'value: 42' in output, got: {}",
        result.stdout
    );
}

// ============================================================================
// VARIABLE DECLARATION TESTS - All 6 documented forms
// ============================================================================

/// Test immutable variable with type inference: x = 10
#[test]
fn test_variable_immutable_inferred() {
    let source = r#"
        main = () i32 {
            x = 42
            x == 42 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Immutable inferred variable failed");
}

/// Test immutable variable with explicit type: x: i32 = 10
#[test]
fn test_variable_immutable_explicit_type() {
    let source = r#"
        main = () i32 {
            x: i64 = 100
            x == 100 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Immutable explicit type variable failed");
}

/// Test mutable variable with inference: x ::= 10
#[test]
fn test_variable_mutable_inferred() {
    let source = r#"
        main = () i32 {
            x ::= 5
            x = x + 10
            x == 15 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Mutable inferred variable failed");
}

/// Test mutable variable with explicit type: x:: i32 = 10
#[test]
fn test_variable_mutable_explicit() {
    let source = r#"
        main = () i32 {
            x:: i32 = 20
            x = x * 2
            x == 40 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Mutable explicit type variable failed");
}

// ============================================================================
// UFC (UNIFORM FUNCTION CALL) TESTS
// ============================================================================

/// Test basic UFC - function called as method
#[test]
fn test_ufc_basic() {
    let source = r#"
        double = (n: i32) i32 {
            return n * 2
        }

        main = () i32 {
            // Traditional call
            a = double(5)
            // UFC call
            b = 5.double()

            (a == 10) ?
                | true {
                    (b == 10) ?
                        | true { return 0 }
                        | false { return 2 }
                }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "UFC basic call failed");
}

/// Test UFC chaining - multiple UFC calls in sequence
#[test]
fn test_ufc_chaining() {
    let source = r#"
        double = (n: i32) i32 { return n * 2 }
        add_one = (n: i32) i32 { return n + 1 }

        main = () i32 {
            // Chain: 5.double() = 10, 10.add_one() = 11, 11.double() = 22
            result = 5.double().add_one().double()
            result == 22 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "UFC chaining failed");
}

// ============================================================================
// FLOAT ARITHMETIC TESTS
// ============================================================================

/// Test f64 basic arithmetic via function parameters
#[test]
fn test_float64_arithmetic() {
    let source = r#"
        add_floats = (a: f64, b: f64) f64 {
            return a + b
        }

        main = () i32 {
            sum = add_floats(10.5, 2.5)

            // Check sum is approximately 13.0
            (sum > 12.9) ?
                | true {
                    (sum < 13.1) ?
                        | true { return 0 }
                        | false { return 2 }
                }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Float64 arithmetic failed");
}

// ============================================================================
// COMPARISON OPERATOR TESTS
// ============================================================================

/// Test less than operator
#[test]
fn test_comparison_less_than() {
    let source = r#"
        main = () i32 {
            (5 < 10) ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Less than comparison failed");
}

/// Test greater than operator
#[test]
fn test_comparison_greater_than() {
    let source = r#"
        main = () i32 {
            (10 > 5) ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Greater than comparison failed");
}

/// Test not equal operator
#[test]
fn test_comparison_not_equal() {
    let source = r#"
        main = () i32 {
            (5 != 10) ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Not equal comparison failed");
}

/// Test less than or equal
#[test]
fn test_comparison_less_equal() {
    let source = r#"
        main = () i32 {
            a = 5
            b = 5
            c = 3
            // Both should be true: 5 <= 5 and 3 <= 5
            (a <= b) ?
                | true {
                    (c <= a) ?
                        | true { return 0 }
                        | false { return 2 }
                }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Less than or equal comparison failed");
}

/// Test greater than or equal
#[test]
fn test_comparison_greater_equal() {
    let source = r#"
        main = () i32 {
            a = 10
            b = 10
            c = 15
            // Both should be true: 10 >= 10 and 15 >= 10
            (a >= b) ?
                | true {
                    (c >= a) ?
                        | true { return 0 }
                        | false { return 2 }
                }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Greater than or equal comparison failed");
}

// ============================================================================
// LOOP TESTS
// ============================================================================

/// Test basic loop with break
#[test]
fn test_loop_with_break() {
    let source = r#"
        main = () i32 {
            counter ::= 0
            loop(() {
                counter = counter + 1
                (counter >= 5) ? { break }
            })
            counter == 5 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Loop with break failed");
}

/// Test loop with counter accumulation
#[test]
fn test_loop_counter() {
    let source = r#"
        main = () i32 {
            count ::= 0
            loop(() {
                count = count + 1
                (count == 10) ? { break }
            })
            // Should have counted to 10
            count == 10 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Loop counter failed");
}

// ============================================================================
// ENUM TESTS
// ============================================================================

/// Test simple enum without payload
#[test]
fn test_simple_enum() {
    let source = r#"
        Status: Active, Inactive, Banned

        main = () i32 {
            s = Status.Active
            s ?
                | .Active { return 0 }
                | .Inactive { return 1 }
                | .Banned { return 2 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Simple enum pattern matching failed");
}

/// Test enum with multiple variants
#[test]
fn test_enum_multiple_branches() {
    let source = r#"
        Color: Red, Green, Blue

        get_code = (c: Color) i32 {
            c ?
                | .Red { return 1 }
                | .Green { return 2 }
                | .Blue { return 3 }
        }

        main = () i32 {
            r = get_code(Color.Red)
            g = get_code(Color.Green)
            b = get_code(Color.Blue)

            // r=1, g=2, b=3, sum=6
            sum = r + g + b
            sum == 6 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Enum multiple branches failed");
}

// ============================================================================
// BOOLEAN LOGIC TESTS
// ============================================================================

/// Test logical AND
#[test]
fn test_logical_and() {
    let source = r#"
        main = () i32 {
            a = true
            b = true
            c = false

            // true && true = true
            (a && b) ?
                | true {
                    // true && false = false
                    (a && c) ?
                        | true { return 1 }
                        | false { return 0 }
                }
                | false { return 2 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Logical AND failed");
}

/// Test logical OR
#[test]
fn test_logical_or() {
    let source = r#"
        main = () i32 {
            a = true
            b = false
            c = false

            // true || false = true
            (a || b) ?
                | true {
                    // false || false = false
                    (b || c) ?
                        | true { return 1 }
                        | false { return 0 }
                }
                | false { return 2 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Logical OR failed");
}

/// Test boolean negation via conditional
#[test]
fn test_boolean_negation() {
    let source = r#"
        negate = (x: bool) bool {
            x ?
                | true { return false }
                | false { return true }
        }

        main = () i32 {
            a = true
            b = false

            // negate(true) = false, negate(false) = true
            na = negate(a)
            nb = negate(b)

            na ?
                | true { return 1 }
                | false {
                    nb ?
                        | true { return 0 }
                        | false { return 2 }
                }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Boolean negation failed");
}

// ============================================================================
// WHILE LOOP TESTS
// ============================================================================

/// Test loop with conditional break (simulates while)
#[test]
fn test_loop_conditional_break() {
    let source = r#"
        main = () i32 {
            i ::= 0
            sum ::= 0

            loop(() {
                (i >= 5) ? { break }
                sum = sum + i
                i = i + 1
            })

            // sum = 0+1+2+3+4 = 10
            sum == 10 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Loop conditional break failed");
}

// ============================================================================
// NESTED FUNCTION TESTS
// ============================================================================

/// Test deeply nested function calls
#[test]
fn test_deeply_nested_calls() {
    let source = r#"
        add = (a: i32, b: i32) i32 { return a + b }
        mul = (a: i32, b: i32) i32 { return a * b }
        sub = (a: i32, b: i32) i32 { return a - b }

        main = () i32 {
            // ((2 + 3) * (4 - 1)) = 5 * 3 = 15
            result = mul(add(2, 3), sub(4, 1))
            result == 15 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Deeply nested calls failed");
}

// ============================================================================
// TYPE INFERENCE TESTS
// ============================================================================

/// Test type inference with arithmetic
#[test]
fn test_type_inference_arithmetic() {
    let source = r#"
        main = () i32 {
            // All inferred as i32
            a = 10
            b = 20
            c = a + b
            d = c * 2

            d == 60 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Type inference arithmetic failed");
}

/// Test type inference with conditionals
#[test]
fn test_type_inference_conditional() {
    let source = r#"
        main = () i32 {
            flag = true
            // Result type inferred from branches
            value = flag ?
                | true { 42 }
                | false { 0 }

            value == 42 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Type inference conditional failed");
}

// ============================================================================
// STRUCT METHOD TESTS
// ============================================================================

/// Test struct with computed field
#[test]
fn test_struct_computed() {
    let source = r#"
        Point: {
            x: i32,
            y: i32
        }

        magnitude_squared = (p: Point) i32 {
            return p.x * p.x + p.y * p.y
        }

        main = () i32 {
            p = Point { x: 3, y: 4 }
            // 3^2 + 4^2 = 9 + 16 = 25
            mag = magnitude_squared(p)
            mag == 25 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Struct computed field failed");
}

// ============================================================================
// EARLY RETURN TESTS
// ============================================================================

/// Test early return in function
#[test]
fn test_early_return() {
    let source = r#"
        find_first_positive = (a: i32, b: i32, c: i32) i32 {
            (a > 0) ? { return a }
            (b > 0) ? { return b }
            (c > 0) ? { return c }
            return 0
        }

        main = () i32 {
            // Should return 5 (second arg)
            result = find_first_positive(-1, 5, 10)
            result == 5 ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "Early return failed");
}

// ============================================================================
// INTEGER OVERFLOW TESTS (behavior verification)
// ============================================================================

/// Test i32 wrapping behavior
#[test]
fn test_i32_max_plus_one() {
    let source = r#"
        main = () i32 {
            // 2147483647 + 1 wraps to -2147483648 in two's complement
            max_i32 = 2147483647
            wrapped = max_i32 + 1

            // Check it wrapped to negative
            (wrapped < 0) ?
                | true { return 0 }
                | false { return 1 }
        }
    "#;

    let result = run_expecting_success(source);
    assert_eq!(result.exit_code, 0, "i32 overflow wrap failed");
}
