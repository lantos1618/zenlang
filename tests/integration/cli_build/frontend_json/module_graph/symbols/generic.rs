use super::*;

#[test]
fn emit_json_symbols_reports_multi_file_generic_enum_method_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_generic_enum_method/main.zen",
        "multi-file generic enum method symbols",
    );

    assert_eq!(
        modules.len(),
        2,
        "fixture should report main and option modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "Option", |symbol| {
        symbol["import_source"] == "option"
    });

    let option = module_by_file(&modules, "option.zen");
    assert_symbol(option, "Type", "Option", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T"])
            && string_array_eq(&symbol["variant_names"], &["None", "Some"])
    });
    assert_symbol(option, "Value", "Option.unwrap_or", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Self", "T"])
            && symbol["return_type_name"] == "T"
    });
}

#[test]
fn emit_json_symbols_reports_multi_file_generic_result_method_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_generic_result_enum_method/main.zen",
        "multi-file generic Result enum method symbols",
    );

    assert_eq!(
        modules.len(),
        2,
        "fixture should report main and result modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "Result", |symbol| {
        symbol["import_source"] == "result"
    });

    let result = module_by_file(&modules, "result.zen");
    assert_symbol(result, "Type", "Result", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T", "E"])
            && string_array_eq(&symbol["variant_names"], &["Ok", "Err"])
    });
    assert_symbol(result, "Value", "Result.unwrap_or", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Self", "T"])
            && symbol["return_type_name"] == "T"
    });
}

#[test]
fn emit_json_symbols_reports_multi_file_generic_function_return_enum_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_imported_generic_function_return_enum_dependency/main.zen",
        "multi-file imported generic function return enum symbols",
    );

    assert_eq!(
        modules.len(),
        3,
        "fixture should report main, model, and types modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "wrap", |symbol| {
        symbol["import_source"] == "model"
    });
    assert_symbol(main, "Import", "unwrap", |symbol| {
        symbol["import_source"] == "model"
    });

    let model = module_by_file(&modules, "model.zen");
    assert_symbol(model, "Import", "Option", |symbol| {
        symbol["import_source"] == "types"
    });
    assert_symbol(model, "Value", "wrap", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["T"])
            && symbol["return_type_name"] == "Option<T>"
    });
    assert_symbol(model, "Value", "unwrap", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Option<T>", "T"])
            && symbol["return_type_name"] == "T"
    });

    let types = module_by_file(&modules, "types.zen");
    assert_symbol(types, "Type", "Option", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T"])
            && string_array_eq(&symbol["variant_names"], &["None", "Some"])
    });
}

#[test]
fn emit_json_symbols_reports_multi_file_generic_method_nested_result_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_type_method_nested_result_dependency/main.zen",
        "multi-file generic method nested Result symbols",
    );

    assert_eq!(
        modules.len(),
        3,
        "multi-file fixture should report main, types, and model modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "Box", |symbol| {
        symbol["import_source"] == "model"
    });
    assert_symbol(main, "Import", "Option", |symbol| {
        symbol["import_source"] == "types"
    });
    assert_symbol(main, "Import", "Result", |symbol| {
        symbol["import_source"] == "types"
    });

    let types = module_by_file(&modules, "types.zen");
    assert_symbol(types, "Type", "Option", |symbol| {
        string_array_eq(&symbol["type_parameter_names"], &["T"])
            && string_array_eq(&symbol["variant_names"], &["None", "Some"])
    });
    assert_symbol(types, "Type", "Result", |symbol| {
        string_array_eq(&symbol["type_parameter_names"], &["T", "E"])
            && string_array_eq(&symbol["variant_names"], &["Ok", "Err"])
    });

    let model = module_by_file(&modules, "model.zen");
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

#[test]
fn emit_json_symbols_reports_multi_file_generic_result_multi_specialization_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_generic_result_enum_multi_specialization/main.zen",
        "multi-file generic Result multi-specialization symbols",
    );

    assert_eq!(
        modules.len(),
        2,
        "fixture should report main and result modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "Result", |symbol| {
        symbol["import_source"] == "result"
    });

    let result = module_by_file(&modules, "result.zen");
    assert_symbol(result, "Type", "Result", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T", "E"])
            && string_array_eq(&symbol["variant_names"], &["Ok", "Err"])
    });
    assert_symbol(result, "Value", "Result.unwrap_or", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Self", "T"])
            && symbol["return_type_name"] == "T"
    });
}

#[test]
fn emit_json_symbols_reports_multi_file_generic_result_error_multi_specialization_surface() {
    let modules = symbols_modules_for_fixture(
        "tests/zen/multi_file_generic_result_error_multi_specialization/main.zen",
        "multi-file generic Result error-type multi-specialization symbols",
    );

    assert_eq!(
        modules.len(),
        2,
        "fixture should report main and result modules: {modules:?}"
    );

    let main = module_by_file(&modules, "main.zen");
    assert_symbol(main, "Import", "Result", |symbol| {
        symbol["import_source"] == "result"
    });

    let result = module_by_file(&modules, "result.zen");
    assert_symbol(result, "Type", "Result", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["type_parameter_names"], &["T", "E"])
            && string_array_eq(&symbol["variant_names"], &["Ok", "Err"])
    });
    assert_symbol(result, "Value", "Result.unwrap_err", |symbol| {
        symbol["is_public"] == true
            && string_array_eq(&symbol["parameter_type_names"], &["Self", "E"])
            && symbol["return_type_name"] == "E"
    });
}

fn symbols_modules_for_fixture(source_path: &str, description: &str) -> Vec<serde_json::Value> {
    let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(source_path);
    let output = emit_json("symbols", &source_path, description);

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
    json["modules"]
        .as_array()
        .expect("modules array")
        .to_owned()
}
