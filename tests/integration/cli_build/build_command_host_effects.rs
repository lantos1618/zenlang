use std::process::Command;

#[path = "build_command_host_effects/file_reads.rs"]
mod file_reads;

#[test]
fn build_command_build_zen_rejects_undeclared_host_effects() {
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        !output.status.success(),
        "zen build build.zen unexpectedly succeeded: stdout={}, stderr={}",
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

#[test]
fn build_command_build_zen_rejects_undeclared_host_effects_before_dependency_execution() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })
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

    assert!(
        !output.status.success(),
        "zen build build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
        "dependency-ordered build should not start after graph validation fails"
    );
}

#[test]
fn build_command_multi_target_build_zen_rejects_undeclared_host_effects() {
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        !output.status.success(),
        "zen build build.zen unexpectedly succeeded: stdout={}, stderr={}",
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

#[test]
fn build_command_build_zen_rejects_undeclared_host_effects_before_skipping_unrelated_tests() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Test { name: "unit", root: "missing_test.zen" })
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        !output.status.success(),
        "zen build build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("missing_test.zen"),
        "host-effect validation should run before unrelated test source handling, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "build should not start after graph validation fails"
    );
}
