use std::path::{Path, PathBuf};

use super::super::support::{assert_zen_success, run_zen};
use super::golden_support::write_subject as write_json_subject;
mod boundary;
mod match_schema;
mod minimal;

fn write_subject(tmp: &tempfile::TempDir, file_name: &str, source: &str) -> PathBuf {
    write_json_subject(tmp, file_name, source)
}

fn emit_mir(path: &Path, description: &str) -> std::process::Output {
    let _ = description;
    run_zen(&["emit-json", "mir", path.to_str().unwrap()])
}

fn checked_mir_json(path: &Path, description: &str) -> serde_json::Value {
    let output = emit_mir(path, description);

    assert_zen_success(&["emit-json", "mir", path.to_str().unwrap()], &output);

    serde_json::from_slice(&output.stdout).expect("MIR stdout is json")
}
