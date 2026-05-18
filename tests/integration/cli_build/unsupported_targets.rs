use std::process::Command;

#[path = "unsupported_targets/fields.rs"]
mod fields;
#[path = "unsupported_targets/kinds.rs"]
mod kinds;

fn all_build_zen_command_args() -> [&'static [&'static str]; 6] {
    [
        &["build", "build.zen"][..],
        &["build.zen"][..],
        &["check", "build.zen"][..],
        &["test", "build.zen"][..],
        &["emit", "build.zen"][..],
        &["build-graph", "build.zen"][..],
    ]
}

fn run_build_zen_command(
    args: &[&str],
    build_source: String,
) -> (tempfile::TempDir, std::process::Output) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(tmp.path().join("build.zen"), build_source).expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(args)
        .current_dir(tmp.path())
        .output()
        .expect("run zen build.zen command");

    (tmp, output)
}

fn assert_rejected_without_outputs(
    tmp: &tempfile::TempDir,
    output: &std::process::Output,
    args: &[&str],
    expected: String,
    reason: &str,
) {
    assert!(
        !output.status.success(),
        "zen {args:?} unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(&expected),
        "expected {reason} diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !tmp.path().join("build").exists(),
        "zen {args:?} should reject {reason} before creating build outputs"
    );
}
