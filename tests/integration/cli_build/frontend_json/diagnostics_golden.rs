use std::path::Path;
use std::process::Command;

#[path = "diagnostics_golden/behavior_association.rs"]
mod behavior_association;
#[path = "diagnostics_golden/generic_methods.rs"]
mod generic_methods;
#[path = "diagnostics_golden/removed_syntax.rs"]
mod removed_syntax;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_diagnostics_golden(
    file_name: &str,
    source: &str,
    golden: &str,
    description: &str,
    expected_diagnostic_count: usize,
    followup_message: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(file_name);
    std::fs::write(&zen_path, source)
        .unwrap_or_else(|err| panic!("write {description} source: {err}"));

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json diagnostics on {description}: {err}"));

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on {description}: stdout={}, stderr={}",
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
        expected_diagnostic_count,
        "{followup_message}: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}
