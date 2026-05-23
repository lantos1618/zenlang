use super::*;

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
