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
    let (_tmp, output) = super::run_test_command_build_zen(
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
        &[
            ("unit.zen", &super::passing_main_source(0)),
            ("lib.zen", &super::passing_main_source(1)),
        ],
    );

    super::assert_test_command_succeeded(&output, case_name);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("test unit passed"),
        "expected selected test target to pass, stdout={stdout}"
    );
}
