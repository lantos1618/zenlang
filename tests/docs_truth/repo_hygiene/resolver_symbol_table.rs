use super::*;

#[test]
fn resolver_symbol_table_storage_lives_in_focused_helper() {
    let definitions = read("src/resolver/symbol_table/definitions.rs");
    let storage = read("src/resolver/symbol_table/storage.rs");

    for helper in ["define_in_scope", "define_local", "new_scope"] {
        assert!(
            !definitions.contains(&format!("fn {helper}")),
            "symbol definition metadata constructors should not own storage helper: {helper}"
        );
        assert!(
            storage.contains(&format!("fn {helper}")),
            "symbol storage helper should live in focused helper: {helper}"
        );
    }

    let root = read("src/resolver/symbol_table.rs");
    assert!(
        root.contains("include!(\"symbol_table/storage.rs\");"),
        "symbol table should include focused storage helper"
    );
}

#[test]
fn resolver_symbol_definition_metadata_lives_in_focused_helper() {
    let root = read("src/resolver/symbol_table.rs");
    let definitions = read("src/resolver/symbol_table/definitions.rs");
    let metadata = read("src/resolver/symbol_table/definition_metadata.rs");

    for helper in [
        "empty_symbol_metadata",
        "value_symbol_metadata",
        "type_like_symbol_metadata",
        "variant_symbol_metadata",
        "behavior_symbol_metadata",
    ] {
        assert!(
            !definitions.contains(&format!("fn {helper}")),
            "symbol definition dispatch should not own metadata helper: {helper}"
        );
        assert!(
            metadata.contains(&format!("fn {helper}")),
            "symbol definition metadata helper should live in focused helper: {helper}"
        );
    }

    assert!(
        !definitions.contains("SymbolMetadata {"),
        "symbol definition dispatch should call metadata helpers instead of building raw SymbolMetadata"
    );
    assert!(
        definitions.lines().count() < 180,
        "symbol definition dispatch should stay focused on definition routing"
    );
    assert!(
        root.contains("include!(\"symbol_table/definition_metadata.rs\");"),
        "symbol table should include focused definition metadata helper"
    );
}

#[test]
fn resolver_symbol_table_test_support_metadata_setters_live_in_focused_helpers() {
    let root = read("src/resolver/symbol_table_test_support.rs");
    let value_metadata = read("src/resolver/symbol_table_test_support/value_metadata.rs");
    let type_parameters = read("src/resolver/symbol_table_test_support/type_parameters.rs");

    assert!(
        root.lines().count() < 150,
        "symbol table test support root should stay focused on lookup and generic symbol mutation"
    );

    for helper in [
        "set_parameter_count_for_test",
        "set_parameter_type_names_for_test",
        "set_parameter_types_for_test",
        "set_parameter_names_for_test",
        "set_return_type_name_for_test",
        "set_return_type_for_test",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "symbol table test support root should not own value metadata setter: {helper}"
        );
        assert!(
            value_metadata.contains(&format!("fn {helper}")),
            "value_metadata.rs should own value metadata setter: {helper}"
        );
    }

    for helper in [
        "set_type_parameter_count_for_test",
        "set_type_parameter_names_for_test",
        "set_type_parameter_bounds_for_test",
        "set_type_parameter_bound_refs_for_test",
    ] {
        assert!(
            !root.contains(&format!("fn {helper}")),
            "symbol table test support root should not own type parameter setter: {helper}"
        );
        assert!(
            type_parameters.contains(&format!("fn {helper}")),
            "type_parameters.rs should own type parameter setter: {helper}"
        );
    }

    for module_name in ["value_metadata", "type_parameters"] {
        assert!(
            root.contains(&format!("mod {module_name};")),
            "symbol table test support root should include focused helper: {module_name}"
        );
    }
}
