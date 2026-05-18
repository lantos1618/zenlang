use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_build_graph_project_schema_matches_golden() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", "examples/project/build.zen"])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        output.status.success(),
        "emit-json build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("build graph json");

    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_project.golden.json",
    ))
    .expect("read build graph golden fixture");
    let actual = String::from_utf8(output.stdout).expect("utf8 build graph json");

    assert_eq!(actual.trim(), expected.trim());
}
