use std::process::Command;

#[test]
fn direct_file_command_multi_target_build_zen_rejects_undeclared_host_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "direct multi-target build.zen command should not start after graph validation fails"
    );
}

#[test]
fn direct_file_command_build_zen_rejects_undeclared_host_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

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
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
