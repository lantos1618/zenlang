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

fn assert_test_command_accepts_declared_env_read(fallback_arm: &str, case_name: &str) {
    let (_tmp, output) = super::run_test_command_build_zen(
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Test {{ name: "unit", root: "unit.zen" }})
    .Ok(b.config())
}}
"#,
        ),
        &[("unit.zen", &super::passing_main_source(0))],
    );

    super::assert_test_command_succeeded(&output, case_name);
}
