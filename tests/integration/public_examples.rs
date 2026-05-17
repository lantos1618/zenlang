use std::path::Path;
use std::process::Command;
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
fn public_runnable_examples_compile_through_cli() {
    for path in [
        "examples/01_hello_world.zen",
        "examples/02_variables_and_types.zen",
        "examples/03_pattern_matching.zen",
        "examples/04_structs_and_methods.zen",
        "examples/05_loops.zen",
        "examples/06_error_handling.zen",
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
        "examples/project/main.zen",
    ] {
        assert_zen_build_and_run_succeeds(path);
    }
}

fn assert_zen_check_succeeds(path: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", path])
        .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")))
        .output()
        .unwrap_or_else(|err| panic!("run zen check {path}: {err}"));

    assert!(
        output.status.success(),
        "zen check {path} failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_zen_build_succeeds(path: &str) {
    let _built = build_public_example(path);
}

fn assert_zen_build_and_run_succeeds(path: &str) {
    let built = build_public_example(path);
    let run = Command::new(&built.binary_path)
        .current_dir(built.temp_dir.path())
        .output()
        .unwrap_or_else(|err| panic!("run compiled public example {path}: {err}"));

    assert!(
        run.status.success(),
        "compiled public example {path} exited with {}: stdout={}, stderr={}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
}

struct BuiltPublicExample {
    temp_dir: TempDir,
    binary_path: std::path::PathBuf,
}

fn build_public_example(path: &str) -> BuiltPublicExample {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build", source_path.to_str().expect("utf-8 source path")])
        .current_dir(temp_dir.path())
        .output()
        .unwrap_or_else(|err| panic!("run zen build {path}: {err}"));

    assert!(
        output.status.success(),
        "zen build {path} failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .expect("example path has utf-8 file stem");
    BuiltPublicExample {
        binary_path: temp_dir.path().join(stem),
        temp_dir,
    }
}
