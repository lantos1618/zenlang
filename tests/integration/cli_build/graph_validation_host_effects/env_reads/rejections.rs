use std::process::Command;

#[test]
fn check_command_build_zen_rejects_undeclared_host_effects() {
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
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
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
fn check_command_build_zen_rejects_undeclared_host_effects_before_source_validation() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "missing.zen", out_dir: "build/" })
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

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("source not found"),
        "host-effect validation should run before source validation, stderr={stderr}"
    );
}

#[test]
fn check_command_build_zen_rejects_undeclared_host_effects_before_target_typechecking() {
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
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write main.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        !output.status.success(),
        "zen check build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared host effect diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("return type mismatch"),
        "host-effect validation should run before target typechecking, stderr={stderr}"
    );
}
