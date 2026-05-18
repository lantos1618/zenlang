use std::process::Command;

#[test]
fn test_command_build_zen_rejects_undeclared_file_read_effects_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets")
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

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read file `test.targets`"),
        "expected undeclared file read diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}

#[test]
fn test_command_build_zen_rejects_undeclared_file_read_effects_before_unselected_targets() {
    assert_test_command_rejects_file_read_before_unselected_targets(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets")
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
        "undeclared",
    );
}

#[test]
fn test_command_build_zen_rejects_file_read_without_fallback_before_unselected_targets() {
    assert_test_command_rejects_file_read_before_unselected_targets(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
        "missing-fallback",
    );
}

fn assert_test_command_rejects_file_read_before_unselected_targets(
    build_source: &str,
    diagnostic_case: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    super::write_zero_main(tmp.path().join("unit.zen"));
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    1
}
"#,
    )
    .expect("write lib.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undeclared host effect: read file `test.targets`"),
        "expected {diagnostic_case} file read diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("missing_app.zen"),
        "host-effect validation should run before unrelated executable source handling, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}
