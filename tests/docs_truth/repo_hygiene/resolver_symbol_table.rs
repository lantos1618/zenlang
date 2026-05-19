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
