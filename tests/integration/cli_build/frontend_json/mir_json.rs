use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "mir_json/boundary.rs"]
mod boundary;
#[path = "mir_json/match_schema.rs"]
mod match_schema;
#[path = "mir_json/minimal.rs"]
mod minimal;

fn write_subject(tmp: &tempfile::TempDir, file_name: &str, source: &str) -> PathBuf {
    let zen_path = tmp.path().join(file_name);
    std::fs::write(&zen_path, source).unwrap_or_else(|err| panic!("write {file_name}: {err}"));
    zen_path
}

fn emit_mir(path: &Path, description: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", path.to_str().unwrap()])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json mir on {description}: {err}"))
}

fn checked_mir_json(path: &Path, description: &str) -> serde_json::Value {
    let output = emit_mir(path, description);

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked MIR JSON for {description}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("MIR stdout is json")
}
