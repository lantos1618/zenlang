use std::process::Command;

#[test]
fn direct_file_command_build_zen_accepts_declared_file_read_effects() {
    assert_direct_file_command_accepts_declared_file_read_effect(
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_file_read_effects",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects() {
    assert_direct_file_command_accepts_declared_file_read_effect(
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects() {
    assert_direct_file_command_accepts_declared_file_read_effect(
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects",
    );
}

fn assert_direct_file_command_accepts_declared_file_read_effect(
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
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("build.targets"), "myapp\n").expect("write manifest");
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
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let bin_path = tmp.path().join("build").join("myapp");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );
    let run = Command::new(&bin_path).output().expect("run built binary");
    assert!(
        run.status.success(),
        "built binary exited with {}",
        run.status
    );
}

#[test]
fn direct_file_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets() {
    assert_direct_file_command_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| .Err { "default" }"#,
        "direct_file_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_direct_file_command_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| _ { "default" }"#,
        "direct_file_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_for_multiple_targets",
    );
}

#[test]
fn direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_direct_file_command_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| err { "default" }"#,
        "direct_file_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_for_multiple_targets",
    );
}

fn assert_direct_file_command_accepts_declared_file_read_effects_for_multiple_targets(
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
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        output.status.success(),
        "{case_name}: zen build.zen failed: stdout={}, stderr={}",
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
