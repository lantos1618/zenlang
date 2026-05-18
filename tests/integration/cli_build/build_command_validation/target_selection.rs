#[test]
fn build_command_build_zen_rejects_graph_without_executable_targets() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen" })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
    std::fs::write(
        tmp.path().join("test.zen"),
        r#"
main = () i32 {
    0
}
"#,
    )
    .expect("write test.zen");

    let output = super::super::support::run_zen_in(&tmp, &args);

    super::super::support::assert_zen_failure_contains(
        &args,
        &output,
        "build graph execution requires at least one executable target",
    );
    assert!(
        !tmp.path().join("build").exists(),
        "build command should not create outputs for a test-only graph"
    );
}
