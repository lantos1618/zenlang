use std::process::Command;

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
