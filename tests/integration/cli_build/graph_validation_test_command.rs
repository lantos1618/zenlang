use std::process::Command;

#[test]
fn test_command_build_zen_runs_test_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
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
        "zen test build.zen failed: stdout={}, stderr={}",
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
        "expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn test_command_build_zen_runs_multiple_test_targets() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Test { name: "integration", root: "integration.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
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

    for name in ["unit", "integration"] {
        let bin_path = tmp.path().join("build").join("tests").join(name);
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(&format!("test {name} passed")),
            "expected {name} pass output, stdout={}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
}

#[test]
fn test_command_build_zen_runs_test_dependencies_first() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "integration",
        root: "integration.zen",
        dependencies: ["unit"],
    })
    b.add(Test { name: "unit", root: "unit.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let unit_pass = stdout
        .find("test unit passed")
        .unwrap_or_else(|| panic!("expected unit test pass output, stdout={stdout}"));
    let integration_pass = stdout
        .find("test integration passed")
        .unwrap_or_else(|| panic!("expected integration test pass output, stdout={stdout}"));
    assert!(
        unit_pass < integration_pass,
        "expected dependency test target to run before dependent target, stdout={stdout}"
    );

    for name in ["unit", "integration"] {
        let bin_path = tmp.path().join("build").join("tests").join(name);
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
    }
}
