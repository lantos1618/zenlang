use super::super::super::super::support::{
    assert_zen_success, run_zen_in, write_multiple_executable_file_read_graph,
};

#[test]
fn build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets() {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| .Err { "default" }"#,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| _ { "default" }"#,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects_for_multiple_targets(
) {
    assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
        r#"| err { "default" }"#,
    );
}

fn assert_build_command_build_zen_accepts_declared_file_read_effects_for_multiple_targets(
    fallback_arm: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_multiple_executable_file_read_graph(&tmp, fallback_arm);

    let output = run_zen_in(&tmp, &["build", "build.zen"]);
    assert_zen_success(&["build", "build.zen"], &output);

    for bin_path in [
        tmp.path().join("build").join("app").join("app"),
        tmp.path().join("build").join("tool").join("tool"),
    ] {
        assert!(
            bin_path.exists(),
            "expected {} to exist",
            bin_path.display()
        );
    }
}
