use std::path::Path;
use std::process::Command;

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
