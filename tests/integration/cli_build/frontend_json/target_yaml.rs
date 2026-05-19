use std::process::Command;

#[path = "target_yaml/invalid.rs"]
mod invalid;
#[path = "target_yaml/valid.rs"]
mod valid;

fn emit_target_yaml(yaml: &str, description: &str) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(&yaml_path, yaml)
        .unwrap_or_else(|err| panic!("write {description} target yaml: {err}"));

    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json target-yaml on {description}: {err}"))
}

fn assert_valid_target_yaml(yaml: &str, description: &str) -> serde_json::Value {
    let output = emit_target_yaml(yaml, description);

    assert!(
        output.status.success(),
        "zen emit-json target-yaml should validate {description}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("target-yaml stdout is json")
}

fn assert_invalid_target_yaml(yaml: &str, description: &str, stderr_substring: &str) {
    let output = emit_target_yaml(yaml, description);

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should reject {description}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "invalid target-yaml should not emit target JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains(stderr_substring),
        "expected target YAML diagnostic `{stderr_substring}`, stderr={stderr}"
    );
}
