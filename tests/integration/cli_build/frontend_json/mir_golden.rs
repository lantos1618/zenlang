use std::path::Path;
use std::process::Command;

#[path = "mir_golden/basic.rs"]
mod basic;
#[path = "mir_golden/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "mir_golden/generic_enums.rs"]
mod generic_enums;
#[path = "mir_golden/generic_methods.rs"]
mod generic_methods;
#[path = "mir_golden/generic_values.rs"]
mod generic_values;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_mir_golden(source: &str, golden: &str, description: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", fixture(source).to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json mir on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked {description} MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("MIR {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("MIR {description} stdout is JSON: {err}"));
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}

fn assert_mir_source_golden(source: &str, filename: &str, golden: &str, description: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(filename);
    std::fs::write(&zen_path, source)
        .unwrap_or_else(|err| panic!("write MIR {description} subject: {err}"));

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json mir on {description} input: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked {description} MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("MIR {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("MIR {description} stdout is JSON: {err}"));
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
