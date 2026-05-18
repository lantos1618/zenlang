use super::*;

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
fn check_command_multi_target_build_zen_rejects_env_read_without_fallback() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_unit.zen" })
    b.add(Library { name: "core", exports: ["missing_lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert_env_read_without_fallback_failed(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("source not found"),
        "host-effect validation should run before multi-target source validation, stderr={stderr}"
    );
}
