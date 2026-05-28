use super::super::support::{assert_zen_failure_contains, assert_zen_success, run_zen, write_file};
mod invalid;
mod valid;

fn emit_target_yaml(yaml: &str, description: &str) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = "target.yaml";
    write_file(&tmp, yaml_path, yaml);

    let _ = description;
    run_zen(&[
        "emit-json",
        "target-yaml",
        tmp.path().join(yaml_path).to_str().unwrap(),
    ])
}

fn assert_valid_target_yaml(yaml: &str, description: &str) -> serde_json::Value {
    let output = emit_target_yaml(yaml, description);

    assert_zen_success(&["emit-json", "target-yaml", "target.yaml"], &output);

    serde_json::from_slice(&output.stdout).expect("target-yaml stdout is json")
}

fn assert_invalid_target_yaml(yaml: &str, description: &str, stderr_substring: &str) {
    let output = emit_target_yaml(yaml, description);

    assert_zen_failure_contains(
        &["emit-json", "target-yaml", "target.yaml"],
        &output,
        stderr_substring,
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "invalid target-yaml should not emit target JSON, stdout={stdout}"
    );
}
