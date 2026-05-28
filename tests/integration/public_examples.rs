use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

#[test]
fn public_examples_typecheck_through_cli() {
    for path in [
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/07_behaviors_and_generics.zen",
        "examples/project/main.zen",
        "examples/project/math_utils.zen",
    ] {
        assert_zen_check_succeeds(path);
    }
}

#[test]
fn public_project_build_graph_typechecks_through_cli() {
    assert_zen_check_succeeds("examples/project/build.zen");
}

#[test]
fn public_project_build_graph_builds_and_tests_through_cli() {
    let project = copy_public_project_example();

    let build = run_zen(
        &["build", "build.zen"],
        project.path(),
        "build public project",
    );
    assert_command_success(&build, "zen build examples/project/build.zen");

    let binary_path = project.path().join("build").join("myapp");
    assert_binary_runs(&binary_path, project.path(), "public project binary");

    let test = run_zen(
        &["test", "build.zen"],
        project.path(),
        "test public project",
    );
    assert_command_success(&test, "zen test examples/project/build.zen");
}

#[test]
fn public_runnable_examples_compile_through_cli() {
    for path in [
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/07_behaviors_and_generics.zen",
        "examples/project/main.zen",
    ] {
        assert_zen_build_succeeds(path);
    }
}

#[test]
fn public_runnable_examples_execute_through_cli() {
    for path in [
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
        "examples/07_behaviors_and_generics.zen",
        "examples/project/main.zen",
    ] {
        assert_zen_build_and_run_succeeds(path);
    }
}

fn assert_zen_check_succeeds(path: &str) {
    let output = run_zen(
        &["check", path],
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &format!("check {path}"),
    );
    assert_command_success(&output, &format!("zen check {path}"));
}

fn assert_zen_build_succeeds(path: &str) {
    let _built = build_public_example(path);
}

fn assert_zen_build_and_run_succeeds(path: &str) {
    let built = build_public_example(path);
    assert_binary_runs(
        &built.binary_path,
        built.temp_dir.path(),
        &format!("compiled public example {path}"),
    );
}

struct BuiltPublicExample {
    temp_dir: TempDir,
    binary_path: PathBuf,
}

fn build_public_example(path: &str) -> BuiltPublicExample {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = run_zen(
        &["build", source_path.to_str().expect("utf-8 source path")],
        temp_dir.path(),
        &format!("build {path}"),
    );
    assert_command_success(&output, &format!("zen build {path}"));

    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("example path has utf-8 file stem");
    BuiltPublicExample {
        binary_path: temp_dir.path().join(stem),
        temp_dir,
    }
}

fn copy_public_project_example() -> TempDir {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let project_dir = temp_dir.path();
    let source_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/project");
    for file_name in ["build.zen", "main.zen", "math_utils.zen", "test.zen"] {
        fs::copy(source_dir.join(file_name), project_dir.join(file_name))
            .unwrap_or_else(|err| panic!("copy examples/project/{file_name}: {err}"));
    }
    temp_dir
}

fn run_zen(args: &[&str], current_dir: &Path, context: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(current_dir)
        .output()
        .unwrap_or_else(|err| panic!("run zen {context}: {err}"))
}

fn assert_binary_runs(binary_path: &Path, current_dir: &Path, context: &str) {
    let output = Command::new(binary_path)
        .current_dir(current_dir)
        .output()
        .unwrap_or_else(|err| panic!("run {context}: {err}"));
    assert_command_success(&output, context);
}

fn assert_command_success(output: &Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
