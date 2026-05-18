use std::process::Command;

use super::{assert_executable_command_accepts_declared_env_read, ExecutableCommandExpectation};

#[test]
fn emit_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| .Err { "default" }"#,
        "emit_command_build_zen_accepts_declared_env_read_with_fallback",
        ExecutableCommandExpectation::EmitStdout,
    );
}

#[test]
fn emit_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| _ { "default" }"#,
        "emit_command_build_zen_accepts_wildcard_fallback_declared_env_read",
        ExecutableCommandExpectation::EmitStdout,
    );
}

#[test]
fn emit_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_executable_command_accepts_declared_env_read(
        &["emit", "build.zen"],
        r#"| err { "default" }"#,
        "emit_command_build_zen_accepts_identifier_fallback_declared_env_read",
        ExecutableCommandExpectation::EmitStdout,
    );
}

#[test]
fn emit_command_build_zen_accepts_declared_env_read_with_unselected_targets() {
    assert_emit_command_accepts_declared_env_read_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "emit_command_build_zen_accepts_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn emit_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets() {
    assert_emit_command_accepts_declared_env_read_with_unselected_targets(
        r#"| _ { "default" }"#,
        "emit_command_build_zen_accepts_wildcard_fallback_declared_env_read_with_unselected_targets",
    );
}

#[test]
fn emit_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets() {
    assert_emit_command_accepts_declared_env_read_with_unselected_targets(
        r#"| err { "default" }"#,
        "emit_command_build_zen_accepts_identifier_fallback_declared_env_read_with_unselected_targets",
    );
}

fn assert_emit_command_accepts_declared_env_read_with_unselected_targets(
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
    b.add(Test {{ name: "unit", root: "unit.zen" }})
    b.add(Library {{ name: "core", exports: ["lib.zen"] }})
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
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen emit build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("int32_t zen_main(void)"),
        "expected C output after declared env effect, stdout={stdout}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen emit build.zen should not create build outputs"
    );
}
