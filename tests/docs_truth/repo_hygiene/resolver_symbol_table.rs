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
