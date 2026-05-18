use super::*;

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
fn emit_command_build_zen_rejects_env_read_without_fallback_before_unselected_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "missing_test.zen" })
    b.add(Library { name: "core", exports: ["missing_lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("app.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write app.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert_env_read_without_fallback_failed(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("source not found"),
        "emit should reject env effects before unselected target source validation, stderr={stderr}"
    );
    assert!(
        output.stdout.is_empty(),
        "emit should not write C source after graph validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs"
    );
}
