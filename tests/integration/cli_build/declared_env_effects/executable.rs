use std::process::Command;

use super::{assert_executable_command_accepts_declared_env_read, ExecutableCommandExpectation};

#[test]
fn build_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn build_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build", "build.zen"],
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["build.zen"],
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::BuildOutput,
    );
}

#[test]
fn direct_file_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build.zen"],
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets()
{
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build.zen"],
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets(
) {
    assert_executable_command_accepts_declared_env_read_for_multiple_targets(
        &["build.zen"],
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

fn assert_executable_command_accepts_declared_env_read_for_multiple_targets(
    args: &[&str],
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
    b.add(Executable {{ name: "app", main: "app.zen", out_dir: "build/app/" }})
    b.add(Executable {{ name: "tool", main: "tool.zen", out_dir: "build/tool/" }})
    .Ok(b.config())
}}
"#,
        ),
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
    std::fs::write(
        tmp.path().join("tool.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write tool.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen executable build graph command");

    assert!(
        output.status.success(),
        "{case_name}: zen executable build graph command failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
    }
}
