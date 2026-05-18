use std::process::Command;

#[test]
fn build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets() {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| .Err { "default" }"#,
        "build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| _ { "default" }"#,
        "build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_for_multiple_targets",
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| err { "default" }"#,
        "build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_for_multiple_targets",
    );
}

fn assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "app", main: "app.zen", out_dir: "build/app/" }})
    b.add(Executable {{ name: "tool", main: "tool.zen", out_dir: "build/tool/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("build.targets"), "app\ntool\n").expect("write manifest");
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
        .args(["build", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen build build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build build.zen failed: stdout={}, stderr={}",
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
