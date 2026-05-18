use std::process::Command;

#[test]
fn test_command_build_zen_rejects_unknown_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "test.zen",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target `unit` depends on unknown target `core`",
    );
}

#[test]
fn test_command_build_zen_rejects_self_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "test.zen",
        dependencies: ["unit"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target `unit` cannot depend on itself",
    );
}

#[test]
fn test_command_build_zen_rejects_cyclic_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: "unit.zen",
        dependencies: ["integration"],
    })
    b.add(Test {
        name: "integration",
        root: "integration.zen",
        dependencies: ["unit"],
    })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    assert_test_command_rejects_dependency_shape(
        tmp.path(),
        "build target dependency cycle includes `integration`",
    );
}

fn assert_test_command_rejects_dependency_shape(
    project_dir: &std::path::Path,
    expected_diagnostic: &str,
) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(project_dir)
        .output()
        .expect("run zen test build.zen");

    assert!(
        !output.status.success(),
        "zen test build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected dependency diagnostic `{expected_diagnostic}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !project_dir.join("build").exists(),
        "test command should not create outputs after dependency validation fails"
    );
}
