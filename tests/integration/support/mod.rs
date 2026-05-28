use std::path::{Path, PathBuf};
use std::process::Command;

use zen::codegen::c;
use zen::error::FileTable;
use zen::typechecker::TypeChecker;
mod generated_c;

pub use generated_c::{
    assert_c_call_resolves_to_single_definition, assert_generated_c_calls_resolve_to_definitions,
    assert_generated_c_function_definitions_are_unique, assert_generated_c_specialization,
    has_c_call_outside_signature, undefined_generated_c_calls,
};

pub fn test_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/zen")
}

pub fn compile_to_c(zen_path: &Path) -> String {
    let mut files = FileTable::default();
    let graph =
        zen::module_system::load_module_graph(zen_path, &mut files).unwrap_or_else(|errs| {
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

    c::generate(&typed)
}

pub fn compile_to_c_with_generated_call_check(zen_path: &Path) -> String {
    let c_source = compile_to_c(zen_path);
    assert_generated_c_calls_resolve_to_definitions(&c_source);
    c_source
}

pub fn write_module(path: &Path, contents: &str) {
    std::fs::write(path, contents).unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
}

pub fn compile_error_message_for_modules(
    modules: &[(&str, &str)],
    main: &str,
    expectation: &str,
) -> String {
    let tmp = tempfile::tempdir().expect("create temp dir");
    for (name, contents) in modules {
        write_module(&tmp.path().join(name), contents);
    }
    let main_path = tmp.path().join("main.zen");
    write_module(&main_path, main);
    let panic = std::panic::catch_unwind(|| compile_to_c(&main_path)).expect_err(expectation);

    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("<non-string panic>")
        .to_string()
}

pub fn assert_message_contains_any(message: &str, expected: &[&str], context: &str) {
    assert!(
        expected.iter().any(|needle| message.contains(needle)),
        "{context}, panic={message}"
    );
}

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

pub fn run_test(name: &str) {
    let zen_path = test_dir().join(format!("{}.zen", name));
    let actual = compile_and_run(&zen_path);
    let expected_path = test_dir()
        .join("expected")
        .join(format!("{}.expected", name));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("read {}: {}", expected_path.display(), e));
    assert_eq!(
        actual, expected,
        "\n--- test: {} ---\nexpected:\n{}\nactual:\n{}",
        name, expected, actual
    );
}
