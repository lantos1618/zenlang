use std::process::Command;

#[test]
fn emit_json_layout_rejects_hand_authored_json_before_layout_override() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let json_path = tmp.path().join("forged_layout.json");
    std::fs::write(
        &json_path,
        r#"
{
  "format": "zen.layout.v0",
  "semantic_status": "checked",
  "layouts": {
    "StaticString": {
      "size": 1,
      "alignment": 1
    }
  }
}
"#,
    )
    .expect("write forged layout JSON");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "layout", json_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json layout on hand-authored JSON input");

    assert!(
        !output.status.success(),
        "zen emit-json layout should gate hand-authored layout IR before override: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated layout should not emit or accept hand-authored layout JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("compiler-owned layout schemas"),
        "layout gate should name the compiler-owned layout schema boundary, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "layout should reject through the IR-boundary gate, not command/path handling, stderr={stderr}"
    );
}
