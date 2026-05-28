use super::emit_json;
use super::write_two_module_project;
use crate::cli_build::frontend_json::golden_support::write_subject;
use crate::cli_build::support::{assert_zen_failure_contains, assert_zen_success};

#[test]
fn emit_json_ast_command_outputs_resolved_module_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = write_two_module_project(&tmp);

    let output = emit_json("ast", &main_path, "resolved module graph");

    assert_zen_success(&["emit-json", "ast", main_path.to_str().unwrap()], &output);

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json ast stdout is json");
    assert_eq!(json["format"], "zen.ast.v0");
    assert_eq!(json["schema_version"], 0);
    assert_eq!(json["semantic_status"], "unchecked");
    assert_eq!(json["entry_module"], 0);
    assert_eq!(json["modules"].as_array().expect("modules array").len(), 2);

    let entry = &json["modules"][0];
    assert_eq!(entry["id"], 0);
    assert_eq!(entry["imports"][0]["local_name"], "add");
    assert_eq!(entry["imports"][0]["source_symbol"], "add");
    assert!(
        entry["program"]["declarations"]
            .as_array()
            .expect("entry declarations")
            .iter()
            .any(|decl| decl["Function"]["name"] == "main"),
        "entry AST should contain main function: {entry}"
    );

    let imported = &json["modules"][1];
    assert_eq!(imported["id"], 1);
    assert!(
        imported["program"]["declarations"]
            .as_array()
            .expect("imported declarations")
            .iter()
            .any(|decl| decl["Function"]["name"] == "add"),
        "imported AST should contain add function: {imported}"
    );
}

#[test]
fn emit_json_ast_marks_semantically_unchecked_sources_that_typed_json_rejects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let source_path = write_subject(
        &tmp,
        "bad_semantics.zen",
        r#"
main = () i32 {
    true
}
"#,
    );

    let ast_output = emit_json("ast", &source_path, "semantically invalid AST");

    assert_zen_success(
        &["emit-json", "ast", source_path.to_str().unwrap()],
        &ast_output,
    );

    let ast_json: serde_json::Value =
        serde_json::from_slice(&ast_output.stdout).expect("emit-json ast stdout is json");
    assert_eq!(ast_json["format"], "zen.ast.v0");
    assert_eq!(ast_json["semantic_status"], "unchecked");

    let typed_output = emit_json("typed", &source_path, "semantically invalid typed JSON");

    assert_zen_failure_contains(
        &["emit-json", "typed", source_path.to_str().unwrap()],
        &typed_output,
        "return type mismatch",
    );
}
