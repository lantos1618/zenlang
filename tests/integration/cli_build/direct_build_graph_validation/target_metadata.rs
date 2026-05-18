use std::process::Command;

#[test]
fn direct_file_command_build_zen_rejects_duplicate_target_fields() {
    assert_direct_file_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        name: "tool",
        main: "app.zen",
        out_dir: "build/app/",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Executable` build target",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_missing_required_target_fields() {
    assert_direct_file_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
    })
    .Ok(b.config())
}
"#,
        "missing required field `out_dir` in `Executable` build target",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_invalid_target_field_types() {
    assert_direct_file_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: 42,
    })
    .Ok(b.config())
}
"#,
        "field `out_dir` in `Executable` build target must be a string",
    );
}

fn assert_direct_file_command_rejects_target_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        !output.status.success(),
        "zen build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
        "direct build.zen command should not create outputs after target metadata validation fails"
    );
}
