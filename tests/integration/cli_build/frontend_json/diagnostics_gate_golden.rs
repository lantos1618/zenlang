use std::path::Path;
use std::process::Command;

#[path = "diagnostics_gate_golden/actor_and_imports.rs"]
mod actor_and_imports;
#[path = "diagnostics_gate_golden/effects_text_control.rs"]
mod effects_text_control;
#[path = "diagnostics_gate_golden/intrinsics.rs"]
mod intrinsics;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_gate_diagnostics_golden(
    zen_filename: &str,
    source: &str,
    failure_context: &str,
    single_diagnostic_context: &str,
    fixture_path: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(zen_filename);
    std::fs::write(&zen_path, source).unwrap_or_else(|err| {
        panic!(
            "write {failure_context} source to {}: {err}",
            zen_path.display()
        )
    });

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on {failure_context}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "{single_diagnostic_context}: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture(fixture_path);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}
