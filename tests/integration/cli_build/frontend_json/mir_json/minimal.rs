use super::checked_mir_json;
use super::write_subject;

#[test]
fn emit_json_mir_outputs_checked_minimal_function_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(
        &tmp,
        "mir_subject.zen",
        r#"
main = () i32 {
    value = 40 + 2
    value
}
"#,
    );

    let json = checked_mir_json(&zen_path, "program input");
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
