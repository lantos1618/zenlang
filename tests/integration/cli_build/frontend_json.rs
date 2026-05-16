use crate::support::*;
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

#[test]
fn emit_json_diagnostics_command_outputs_machine_readable_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_type.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write bad source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on errors: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    assert_eq!(json["format"], "zen.diagnostics.v0");
    assert_eq!(json["files"].as_array().expect("files array").len(), 1);

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["code"], "E3030");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let span = &diagnostic["span"];
    assert!(span["path"]
        .as_str()
        .expect("span path")
        .ends_with("bad_type.zen"));
    assert_eq!(span["line"], 3);
    assert_eq!(span["column"], 5);
}
