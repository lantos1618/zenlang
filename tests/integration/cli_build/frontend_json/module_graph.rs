use std::process::Command;

#[test]
fn emit_json_ast_command_outputs_resolved_module_graph() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "ast", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json ast");

    assert!(
        output.status.success(),
        "zen emit-json ast failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json ast stdout is json");
    assert_eq!(json["format"], "zen.ast.v0");
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
    let source_path = tmp.path().join("bad_semantics.zen");
    std::fs::write(
        &source_path,
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write semantically invalid module");

    let ast_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "ast", source_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json ast");

    assert!(
        ast_output.status.success(),
        "AST JSON remains a tooling view and should emit parse/resolver output: stdout={}, stderr={}",
        String::from_utf8_lossy(&ast_output.stdout),
        String::from_utf8_lossy(&ast_output.stderr)
    );

    let ast_json: serde_json::Value =
        serde_json::from_slice(&ast_output.stdout).expect("emit-json ast stdout is json");
    assert_eq!(ast_json["format"], "zen.ast.v0");
    assert_eq!(ast_json["semantic_status"], "unchecked");

    let typed_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "typed", source_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json typed");

    assert!(
        !typed_output.status.success(),
        "typed JSON should reject the same semantic error: stdout={}, stderr={}",
        String::from_utf8_lossy(&typed_output.stdout),
        String::from_utf8_lossy(&typed_output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&typed_output.stderr).contains("return type mismatch"),
        "typed JSON should report the semantic mismatch, stderr={}",
        String::from_utf8_lossy(&typed_output.stderr)
    );
}

#[test]
fn emit_json_symbols_command_outputs_module_symbol_tables() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(
        &math_path,
        r#"
pub add = (a: i32, b: i32) i32 {
    a + b
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"
{ add } = math

main = () i32 {
    add(20, 22)
}
"#,
    )
    .expect("write entry module");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "symbols", main_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json symbols");

    assert!(
        output.status.success(),
        "zen emit-json symbols failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json symbols stdout is json");
    assert_eq!(json["format"], "zen.symbols.v0");
    assert_eq!(json["entry_module"], 0);
    assert_eq!(json["modules"].as_array().expect("modules array").len(), 2);

    let entry_symbols = json["modules"][0]["symbols"]
        .as_array()
        .expect("entry symbols array");
    assert!(
        entry_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Value"
                && symbol["name"] == "main"
                && symbol["return_type_name"] == "i32"
        }),
        "entry symbols should contain main value symbol: {json}"
    );
    assert!(
        entry_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Import"
                && symbol["name"] == "add"
                && symbol["import_source"] == "math"
        }),
        "entry symbols should contain add import symbol: {json}"
    );

    let imported_symbols = json["modules"][1]["symbols"]
        .as_array()
        .expect("imported symbols array");
    assert!(
        imported_symbols.iter().any(|symbol| {
            symbol["namespace"] == "Value"
                && symbol["name"] == "add"
                && symbol["is_public"] == true
                && symbol["parameter_count"] == 2
                && symbol["return_type_name"] == "i32"
        }),
        "imported symbols should contain public add signature: {json}"
    );
}
