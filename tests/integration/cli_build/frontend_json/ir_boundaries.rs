use std::process::Command;

#[test]
fn emit_json_typed_rejects_hand_authored_json_before_checked_ir_override() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let json_path = tmp.path().join("forged_typed.json");
    std::fs::write(
        &json_path,
        r#"
{
  "format": "zen.typed.v0",
  "semantic_status": "checked",
  "program": {
    "types": [{ "name": "i32", "kind": "forged-pointer" }],
    "functions": [{ "name": "main", "return_type": "i32" }]
  }
}
"#,
    )
    .expect("write forged typed JSON");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "typed", json_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json typed on hand-authored JSON input");

    assert!(
        !output.status.success(),
        "zen emit-json typed should reject hand-authored checked IR before override: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "typed JSON should not emit or accept hand-authored checked IR, stdout={stdout}"
    );
    assert!(
        stderr.contains("compiler-owned typed JSON"),
        "typed gate should name the compiler-owned typed JSON boundary, stderr={stderr}"
    );
    assert!(
        !stderr.contains("expected function name") && !stderr.contains("unexpected token"),
        "typed JSON should reject before treating JSON as Zen source, stderr={stderr}"
    );
}

#[test]
fn emit_json_hir_rejects_hand_authored_json_before_ir_override() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let json_path = tmp.path().join("forged_hir.json");
    std::fs::write(
        &json_path,
        r#"
{
  "format": "zen.hir.v0",
  "semantic_status": "checked",
  "program": {
    "types": {
      "Forged": {
        "fields": [{ "name": "ptr", "type": "RawPtr<i32>" }]
      }
    }
  }
}
"#,
    )
    .expect("write forged HIR JSON");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", json_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json hir on hand-authored JSON input");

    assert!(
        !output.status.success(),
        "zen emit-json hir should gate hand-authored HIR before override: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated HIR should not emit or accept hand-authored HIR JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("compiler-owned IR schemas"),
        "HIR gate should name the compiler-owned IR schema boundary, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "HIR should reject through the IR-boundary gate, not command/path handling, stderr={stderr}"
    );
}

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
