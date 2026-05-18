use super::*;

#[test]
fn build_command_build_zen_rejects_env_read_without_fallback_before_execution() {
    assert_env_read_without_fallback_is_rejected(
        &["build", "build.zen"],
        "build command should not start after graph validation fails",
    );
}

#[test]
fn build_command_multi_target_build_zen_rejects_env_read_without_fallback_before_execution() {
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert_env_read_without_fallback_failed(&output);
    assert!(
        !tmp.path().join("build").exists(),
        "build command should reject env effects before target execution"
    );
}

#[test]
fn build_command_build_zen_rejects_env_read_without_fallback_before_unselected_targets() {
    assert_env_read_without_fallback_before_unselected_targets(
        &["build", "build.zen"],
        "build command should reject env effects before selected target execution",
    );
}
