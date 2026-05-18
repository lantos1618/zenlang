use std::process::Command;

#[test]
fn test_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_test_command_accepts_declared_env_read(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_env_read_with_fallback",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_test_command_accepts_declared_env_read(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_env_read",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_test_command_accepts_declared_env_read(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_env_read",
    );
}

#[test]
fn test_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    assert_test_command_accepts_declared_env_read_for_multiple_targets(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_test_command_accepts_declared_env_read_for_multiple_targets(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_test_command_accepts_declared_env_read_for_multiple_targets(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_declared_env_read_with_unselected_targets() {
    assert_test_command_accepts_declared_env_read_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets() {
    assert_test_command_accepts_declared_env_read_with_unselected_targets(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets() {
    assert_test_command_accepts_declared_env_read_with_unselected_targets(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets",
    );
}

fn assert_test_command_accepts_declared_env_read_with_unselected_targets(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "missing_app.zen", out_dir: "build/app/" }})
    b.add(Test {{ name: "unit", root: "unit.zen" }})
    b.add(Library {{ name: "core", exports: ["lib.zen"] }})
    .Ok(b.config())
}}
"#,
        ),
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
        output.status.success(),
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test unit passed"),
        "expected selected test target to pass, stdout={stdout}"
    );
}

fn assert_test_command_accepts_declared_env_read_for_multiple_targets(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Test {{ name: "unit", root: "unit.zen" }})
    b.add(Test {{ name: "integration", root: "integration.zen" }})
    .Ok(b.config())
}}
"#,
        ),
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
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test unit passed") && stdout.contains("test integration passed"),
        "expected both test targets to pass, stdout={stdout}"
    );
}

fn assert_test_command_accepts_declared_env_read(fallback_arm: &str, case_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Test {{ name: "unit", root: "test.zen" }})
    .Ok(b.config())
}}
"#,
        ),
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
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
