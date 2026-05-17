use std::process::Command;

#[test]
fn build_command_build_zen_rejects_library_only_graph_execution() {
    assert_library_only_graph_is_rejected(
        &["build", "build.zen"],
        "build graph execution requires at least one executable target",
    );
}

#[test]
fn direct_file_command_build_zen_rejects_library_only_graph_execution() {
    assert_library_only_graph_is_rejected(
        &["build.zen"],
        "build graph execution requires at least one executable target",
    );
}

#[test]
fn build_graph_command_rejects_library_only_graph_execution() {
    assert_library_only_graph_is_rejected(
        &["build-graph", "build.zen"],
        "build graph execution requires at least one executable target",
    );
}

#[test]
fn emit_command_build_zen_rejects_library_only_graph_execution() {
    assert_library_only_graph_is_rejected(
        &["emit", "build.zen"],
        "build graph C emission supports exactly one target, found 0",
    );
}

#[test]
fn test_command_build_zen_rejects_library_only_graph_execution() {
    assert_library_only_graph_is_rejected(
        &["test", "build.zen"],
        "build graph test execution requires at least one test target",
    );
}

fn assert_library_only_graph_is_rejected(args: &[&str], expected_stderr: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["lib.zen"] })
    .Ok(b.config())
}
"#,
    )
    .expect("write build.zen");
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
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen command");

    assert!(
        !output.status.success(),
        "zen {args:?} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_stderr),
        "expected library-only graph diagnostic `{expected_stderr}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should not create build outputs for a library-only graph"
    );
}
