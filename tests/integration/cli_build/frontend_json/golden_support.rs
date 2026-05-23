use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

pub(super) fn assert_stage_golden(
    mode: &str,
    label: &str,
    source_path: &Path,
    golden: &str,
    description: &str,
) {
    let actual = emit_checked_json(mode, label, source_path, description);
    let expected = read_fixture(golden);

    assert_eq!(actual.trim(), expected.trim());
}

pub(super) fn assert_stage_source_golden(
    mode: &str,
    label: &str,
    source: &str,
    filename: &str,
    golden: &str,
    description: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(filename);
    std::fs::write(&zen_path, source)
        .unwrap_or_else(|err| panic!("write {label} {description} subject: {err}"));

    assert_stage_golden(mode, label, &zen_path, golden, description);
}

fn emit_checked_json(mode: &str, label: &str, source_path: &Path, description: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", mode, source_path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json {mode} on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json {mode} should emit checked {description} {label} JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("{label} {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("{label} {description} stdout is JSON: {err}"));
    actual
}

fn read_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()))
}
