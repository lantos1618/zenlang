use std::process::Command;

#[test]
fn emit_json_mir_outputs_checked_minimal_function_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("mir_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    value = 40 + 2
    value
}
"#,
    )
    .expect("write MIR subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked minimal MIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("MIR stdout is json");
    assert_eq!(json["format"], "zen.mir.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");
    assert_eq!(json["lowering_status"], "minimal");

    let functions = json["functions"].as_array().expect("MIR functions array");
    let main = functions
        .iter()
        .find(|function| function["name"] == "main")
        .expect("main function in MIR");
    assert_eq!(main["return_type"], "i32");

    let entry = &main["blocks"][0];
    assert_eq!(entry["label"], "entry");
    assert_eq!(entry["statements"][0]["kind"], "let");
    assert_eq!(entry["statements"][0]["name"], "value");
    assert_eq!(entry["statements"][0]["type"], "i32");
    assert_eq!(entry["statements"][0]["value"]["kind"], "binary");
    assert_eq!(entry["statements"][0]["value"]["op"], "+");
    assert_eq!(entry["terminator"]["kind"], "return");
    assert_eq!(entry["terminator"]["value"]["kind"], "local");
    assert_eq!(entry["terminator"]["value"]["name"], "value");
    assert_eq!(entry["terminator"]["value"]["type"], "i32");
}

#[test]
fn emit_json_mir_outputs_match_arm_schema() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("mir_match_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Choice:
    Empty,
    Value(i32)

score = (choice: Choice) i32 {
    choice ?
        | Empty { 0 }
        | Value(n) { n }
}

main = () i32 {
    score(Choice.Value(42))
}
"#,
    )
    .expect("write MIR match subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on match program input");

    assert!(
        output.status.success(),
        "zen emit-json mir should emit checked MIR match JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("MIR match stdout is json");
    assert_eq!(json["format"], "zen.mir.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");

    let functions = json["functions"].as_array().expect("MIR functions array");
    let score = functions
        .iter()
        .find(|function| function["name"] == "score")
        .expect("score function in MIR");
    let entry = &score["blocks"][0];
    let terminator_value = &entry["terminator"]["value"];
    assert_eq!(terminator_value["kind"], "match");
    assert_eq!(terminator_value["match_kind"], "enum");
    assert_eq!(terminator_value["target"]["kind"], "local");
    assert_eq!(terminator_value["target"]["name"], "choice");

    let arms = terminator_value["arms"].as_array().expect("MIR match arms");
    assert_eq!(arms[0]["pattern"]["kind"], "enum_variant");
    assert_eq!(arms[0]["pattern"]["name"], "Choice.Empty");
    assert!(arms[0]["pattern"]["bindings"]
        .as_array()
        .expect("Empty bindings")
        .is_empty());
    assert_eq!(arms[0]["body"]["terminator"]["value"]["kind"], "block");
    assert_eq!(
        arms[0]["body"]["terminator"]["value"]["value"]["result"]["kind"],
        "int"
    );
    assert_eq!(
        arms[0]["body"]["terminator"]["value"]["value"]["result"]["value"],
        0
    );

    assert_eq!(arms[1]["pattern"]["kind"], "enum_variant");
    assert_eq!(arms[1]["pattern"]["name"], "Choice.Value");
    assert_eq!(arms[1]["pattern"]["bindings"][0]["name"], "n");
    assert_eq!(arms[1]["pattern"]["bindings"][0]["type"], "i32");
    assert_eq!(arms[1]["body"]["terminator"]["value"]["kind"], "block");
    assert_eq!(
        arms[1]["body"]["terminator"]["value"]["value"]["result"]["kind"],
        "local"
    );
    assert_eq!(
        arms[1]["body"]["terminator"]["value"]["value"]["result"]["name"],
        "n"
    );
}

#[test]
fn emit_json_mir_rejects_hand_authored_json_before_ir_override() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let json_path = tmp.path().join("forged_mir.json");
    std::fs::write(
        &json_path,
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
    )
    .expect("write forged MIR JSON");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", json_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on hand-authored JSON input");

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
