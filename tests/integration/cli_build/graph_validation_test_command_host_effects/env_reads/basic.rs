#[test]
fn test_command_build_zen_rejects_undeclared_host_effects() {
    let args = ["test", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}

#[test]
fn test_command_multi_target_build_zen_rejects_undeclared_host_effects() {
    let args = ["test", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Test { name: "unit", root: "unit.zen" })
    b.add(Test { name: "integration", root: "integration.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("unit.zen"), main_source("0")).expect("write unit.zen");
    std::fs::write(tmp.path().join("integration.zen"), main_source("0"))
        .expect("write integration.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}

fn main_source(value: &str) -> String {
    format!(
        r#"
main = () i32 {{
    {value}
}}
"#,
    )
}
