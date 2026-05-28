use super::super::support::{
    assert_stderr_lacks, assert_zen_failure_contains, run_zen, write_file,
};
mod compiler_json;
mod lowered_ir;

fn assert_rejects_hand_authored_json(
    mode: &str,
    filename: &str,
    forged_json: &str,
    description: &str,
    required_stderr: &str,
    forbidden_stderr: &[&str],
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    write_file(&tmp, filename, forged_json);

    let json_path = tmp.path().join(filename);
    let args = ["emit-json", mode, json_path.to_str().unwrap()];
    let output = run_zen(&args);
    assert_zen_failure_contains(&args, &output, required_stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "{description} JSON should not emit or accept hand-authored IR, stdout={stdout}"
    );
    assert_stderr_lacks(&output, forbidden_stderr, description);
}
