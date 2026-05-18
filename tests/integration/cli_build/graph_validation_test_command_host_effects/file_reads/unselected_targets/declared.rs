use std::process::Command;

#[test]
fn test_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets() {
    assert_test_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| .Err { "default" }"#,
        "test_command_build_zen_accepts_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_test_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| _ { "default" }"#,
        "test_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

#[test]
fn test_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets(
) {
    assert_test_command_accepts_declared_file_read_effects_with_unselected_targets(
        r#"| err { "default" }"#,
        "test_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_with_unselected_targets",
    );
}

fn assert_test_command_accepts_declared_file_read_effects_with_unselected_targets(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("test.targets") ?
        | .Ok(contents) {{ contents }}
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
    std::fs::write(tmp.path().join("test.targets"), "unit\n").expect("write manifest");
    super::write_zero_main(tmp.path().join("unit.zen"));
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

    let bin_path = tmp.path().join("build").join("tests").join("unit");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("test unit passed"),
        "{case_name}: expected test pass output, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}
