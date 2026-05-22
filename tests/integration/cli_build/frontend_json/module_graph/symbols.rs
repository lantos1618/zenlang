use super::emit_json;
use super::write_two_module_project;
use std::path::Path;

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

#[test]
fn emit_json_symbols_reports_multi_file_generic_method_nested_result_surface() {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/zen/multi_file_type_method_nested_result_dependency/main.zen");

    let output = emit_json(
        "symbols",
        &source_path,
        "multi-file generic method nested Result symbols",
    );

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

    let modules = json["modules"].as_array().expect("modules array");
    assert_eq!(
        modules.len(),
        3,
        "multi-file fixture should report main, types, and model modules: {json}"
    );

    let main = module_by_file(modules, "main.zen");
    assert_symbol(main, "Import", "Box", |symbol| {
        symbol["import_source"] == "model"
    });
    assert_symbol(main, "Import", "Option", |symbol| {
        symbol["import_source"] == "types"
    });
    assert_symbol(main, "Import", "Result", |symbol| {
        symbol["import_source"] == "types"
    });

    let types = module_by_file(modules, "types.zen");
    assert_symbol(types, "Type", "Option", |symbol| {
        string_array_eq(&symbol["type_parameter_names"], &["T"])
            && string_array_eq(&symbol["variant_names"], &["None", "Some"])
    });
    assert_symbol(types, "Type", "Result", |symbol| {
        string_array_eq(&symbol["type_parameter_names"], &["T", "E"])
            && string_array_eq(&symbol["variant_names"], &["Ok", "Err"])
    });

    let model = module_by_file(modules, "model.zen");
    assert_symbol(model, "Type", "Box", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T"])
            && field_type_array_eq(&symbol["field_type_names"], &[("value", "T")])
    });
    assert_symbol(model, "Value", "Box.wrap_result", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Box<T>"])
            && symbol["return_type_name"] == "Result<Option<T>, StaticString>"
    });
    assert_symbol(model, "Value", "unwrap_result", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Result<T, E>", "T"])
            && symbol["return_type_name"] == "T"
    });
}

fn module_by_file<'a>(modules: &'a [serde_json::Value], file_name: &str) -> &'a serde_json::Value {
    modules
        .iter()
        .find(|module| {
            module["canonical_path"]
                .as_str()
                .and_then(|path| Path::new(path).file_name())
                .is_some_and(|name| name.to_string_lossy() == file_name)
        })
        .unwrap_or_else(|| panic!("missing module for {file_name}"))
}

fn assert_symbol(
    module: &serde_json::Value,
    namespace: &str,
    name: &str,
    extra: impl Fn(&serde_json::Value) -> bool,
) {
    let symbols = module["symbols"].as_array().expect("module symbols array");
    assert!(
        symbols.iter().any(|symbol| {
            symbol["namespace"] == namespace && symbol["name"] == name && extra(symbol)
        }),
        "missing {namespace} symbol {name} in module {}",
        module["canonical_path"]
    );
}

fn string_array_eq(value: &serde_json::Value, expected: &[&str]) -> bool {
    value.as_array().is_some_and(|actual| {
        actual
            .iter()
            .map(serde_json::Value::as_str)
            .collect::<Option<Vec<_>>>()
            .is_some_and(|actual| actual == expected)
    })
}

fn field_type_array_eq(value: &serde_json::Value, expected: &[(&str, &str)]) -> bool {
    value.as_array().is_some_and(|actual| {
        let actual = actual
            .iter()
            .map(|field| {
                let field = field.as_array()?;
                Some((field.first()?.as_str()?, field.get(1)?.as_str()?))
            })
            .collect::<Option<Vec<_>>>();
        actual.is_some_and(|actual| actual == expected)
    })
}
