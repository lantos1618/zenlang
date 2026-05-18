use crate::support::*;
use std::process::Command;

#[path = "frontend_json/diagnostics_behavior_association_golden.rs"]
mod diagnostics_behavior_association_golden;
#[path = "frontend_json/diagnostics_generic_arity_golden.rs"]
mod diagnostics_generic_arity_golden;
#[path = "frontend_json/diagnostics_golden.rs"]
mod diagnostics_golden;
#[path = "frontend_json/diagnostics_json.rs"]
mod diagnostics_json;
#[path = "frontend_json/hir_golden.rs"]
mod hir_golden;
#[path = "frontend_json/ir_boundaries.rs"]
mod ir_boundaries;
#[path = "frontend_json/layout_golden.rs"]
mod layout_golden;
#[path = "frontend_json/layout_json.rs"]
mod layout_json;
#[path = "frontend_json/mir_golden.rs"]
mod mir_golden;
#[path = "frontend_json/module_graph.rs"]
mod module_graph;
#[path = "frontend_json/module_graph_golden.rs"]
mod module_graph_golden;
#[path = "frontend_json/target_yaml.rs"]
mod target_yaml;
#[path = "frontend_json/target_yaml_golden.rs"]
mod target_yaml_golden;
#[path = "frontend_json/typed_golden.rs"]
mod typed_golden;

#[test]
fn emit_json_usage_lists_supported_and_gated_modes() {
    let missing_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("emit-json")
        .output()
        .expect("run zen emit-json without mode");
    assert!(
        !missing_mode_output.status.success(),
        "zen emit-json without mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&missing_mode_output.stdout),
        String::from_utf8_lossy(&missing_mode_output.stderr)
    );

    let unknown_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "unknown",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json unknown mode");
    assert!(
        !unknown_mode_output.status.success(),
        "zen emit-json unknown mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&unknown_mode_output.stdout),
        String::from_utf8_lossy(&unknown_mode_output.stderr)
    );

    let expected_usage =
        "Usage: zen emit-json <ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml> <file.zen>";
    for output in [&missing_mode_output, &unknown_mode_output] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_usage),
            "emit-json usage should list supported and gated modes, stderr={stderr}"
        );
    }
}

#[test]
fn emit_json_typed_command_outputs_checked_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed");

    assert!(
        output.status.success(),
        "zen emit-json typed failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json typed stdout is json");
    assert_eq!(json["format"], "zen.typed.v0");
    assert_eq!(json["semantic_status"], "checked");

    let functions = json["program"]["functions"]
        .as_array()
        .expect("typed functions array");
    assert!(
        functions
            .iter()
            .any(|function| function["name"] == "Box.get_i32"),
        "typed JSON should contain specialized generic method: {json}"
    );
    assert!(
        functions.iter().any(|function| function["name"] == "main"),
        "typed JSON should contain main function: {json}"
    );

    let types = json["program"]["types"]
        .as_array()
        .expect("typed types array");
    assert!(
        types.iter().any(|ty| ty["name"] == "Box_i32"),
        "typed JSON should contain specialized generic type: {json}"
    );

    let serialized = String::from_utf8(output.stdout).expect("typed JSON is utf-8");
    assert!(!serialized.contains("Box_T"));
    assert!(!serialized.contains("T Box_get"));
}

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

#[test]
fn emit_json_hir_outputs_checked_declaration_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("hir_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Point: {
    x: i32,
    label: StaticString
}

main = () i32 { 0 }
"#,
    )
    .expect("write HIR subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json hir on program input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("HIR stdout is json");
    assert_eq!(json["format"], "zen.hir.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");

    let types = json["declarations"]["types"]
        .as_array()
        .expect("HIR types array");
    let point = types
        .iter()
        .find(|ty| ty["name"] == "Point")
        .expect("Point type in HIR");
    assert_eq!(point["kind"], "struct");
    assert_eq!(point["fields"][0]["name"], "x");
    assert_eq!(point["fields"][0]["type"], "i32");
    assert_eq!(point["fields"][1]["name"], "label");
    assert_eq!(point["fields"][1]["type"], "StaticString");

    let functions = json["declarations"]["functions"]
        .as_array()
        .expect("HIR functions array");
    let main = functions
        .iter()
        .find(|function| function["name"] == "main")
        .expect("main function in HIR");
    assert_eq!(main["return_type"], "i32");
    assert!(main["params"].as_array().expect("main params").is_empty());
}

#[test]
fn emit_json_hir_outputs_enum_function_and_global_declarations() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("hir_declarations_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Pair: {
    left: i32,
    right: i32,
}

MaybePair:
    None,
    Some(Pair)

threshold ::= 10

choose = (candidate: Pair, enabled: bool) MaybePair {
    enabled ?
        | true { MaybePair.Some(candidate) }
        | false { MaybePair.None }
}

main = () i32 { 0 }
"#,
    )
    .expect("write HIR declarations subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json hir on declaration-rich program input");

    assert!(
        output.status.success(),
        "zen emit-json hir should emit checked declaration-rich HIR JSON: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("declaration-rich HIR stdout is json");
    assert_eq!(json["format"], "zen.hir.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "checked");

    let types = json["declarations"]["types"]
        .as_array()
        .expect("HIR types array");
    let maybe = types
        .iter()
        .find(|ty| ty["name"] == "MaybePair")
        .expect("MaybePair enum in HIR");
    assert_eq!(maybe["kind"], "enum");
    let variants = maybe["variants"].as_array().expect("MaybePair variants");
    assert_eq!(variants[0]["name"], "None");
    assert_eq!(variants[0]["tag"], 0);
    assert!(variants[0]["payload"]
        .as_array()
        .expect("None payload")
        .is_empty());
    assert_eq!(variants[1]["name"], "Some");
    assert_eq!(variants[1]["tag"], 1);
    assert_eq!(variants[1]["payload"][0]["type"], "Pair");

    let functions = json["declarations"]["functions"]
        .as_array()
        .expect("HIR functions array");
    let choose = functions
        .iter()
        .find(|function| function["name"] == "choose")
        .expect("choose function in HIR");
    assert_eq!(choose["return_type"], "MaybePair");
    assert_eq!(choose["params"][0]["name"], "candidate");
    assert_eq!(choose["params"][0]["type"], "Pair");
    assert_eq!(choose["params"][1]["name"], "enabled");
    assert_eq!(choose["params"][1]["type"], "bool");

    let globals = json["declarations"]["globals"]
        .as_array()
        .expect("HIR globals array");
    let threshold = globals
        .iter()
        .find(|global| global["name"] == "threshold")
        .expect("threshold global in HIR");
    assert_eq!(threshold["type"], "i32");
    assert_eq!(threshold["mutable"], true);
}
