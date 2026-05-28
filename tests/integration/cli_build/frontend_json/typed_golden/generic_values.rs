use super::assert_typed_golden;

#[test]
fn emit_json_typed_generic_value_schemas_match_golden() {
    for (source, golden_stem, description) in [
        (
            "generic_ufc_dedup.zen",
            "generic_ufc_dedup",
            "generic UFC dedup",
        ),
        (
            "generic_ufc_function.zen",
            "generic_ufc_function",
            "generic UFC function",
        ),
        (
            "generic_worklist_dedup.zen",
            "generic_worklist_dedup",
            "generic worklist dedup",
        ),
        (
            "generic_recursive_function.zen",
            "generic_recursive_function",
            "generic recursive function",
        ),
        (
            "multi_file_imported_generic_function_return_enum_dependency/main.zen",
            "multi_file_imported_generic_function_return_enum",
            "multi-file imported generic function return enum",
        ),
        ("generic_vec.zen", "generic_vec", "generic Vec"),
    ] {
        assert_typed_golden(source, golden_stem, description);
    }
}
