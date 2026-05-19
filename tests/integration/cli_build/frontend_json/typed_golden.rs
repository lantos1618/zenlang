use crate::support::*;
use std::path::Path;
use std::process::Command;

#[path = "typed_golden/behavior_bounds.rs"]
mod behavior_bounds;
#[path = "typed_golden/generic_enums.rs"]
mod generic_enums;
#[path = "typed_golden/generic_values.rs"]
mod generic_values;
#[path = "typed_golden/methods.rs"]
mod methods;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_typed_golden(source: &str, golden: &str, description: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join(source).to_str().unwrap(),
        ])
        .output()
        .unwrap_or_else(|err| panic!("run zen emit-json typed on {description}: {err}"));

    assert!(
        output.status.success(),
        "zen emit-json typed should emit checked {description} JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout)
        .unwrap_or_else(|err| panic!("{description} stdout is UTF-8: {err}"));
    serde_json::from_str::<serde_json::Value>(&actual)
        .unwrap_or_else(|err| panic!("{description} stdout is JSON: {err}"));
    let expected_path = fixture(golden);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(actual.trim(), expected.trim());
}
