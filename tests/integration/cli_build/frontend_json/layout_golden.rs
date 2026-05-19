use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "layout_golden/generic.rs"]
mod generic;
#[path = "layout_golden/subject.rs"]
mod subject;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn write_subject(tmp: &tempfile::TempDir, file_name: &str, source: &str) -> PathBuf {
    let zen_path = tmp.path().join(file_name);
    std::fs::write(&zen_path, source).unwrap_or_else(|err| panic!("write {file_name}: {err}"));
    zen_path
}

fn emit_layout(path: &Path, description: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "layout", path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json layout on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json layout should emit checked layout JSON for {description}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("layout {description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("layout {description} stdout is JSON: {err}"));
    actual
}

fn expected_fixture(path: &str) -> String {
    let expected_path = fixture(path);
    std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()))
}

fn assert_layout_matches_fixture(source_path: &Path, description: &str, fixture_path: &str) {
    let actual = emit_layout(source_path, description);
    let expected = expected_fixture(fixture_path);

    assert_eq!(actual.trim(), expected.trim());
}
