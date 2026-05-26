use super::super::super::super::support::{
    assert_zen_success, run_zen_in, write_single_executable_file_read_graph,
};

#[test]
fn build_command_build_zen_accepts_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| .Err { "default" }"#,
        true,
    );
}

#[test]
fn build_command_build_zen_accepts_wildcard_fallback_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| _ { "default" }"#,
        false,
    );
}

#[test]
fn build_command_build_zen_accepts_identifier_fallback_declared_file_read_effects() {
    assert_build_command_build_zen_accepts_declared_file_read_effects(
        r#"| err { "default" }"#,
        false,
    );
}

fn assert_build_command_build_zen_accepts_declared_file_read_effects(
    fallback_arm: &str,
    run_binary: bool,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_single_executable_file_read_graph(&tmp, fallback_arm);

    let output = run_zen_in(&tmp, &["build", "build.zen"]);
    assert_zen_success(&["build", "build.zen"], &output);

    let bin_path = tmp.path().join("build").join("myapp");
    assert!(
        bin_path.exists(),
        "expected {} to exist",
        bin_path.display()
    );

    if run_binary {
        let run = std::process::Command::new(&bin_path)
            .output()
            .expect("run built binary");
        assert!(
            run.status.success(),
            "built binary exited with {}",
            run.status
        );
    }
}
