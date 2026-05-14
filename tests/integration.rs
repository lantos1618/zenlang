//! Integration tests for the Zen compiler pipeline.
//!
//! For each `.zen` file in `tests/zen/`, runs the full pipeline:
//! lex → parse → typecheck → C codegen → compile with cc → run → verify output.

use std::path::{Path, PathBuf};
use std::process::Command;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

/// Root of the test fixtures.
fn test_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zen")
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
fn compile_to_c(zen_path: &Path) -> String {
    // 1. Load & parse
    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let graph = module_system
        .load_module_graph(zen_path, &mut files)
        .unwrap_or_else(|errs| {
            panic!(
                "load/parse error in {}:\n  {}",
                zen_path.display(),
                errs.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    // 2. Resolve and typecheck the graph.
    let mut checker = TypeChecker::new();
    let typed = checker
        .check_module_graph_entry(&graph)
        .unwrap_or_else(|diags| {
            panic!(
                "typecheck error in {}:\n  {}",
                zen_path.display(),
                diags
                    .iter()
                    .map(|d| d.message.clone())
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    // 4. Codegen
    let backend = CBackend;
    backend
        .generate(&typed)
        .unwrap_or_else(|e| panic!("codegen error in {}: {}", zen_path.display(), e))
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
fn compile_and_run(zen_path: &Path) -> String {
    let c_source = compile_to_c(zen_path);

    // 4. Compile C → binary in a temp dir
    let tmp = tempfile::tempdir().expect("create temp dir");
    let c_path = tmp.path().join("out.c");
    let bin_path = tmp.path().join("out");

    std::fs::write(&c_path, &c_source).expect("write C source");

    let cc = std::env::var("CC").unwrap_or_else(|_| "cc".into());
    let compile = Command::new(&cc)
        .args([
            c_path.to_str().unwrap(),
            "-o",
            bin_path.to_str().unwrap(),
            "-lm",
        ])
        .output()
        .expect("run cc");

    if !compile.status.success() {
        panic!(
            "cc failed for {}:\n--- C source ---\n{}\n--- stderr ---\n{}",
            zen_path.display(),
            c_source,
            String::from_utf8_lossy(&compile.stderr),
        );
    }

    // 5. Run binary
    let run = Command::new(&bin_path).output().expect("run binary");
    assert!(
        run.status.success(),
        "binary exited with {} for {}",
        run.status,
        zen_path.display()
    );

    String::from_utf8(run.stdout).expect("stdout is utf-8")
}

/// Read the expected output for a test name.
fn expected_output(name: &str) -> String {
    let path = test_dir()
        .join("expected")
        .join(format!("{}.expected", name));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Run one end-to-end test by name.
fn run_test(name: &str) {
    let zen_path = test_dir().join(format!("{}.zen", name));
    let actual = compile_and_run(&zen_path);
    let expected = expected_output(name);
    assert_eq!(
        actual, expected,
        "\n--- test: {} ---\nexpected:\n{}\nactual:\n{}",
        name, expected, actual
    );
}

fn assert_c_function_definition(c_source: &str, name: &str) {
    let needle = format!(" {name}(");
    assert!(
        c_source
            .lines()
            .any(|line| line.trim_end().ends_with('{') && line.contains(&needle)),
        "expected generated C definition for `{name}`:\n{c_source}"
    );
}

fn assert_c_call_resolves_to_definition(c_source: &str, name: &str) {
    assert_c_function_definition(c_source, name);
    assert!(
        has_c_call_outside_signature(c_source, name),
        "expected generated C call to `{name}` outside declarations/definitions:\n{c_source}"
    );
}

fn has_c_call_outside_signature(c_source: &str, name: &str) -> bool {
    let call = format!("{name}(");
    c_source.lines().any(|line| {
        let trimmed = line.trim();
        trimmed.contains(&call) && !is_c_function_signature_line(trimmed, name)
    })
}

fn is_c_function_signature_line(trimmed: &str, name: &str) -> bool {
    let needle = format!(" {name}(");
    let Some(call_start) = trimmed.find(&needle) else {
        return false;
    };
    let prefix = &trimmed[..call_start];
    !prefix.contains('=')
        && !prefix.contains("return")
        && (trimmed.ends_with(';') || trimmed.ends_with('{'))
}

#[test]
fn c_call_assertion_ignores_struct_return_definitions() {
    let c_source = r#"
typedef struct Box_Option_i32 Box_Option_i32;

Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self) {
    return self;
}
"#;

    assert!(
        !has_c_call_outside_signature(c_source, "Box_copy_Option_i32"),
        "definition-only generated C should not count as a call"
    );
}

// ── Individual test cases ───────────────────────────────────────────

#[test]
fn test_hello() {
    run_test("hello");
}

#[test]
fn test_arithmetic() {
    run_test("arithmetic");
}

#[test]
fn test_structs() {
    run_test("structs");
}

#[test]
fn test_enums() {
    run_test("enums");
}

#[test]
fn test_ufc() {
    run_test("ufc");
}

#[test]
fn test_conditionals() {
    run_test("conditionals");
}

#[test]
fn test_loops() {
    run_test("loops");
}

#[test]
fn test_strings() {
    run_test("strings");
}

#[test]
fn test_functions() {
    run_test("functions");
}

#[test]
fn test_generic_identity() {
    run_test("generic_identity");
}

#[test]
fn test_generic_struct() {
    run_test("generic_struct");
}

#[test]
fn test_generic_enum_option() {
    run_test("generic_enum_option");
}

#[test]
fn test_generic_method() {
    run_test("generic_method");
}

#[test]
fn test_generic_method_self() {
    run_test("generic_method_self");
}

#[test]
fn test_generic_result_enum() {
    run_test("generic_result_enum");
}

#[test]
fn test_generic_vec() {
    run_test("generic_vec");
}

#[test]
fn test_generic_worklist() {
    run_test("generic_worklist");
}

#[test]
fn test_generic_worklist_dedup() {
    run_test("generic_worklist_dedup");
}

#[test]
fn test_generic_ufc_function() {
    run_test("generic_ufc_function");
}

#[test]
fn test_behavior_json_explicit_impl() {
    run_test("behavior_json_explicit_impl");
}

#[test]
fn test_behavior_default_method_dispatch() {
    run_test("behavior_default_method_dispatch");
}

#[test]
fn test_behavior_inherited_default_method() {
    run_test("behavior_inherited_default_method");
}

#[test]
fn test_behavior_json_generic_dispatch() {
    run_test("behavior_json_generic_dispatch");
}

#[test]
fn test_behavior_json_generic_association() {
    run_test("behavior_json_generic_association");
}

#[test]
fn test_behavior_json_generic_bound() {
    run_test("behavior_json_generic_bound");
}

#[test]
fn test_behavior_json_generic_bound_ufcs() {
    run_test("behavior_json_generic_bound_ufcs");
}

#[test]
fn test_behavior_generic_parent_inheritance() {
    run_test("behavior_generic_parent_inheritance");
}

#[test]
fn test_behavior_inherited_generic_dispatch() {
    run_test("behavior_inherited_generic_dispatch");
}

#[test]
fn generic_specializations_do_not_emit_unspecialized_c_symbols() {
    let c_source = compile_to_c(&test_dir().join("generic_method.zen"));
    assert!(c_source.contains("int32_t Box_get_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_get_i32(box)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_get_i32");
    assert!(!c_source.contains("Box_T"));
    assert!(!c_source.contains("T Box_get"));

    let c_source = compile_to_c(&test_dir().join("generic_method_self.zen"));
    assert!(c_source.contains("Box_i32 Box_copy_i32(Box_i32 self)"));
    assert!(c_source.contains("Box_copy_i32(box)"));
    assert!(c_source.contains("Box_Option_i32 Box_copy_Option_i32(Box_Option_i32 self)"));
    assert!(c_source.contains("Box_copy_Option_i32(nested)"));
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_i32");
    assert_c_call_resolves_to_definition(&c_source, "Box_copy_Option_i32");
    assert!(
        c_source
            .find("struct Option_i32")
            .expect("Option_i32 struct")
            < c_source
                .find("struct Box_Option_i32")
                .expect("Box_Option_i32 struct")
    );
    assert!(!c_source.contains("Box_copy(box"));
    assert!(!c_source.contains("Unknown"));

    let c_source = compile_to_c(&test_dir().join("generic_vec.zen"));
    assert!(c_source.contains("int32_t Vec_len_i32(Vec_i32 self)"));
    assert!(c_source.contains("int32_t Vec_len_str(Vec_str self)"));
    assert!(c_source.contains("Vec_len_i32(ints)"));
    assert!(c_source.contains("Vec_len_str(words)"));
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_i32");
    assert_c_call_resolves_to_definition(&c_source, "Vec_len_str");
    assert!(!c_source.contains("Vec_T"));
    assert!(!c_source.contains("T Vec_len"));

    let c_source = compile_to_c(&test_dir().join("generic_worklist.zen"));
    assert!(c_source.contains("int32_t inner_i32(int32_t value)"));
    assert!(c_source.contains("int32_t outer_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "outer_i32");
    assert_eq!(
        c_source.matches("int32_t inner_i32(int32_t value)").count(),
        2
    );
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c(&test_dir().join("generic_worklist_dedup.zen"));
    assert!(c_source.contains("int32_t left_i32(int32_t value)"));
    assert!(c_source.contains("int32_t right_i32(int32_t value)"));
    assert!(c_source.contains("inner_i32(value)"));
    assert_c_call_resolves_to_definition(&c_source, "inner_i32");
    assert_c_call_resolves_to_definition(&c_source, "left_i32");
    assert_c_call_resolves_to_definition(&c_source, "right_i32");
    assert_eq!(
        c_source.matches("int32_t inner_i32(int32_t value)").count(),
        2
    );
    assert!(!c_source.contains("T inner"));
    assert!(!c_source.contains("inner_T"));

    let c_source = compile_to_c(&test_dir().join("generic_enum_option.zen"));
    assert!(c_source.contains("typedef struct Option_i32 Option_i32;"));
    assert!(c_source.contains("int32_t unwrap_or_i32(Option_i32 value, int32_t fallback)"));
    assert!(c_source.contains("Option_i32_Some"));
    assert!(c_source.contains("unwrap_or_i32(x, 0LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32");
    assert!(!c_source.contains("Option_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(x"));

    let c_source = compile_to_c(&test_dir().join("generic_result_enum.zen"));
    assert!(c_source.contains("typedef struct Result_i32_str Result_i32_str;"));
    assert!(c_source.contains("int32_t unwrap_or_i32_str(Result_i32_str value, int32_t fallback)"));
    assert!(c_source.contains("Result_i32_str_Err"));
    assert!(c_source.contains("unwrap_or_i32_str(err, 9LL)"));
    assert_c_call_resolves_to_definition(&c_source, "unwrap_or_i32_str");
    assert!(!c_source.contains("Result_T"));
    assert!(!c_source.contains("T unwrap_or"));
    assert!(!c_source.contains("unwrap_or(err"));

    let c_source = compile_to_c(&test_dir().join("generic_ufc_function.zen"));
    assert!(c_source.contains("int32_t id_i32(int32_t value)"));
    assert!(c_source.contains("id_i32(12LL)"));
    assert_c_call_resolves_to_definition(&c_source, "id_i32");
    assert!(!c_source.contains("id(12LL)"));
    assert!(!c_source.contains("T id"));

    let c_source = compile_to_c(&test_dir().join("behavior_json_generic_bound_ufcs.zen"));
    assert!(c_source.contains("Point Point_encode(Point value)"));
    assert!(c_source.contains("Point_encode(value)"));
    assert!(!c_source.contains("T_encode"));
}

#[test]
fn check_command_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    return missing_local
}
"#,
    )
    .expect("write test file");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    return a + b
}

pub broken = () i32 {
    return missing_dep_local
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    return add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("unknown value symbol 'missing_dep_local'"),
        "expected imported module resolver diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    return a + b
}

pub broken = () i32 {
    return true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    return add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", main_path.to_str().unwrap()])
        .output()
        .expect("run zen check");

    assert!(
        !output.status.success(),
        "zen check unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emit_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    return a + b
}

pub broken = () i32 {
    return true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    return add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit");

    assert!(
        !output.status.success(),
        "zen emit unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_command_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    return a + b
}

pub broken = () i32 {
    return true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    return add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build", main_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build");

    assert!(
        !output.status.success(),
        "zen build unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn build_command_rejects_build_zen_until_deterministic_graph_exists() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
main = () i32 {
    return 0
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build", build_path.to_str().unwrap()])
        .output()
        .expect("run zen build");

    assert!(
        !output.status.success(),
        "zen build build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            "build.zen execution is gated until deterministic build graph support exists"
        ),
        "expected build.zen gated diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn integration_frontend_helper_runs_resolver_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_resolver_ref.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    return missing_local
}
"#,
    )
    .expect("write test file");

    let panic = std::panic::catch_unwind(|| compile_to_c(&zen_path))
        .expect_err("compile_to_c should reject resolver errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("unknown value symbol 'missing_local'"),
        "expected resolver diagnostic, panic={message}"
    );
}

#[test]
fn integration_frontend_helper_reports_imported_module_type_diagnostics() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    return a + b
}

pub broken = () i32 {
    return true
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    return add(1, 2)
}
"#,
    )
    .expect("write entry module");

    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path))
        .expect_err("compile_to_c should reject imported module type errors");
    let message = panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>");

    assert!(
        message.contains("return type mismatch: expected `i32`, found `bool`"),
        "expected imported module type diagnostic, panic={message}"
    );
}

#[test]
fn test_defer() {
    run_test("defer");
}

#[test]
fn test_defer_early_return() {
    run_test("defer_early_return");
}

#[test]
fn test_boolean_ops() {
    run_test("boolean_ops");
}

#[test]
fn test_nested_structs() {
    run_test("nested_structs");
}

#[test]
fn test_enum_match() {
    run_test("enum_match");
}

#[test]
fn test_mutability() {
    run_test("mutability");
}

#[test]
fn test_recursion() {
    run_test("recursion");
}

#[test]
fn test_nested_match() {
    run_test("nested_match");
}

#[test]
fn test_cast() {
    run_test("cast");
}

#[test]
fn test_multiple_defer() {
    run_test("multiple_defer");
}

#[test]
fn test_multi_file_imports() {
    let zen_path = test_dir().join("multi_file/main.zen");
    let actual = compile_and_run(&zen_path);
    assert_eq!(actual, "37\n");
}

// ── Discovery test: all .zen files have matching .expected ──────────

#[test]
fn all_zen_files_have_expected_output() {
    let dir = test_dir();
    let mut missing = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("read tests/zen") {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("zen") {
            let stem = path.file_stem().unwrap().to_str().unwrap();
            let expected = dir.join("expected").join(format!("{}.expected", stem));
            if !expected.exists() {
                missing.push(stem.to_string());
            }
        }
    }
    assert!(
        missing.is_empty(),
        "missing .expected files for: {:?}",
        missing
    );
}
