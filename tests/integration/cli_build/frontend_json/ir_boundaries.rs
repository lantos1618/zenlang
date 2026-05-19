use std::process::Command;

#[path = "ir_boundaries/compiler_json.rs"]
mod compiler_json;
#[path = "ir_boundaries/lowered_ir.rs"]
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
    let json_path = tmp.path().join(filename);
    std::fs::write(&json_path, forged_json)
        .unwrap_or_else(|err| panic!("write forged {description} JSON: {err}"));

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", mode, json_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| {
            panic!("run zen emit-json {mode} on hand-authored JSON input: {err}")
        });

    assert!(
        !output.status.success(),
        "zen emit-json {mode} should reject hand-authored {description} before override: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "{description} JSON should not emit or accept hand-authored IR, stdout={stdout}"
    );
    assert!(
        stderr.contains(required_stderr),
        "{description} gate should name the compiler-owned boundary `{required_stderr}`, stderr={stderr}"
    );
    for forbidden in forbidden_stderr {
        assert!(
            !stderr.contains(forbidden),
            "{description} JSON should reject before `{forbidden}`, stderr={stderr}"
        );
    }
}
