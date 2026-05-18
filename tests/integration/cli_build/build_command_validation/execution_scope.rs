#[test]
fn build_command_build_zen_ignores_unrelated_gated_test_source_errors() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "missing_test.zen" })
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    .Ok(b.config())
}
"#,
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

    let output = super::super::support::run_zen_in(&tmp, &args);

    super::super::support::assert_zen_success(&args, &output);
    assert!(
        tmp.path().join("build").join("app").join("app").exists(),
        "expected executable output to exist"
    );
}
