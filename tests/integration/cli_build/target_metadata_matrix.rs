use std::process::Command;

#[path = "target_metadata_matrix/deterministic_body.rs"]
mod deterministic_body;
#[path = "target_metadata_matrix/executable.rs"]
mod executable;
#[path = "target_metadata_matrix/library.rs"]
mod library;
#[path = "target_metadata_matrix/test_target.rs"]
mod test_target;

fn assert_build_zen_commands_reject_build_graph_metadata(
    build_source: &str,
    expected_diagnostic: &str,
) {
    for args in [
        &["build", "build.zen"][..],
        &["build.zen"][..],
        &["check", "build.zen"][..],
        &["test", "build.zen"][..],
        &["emit", "build.zen"][..],
        &["build-graph", "build.zen"][..],
    ] {
        assert_build_zen_command_rejects_library_target_metadata(
            args,
            build_source,
            expected_diagnostic,
        );
    }
}

fn assert_build_zen_command_rejects_library_target_metadata(
    args: &[&str],
    build_source: &str,
    expected_diagnostic: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen command");

    assert!(
        !output.status.success(),
        "zen {args:?} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected_diagnostic),
        "expected library target metadata diagnostic `{expected_diagnostic}`, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should reject library target metadata before creating build outputs"
    );
}
