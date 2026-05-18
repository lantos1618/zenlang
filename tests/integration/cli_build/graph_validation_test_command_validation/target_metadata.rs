use std::process::Command;

#[test]
fn test_command_build_zen_rejects_duplicate_test_target_fields() {
    assert_test_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        name: "integration",
        root: "test.zen",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Test` build target",
    );
}

#[test]
fn test_command_build_zen_rejects_missing_required_test_target_fields() {
    assert_test_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
    })
    .Ok(b.config())
}
"#,
        "missing required field `root` or `root_source_file` in `Test` build target",
    );
}

#[test]
fn test_command_build_zen_rejects_invalid_test_target_field_types() {
    assert_test_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        root: 42,
    })
    .Ok(b.config())
}
"#,
        "field `root` in `Test` build target must be a string",
    );
}

fn assert_test_command_rejects_target_metadata(build_source: &str, expected_diagnostic: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
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
        stderr.contains(expected_diagnostic),
        "expected target metadata diagnostic `{expected_diagnostic}`, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not create outputs after target metadata validation fails"
    );
}
