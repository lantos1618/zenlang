#[test]
fn test_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking() {
    let args = ["test", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Test { name: "unit", root: "test.zen" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("test.zen"), main_source("0")).expect("write test.zen");
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    true
}
"#,
    )
    .expect("write lib.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("return type mismatch"),
        "host-effect validation should run before graph-only library typechecking, stderr={stderr}"
    );
    assert!(
        !tmp.path().join("build").exists(),
        "test command should not start after graph validation fails"
    );
}

#[test]
fn test_command_build_zen_rejects_undeclared_host_effects_before_skipping_unrelated_executables() {
    let args = ["test", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "app", main: "missing_app.zen", out_dir: "build/app/" })
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(tmp.path().join("test.zen"), main_source("0")).expect("write test.zen");

    let output = super::super::super::support::run_zen_in(&tmp, &args);

    super::super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "undeclared host effect: read env `ZEN_STD`",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("missing_app.zen"),
        "host-effect validation should run before unrelated executable source handling, stderr={stderr}"
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
