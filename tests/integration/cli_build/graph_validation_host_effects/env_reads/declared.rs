use std::process::Command;

#[test]
fn check_command_build_zen_accepts_declared_env_read_with_fallback() {
    assert_check_command_accepts_declared_env_read_fallback(
        r#"| .Err { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_declared_env_read_with_fallback",
    );
}

#[test]
fn check_command_build_zen_accepts_wildcard_fallback_declared_env_read() {
    assert_check_command_accepts_declared_env_read_fallback(
        r#"| _ { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_wildcard_fallback_declared_env_read",
    );
}

#[test]
fn check_command_build_zen_accepts_identifier_fallback_declared_env_read() {
    assert_check_command_accepts_declared_env_read_fallback(
        r#"| err { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_identifier_fallback_declared_env_read",
    );
}

fn assert_check_command_accepts_declared_env_read_fallback(fallback_arm: &str, case_name: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(path) {{ path }}
        {fallback_arm}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("main.zen"),
        r#"
main = () i32 {
    0
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
        output.status.success(),
        "{case_name}: zen check build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 build targets"),
        "{case_name}: expected build graph check summary, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn check_command_build_zen_accepts_declared_env_read_for_multiple_targets() {
    assert_check_command_accepts_declared_env_read_for_multiple_targets(
        r#"| .Err { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn check_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets() {
    assert_check_command_accepts_declared_env_read_for_multiple_targets(
        r#"| _ { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_wildcard_fallback_declared_env_read_for_multiple_targets",
    );
}

#[test]
fn check_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets() {
    assert_check_command_accepts_declared_env_read_for_multiple_targets(
        r#"| err { "~/.zen/std" }"#,
        "check_command_build_zen_accepts_identifier_fallback_declared_env_read_for_multiple_targets",
    );
}

fn assert_check_command_accepts_declared_env_read_for_multiple_targets(
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
        | .Ok(path) {{ path }}
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
        .args(["check", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen check build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen check build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("3 build targets"),
        "expected build graph check summary, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen check build.zen should not compile graph targets"
    );
}
