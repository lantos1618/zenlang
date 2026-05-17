use std::process::Command;

use super::*;

#[test]
fn build_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build", "build.zen"],
        "build command should not start after graph validation fails",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build.zen"],
        "direct build.zen command should not start after graph validation fails",
    );
}

#[test]
fn build_graph_command_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build-graph", "build.zen"],
        "build-graph command should not start after graph validation fails",
    );
}

#[test]
fn check_command_build_zen_rejects_env_read_without_fallback_before_source_validation() {
    let tmp = executable_graph_with_env_read_without_fallback("missing.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert_env_read_without_fallback_failed(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("source not found"),
        "host-effect validation should run before source validation, stderr={stderr}"
    );
}

#[test]
fn emit_command_build_zen_rejects_env_read_without_fallback() {
    let tmp = executable_graph_with_env_read_without_fallback("main.zen");
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write main.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert_env_read_without_fallback_failed(&output);
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert_env_read_without_fallback_failed(&output);
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}
