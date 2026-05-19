use super::emit_json;
use super::write_two_module_project;

#[test]
fn emit_json_symbols_command_outputs_module_symbol_tables() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let main_path = write_two_module_project(&tmp);

    let output = emit_json("symbols", &main_path, "module symbol tables");

    assert!(
        output.status.success(),
        "zen emit-json symbols failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json symbols stdout is json");
    assert_eq!(json["format"], "zen.symbols.v0");
    assert_eq!(json["semantic_status"], "resolved");
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
