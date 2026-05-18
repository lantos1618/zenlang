#[test]
fn build_command_build_zen_rejects_undeclared_host_effects_before_library_typechecking() {
    let args = ["build", "build.zen"];
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    b.add(Library { name: "core", exports: ["lib.zen"] })
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
    std::fs::write(
        tmp.path().join("lib.zen"),
        r#"
value = () i32 {
    true
}
"#,
    )
    .expect("write lib.zen");

    let output = super::super::support::run_zen_in(&tmp, &args);

    super::super::support::assert_zen_failure_contains(
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
        "build command should not start after graph validation fails"
    );
}
