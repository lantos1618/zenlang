use super::checked_mir_json;
use super::write_subject;

#[test]
fn emit_json_mir_outputs_match_arm_schema() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = write_subject(
        &tmp,
        "mir_match_subject.zen",
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
    );

    let json = checked_mir_json(&zen_path, "match program input");
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
