use std::path::Path;
use std::process::Command;

#[path = "diagnostics_behavior_association_golden/relationship_arity.rs"]
mod relationship_arity;
#[path = "diagnostics_behavior_association_golden/requires.rs"]
mod requires;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_behavior_association_diagnostics_golden(
    source: &str,
    filename: &str,
    golden: &str,
    description: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(filename);
    std::fs::write(&zen_path, source)
        .unwrap_or_else(|err| panic!("write {description} source: {err}"));

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json diagnostics for {description}: {err}"));

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
        1,
        "{description} should emit one diagnostic: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}
