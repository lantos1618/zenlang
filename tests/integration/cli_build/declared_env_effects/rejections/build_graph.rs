use super::*;

#[test]
fn build_graph_command_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build-graph", "build.zen"],
        "build-graph command should not start after graph validation fails",
    );
}

#[test]
fn build_graph_command_multi_target_rejects_env_read_without_fallback_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["build-graph", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build-graph build.zen");

    assert_env_read_without_fallback_failed(&output);
    assert!(
        !tmp.path().join("build").exists(),
        "build-graph command should reject env effects before target execution"
    );
}
