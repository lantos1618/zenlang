use std::process::Command;

#[path = "runner_command/multiple_targets.rs"]
mod multiple_targets;
#[path = "runner_command/single_target.rs"]
mod single_target;
#[path = "runner_command/unselected_targets.rs"]
mod unselected_targets;

fn run_test_command_build_zen(
    build_source: String,
    files: &[(&str, &str)],
) -> (tempfile::TempDir, std::process::Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");
    for (path, source) in files {
        std::fs::write(tmp.path().join(path), source).expect("write source file");
    }

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["test", "build.zen"])
        .current_dir(tmp.path())
        .output()
        .expect("run zen test build.zen");
    (tmp, output)
}

fn assert_test_command_succeeded(output: &std::process::Output, case_name: &str) {
    assert!(
        output.status.success(),
        "{case_name}: zen test build.zen failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn passing_main_source(value: i32) -> String {
    format!(
        r#"
main = () i32 {{
    {value}
}}
"#,
    )
}
