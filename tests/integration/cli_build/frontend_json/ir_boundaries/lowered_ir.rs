use super::assert_rejects_hand_authored_json;

#[test]
fn emit_json_hir_rejects_hand_authored_json_before_ir_override() {
    assert_rejects_hand_authored_json(
        "hir",
        "forged_hir.json",
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
        "HIR",
        "compiler-owned IR schemas",
        &["unknown command", "No such file"],
    );
}

#[test]
fn emit_json_layout_rejects_hand_authored_json_before_layout_override() {
    assert_rejects_hand_authored_json(
        "layout",
        "forged_layout.json",
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
        "layout",
        "compiler-owned layout schemas",
        &["unknown command", "No such file"],
    );
}
