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
fn compile_and_run(zen_path: &Path) -> String {
    // 1. Load & parse
    let mut files = FileTable::new();
    let mut module_system = ModuleSystem::new();
    let program = module_system
        .load_file(zen_path, &mut files)
        .unwrap_or_else(|errs| {
            panic!(
                "parse error in {}:\n  {}",
                zen_path.display(),
                errs.iter()
                    .map(|e| format!("{}", e))
                    .collect::<Vec<_>>()
                    .join("\n  ")
            );
        });

    // 2. Typecheck
    let mut checker = TypeChecker::new();
    let typed = checker.check_program(&program).unwrap_or_else(|diags| {
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

    // 3. Codegen
    let backend = CBackend;
    let c_source = backend.generate(&typed).unwrap_or_else(|e| {
        panic!("codegen error in {}: {}", zen_path.display(), e);
    });

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
fn test_defer() {
    run_test("defer");
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
