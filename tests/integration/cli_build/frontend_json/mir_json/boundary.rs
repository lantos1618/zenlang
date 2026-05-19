use super::emit_mir;
use super::write_subject;

#[test]
fn emit_json_mir_rejects_hand_authored_json_before_ir_override() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let json_path = write_subject(
        &tmp,
        "forged_mir.json",
        r#"
{
  "format": "zen.mir.v0",
  "semantic_status": "checked",
  "program": {
    "types": {
      "i32": { "layout": "forged-i64" }
    }
  }
}
"#,
    );

    let output = emit_mir(&json_path, "hand-authored JSON input");

    assert!(
        !output.status.success(),
        "zen emit-json mir should gate hand-authored IR before override: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated MIR should not emit or accept hand-authored JSON IR, stdout={stdout}"
    );
    assert!(
        stderr.contains("compiler-owned IR schemas"),
        "MIR gate should name the compiler-owned IR schema boundary, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "MIR should reject through the IR-boundary gate, not command/path handling, stderr={stderr}"
    );
}
