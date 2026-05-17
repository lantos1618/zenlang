use std::process::Command;

#[test]
fn test_command_build_zen_accepts_declared_file_read_effects() {
    assert_test_command_accepts_declared_file_read_effect(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_file_read_effects",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects() {
    assert_test_command_accepts_declared_file_read_effect(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_file_read_effects() {
    assert_test_command_accepts_declared_file_read_effect(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_file_read_effects",
    );
}

fn assert_test_command_accepts_declared_file_read_effect(fallback_arm: &str, case_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Test {{ name: "unit", root: "test.zen" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("test.targets"), "unit\n").expect("write manifest");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bin_path = tmp.path().join("build").join("tests").join("unit");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test unit passed"),
        "{case_name}: expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) { contents }
        | .Err { "default" }
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Test { name: "integration", root: "integration.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("test.targets"), "unit\nintegration\n").expect("write manifest");
    std::fs::write(
        tmp.path().join("unit.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write unit.zen");
    std::fs::write(
        tmp.path().join("integration.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write integration.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");

    assert!(
        output.status.success(),
        "zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for test_name in ["unit", "integration"] {
        let bin_path = tmp.path().join("build").join("tests").join(test_name);
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(&format!("test {test_name} passed")),
            "expected {test_name} pass output, stdout={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

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
fn test_command_multi_target_build_zen_rejects_undeclared_file_read_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets")
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Test { name: "integration", root: "integration.zen" })
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
        "multi-target test command should not start after graph validation fails"
    );
}

#[test]
fn test_command_build_zen_rejects_file_read_without_fallback_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) { contents }
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
fn test_command_multi_target_build_zen_rejects_file_read_without_fallback_before_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) { contents }
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Test { name: "integration", root: "integration.zen" })
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
        "multi-target test command should not start after graph validation fails"
    );
}
