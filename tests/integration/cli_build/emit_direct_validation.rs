use std::process::Command;

#[path = "emit_direct_validation/gated_dependencies.rs"]
mod gated_dependencies;
#[path = "emit_direct_validation/graph_only_libraries.rs"]
mod graph_only_libraries;
#[path = "emit_direct_validation/target_selection.rs"]
mod target_selection;

#[test]
fn emit_command_build_zen_ignores_unrelated_gated_test_source_errors() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen emit build.zen");

    assert!(
        output.status.success(),
        "zen emit build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let c_source = String::from_utf8_lossy(&output.stdout);
    assert!(
        c_source.contains("int32_t zen_main(void)"),
        "expected target C source, stdout={c_source}"
    );
    assert!(
        !tmp.path().join("build").join("app").exists(),
        "zen emit build.zen should not compile the target binary"
    );
}
