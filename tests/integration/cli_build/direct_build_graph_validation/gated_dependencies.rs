use std::process::Command;

#[test]
fn direct_file_command_build_zen_rejects_gated_test_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen" })
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["unit"],
    })
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
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        !output.status.success(),
        "zen build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `app` depends on gated test target `unit`"),
        "expected gated test dependency diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "direct build.zen command should not start after gated dependency validation fails"
    );
}

#[test]
fn direct_file_command_build_zen_rejects_transitive_gated_test_dependencies() {
    let tmp = super::super::support::transitive_gated_test_dependency_graph();

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("build.zen")
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen");

    assert!(
        !output.status.success(),
        "zen build.zen unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("build graph target `core` depends on gated test target `unit`"),
        "expected transitive gated test dependency diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "direct build.zen command should not start after transitive gated dependency validation fails"
    );
}
