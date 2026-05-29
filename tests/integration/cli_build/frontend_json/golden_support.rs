use std::path::{Path, PathBuf};

use super::super::support::{assert_zen_failure, assert_zen_success, run_zen, write_file};

pub(super) fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

pub(super) fn stage_golden_path(mode: &str, stem: &str) -> String {
    format!("tests/fixtures/ir_json/{mode}_{stem}.golden.json")
}

/// `BLESS=1 cargo test` regenerates the golden at `golden` with `actual` and
/// returns true (caller skips its assertion). Standard snapshot-update hook.
pub(super) fn maybe_bless(golden: &str, actual: &str) -> bool {
    if std::env::var_os("BLESS").is_none() {
        return false;
    }
    let path = fixture(golden);
    let mut contents = actual.trim().to_string();
    contents.push('\n');
    std::fs::write(&path, contents).unwrap_or_else(|e| panic!("bless {}: {e}", path.display()));
    true
}

pub(super) fn assert_stage_golden(
    mode: &str,
    label: &str,
    source_path: &Path,
    golden: &str,
    description: &str,
) {
    let actual = emit_checked_json(mode, label, source_path, description);
    if maybe_bless(golden, &actual) {
        return;
    }
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
    let zen_path = write_subject(&tmp, filename, source);

    assert_stage_golden(mode, label, &zen_path, golden, description);
}

pub(super) fn write_subject(tmp: &tempfile::TempDir, file_name: &str, source: &str) -> PathBuf {
    write_file(tmp, file_name, source);
    tmp.path().join(file_name)
}

pub(super) fn checked_source_json(
    mode: &str,
    filename: &str,
    source: &str,
    description: &str,
) -> serde_json::Value {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(&tmp, filename, source);
    checked_json(mode, &zen_path, description)
}

pub(super) fn checked_json(mode: &str, source_path: &Path, description: &str) -> serde_json::Value {
    let output = emit_json(mode, source_path);
    assert_zen_success(&["emit-json", mode, source_path.to_str().unwrap()], &output);
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("{mode} {description} stdout is JSON: {err}"))
}

pub(super) fn diagnostics_failure_json(
    filename: &str,
    source: &str,
    description: &str,
) -> serde_json::Value {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(&tmp, filename, source);
    let args = ["emit-json", "diagnostics", zen_path.to_str().unwrap()];
    let output = run_zen(&args);
    assert_zen_failure(&args, &output);
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|err| panic!("diagnostics {description} stdout is JSON: {err}"))
}

pub(super) fn assert_diagnostics_failure_golden(
    filename: &str,
    source: &str,
    _description: &str,
    expected_count: usize,
    count_context: &str,
    golden_stem: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(&tmp, filename, source);

    let args = ["emit-json", "diagnostics", zen_path.to_str().unwrap()];
    let output = run_zen(&args);
    assert_zen_failure(&args, &output);

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        expected_count,
        "{count_context}: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected = read_fixture(&format!(
        "tests/fixtures/ir_json/diagnostics_{golden_stem}.golden.json"
    ));

    assert_eq!(normalized.trim(), expected.trim());
}

pub(super) fn emit_checked_json(
    mode: &str,
    label: &str,
    source_path: &Path,
    description: &str,
) -> String {
    let output = emit_json(mode, source_path);
    assert_zen_success(&["emit-json", mode, source_path.to_str().unwrap()], &output);

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("{label} {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("{label} {description} stdout is JSON: {err}"));
    actual
}

fn emit_json(mode: &str, source_path: &Path) -> std::process::Output {
    run_zen(&["emit-json", mode, source_path.to_str().unwrap()])
}

fn read_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()))
}
