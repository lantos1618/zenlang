use std::process::Command;

#[path = "diagnostics_json/behavior_gates.rs"]
mod behavior_gates;
#[path = "diagnostics_json/core.rs"]
mod core;
#[path = "diagnostics_json/fixes.rs"]
mod fixes;

fn emit_diagnostics_json(source: &str, filename: &str, description: &str) -> serde_json::Value {
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

    serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json")
}
