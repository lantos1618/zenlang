use std::path::{Path, PathBuf};
use std::process::Command;

use zen::codegen::c::CBackend;
use zen::codegen::Backend;
use zen::error::FileTable;
use zen::module_system::ModuleSystem;
use zen::typechecker::TypeChecker;

#[path = "support/generated_c.rs"]
mod generated_c;

pub use generated_c::{
    assert_c_call_resolves_to_definition, assert_c_function_definition_count,
    assert_generated_c_calls_resolve_to_definitions,
    assert_generated_c_function_definitions_are_unique, has_c_call_outside_signature,
    undefined_generated_c_calls,
};

/// Root of the test fixtures.
pub fn test_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zen")
}

/// Run the full pipeline for a `.zen` file and return generated C.
pub fn compile_to_c(zen_path: &Path) -> String {
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

    let backend = CBackend;
    backend
        .generate(&typed)
        .unwrap_or_else(|e| panic!("codegen error in {}: {}", zen_path.display(), e))
}

pub fn compile_to_c_with_generated_call_check(zen_path: &Path) -> String {
    let c_source = compile_to_c(zen_path);
    assert_generated_c_calls_resolve_to_definitions(&c_source);
    c_source
}

/// Run the full pipeline for a `.zen` file and return stdout of the compiled binary.
pub fn compile_and_run(zen_path: &Path) -> String {
    let c_source = compile_to_c_with_generated_call_check(zen_path);

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
pub fn expected_output(name: &str) -> String {
    let path = test_dir()
        .join("expected")
        .join(format!("{}.expected", name));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e))
}

/// Run one end-to-end test by name.
pub fn run_test(name: &str) {
    let zen_path = test_dir().join(format!("{}.zen", name));
    let actual = compile_and_run(&zen_path);
    let expected = expected_output(name);
    assert_eq!(
        actual, expected,
        "\n--- test: {} ---\nexpected:\n{}\nactual:\n{}",
        name, expected, actual
    );
}
