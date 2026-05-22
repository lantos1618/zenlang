use super::*;

#[test]
fn resolver_absence_symbol_source_tests_stay_split_by_helper_surface() {
    let root = read("src/typechecker/tests/resolver_absence/symbols_sources.rs");
    let absent_metadata =
        read("src/typechecker/tests/resolver_absence/symbols_sources/absent_metadata.rs");
    let source_absence =
        read("src/typechecker/tests/resolver_absence/symbols_sources/source_absence.rs");
    let source_validation =
        read("src/typechecker/tests/resolver_absence/symbols_sources/source_validation.rs");
    let symbol_presence =
        read("src/typechecker/tests/resolver_absence/symbols_sources/symbol_presence.rs");

    assert!(
        root.lines().count() < 80,
        "symbols_sources.rs should only route focused resolver absence helper tests"
    );
    for module in [
        "mod absent_metadata;",
        "mod source_absence;",
        "mod source_validation;",
        "mod symbol_presence;",
    ] {
        assert!(
            root.contains(module),
            "symbols_sources.rs should include focused module `{module}`"
        );
    }
    for test_name in [
        "resolver_symbol_presence_validation_formats_messages",
        "source_absence_validation_builds_source_validation",
        "source_validation_formats_message",
        "absent_metadata_entry_formats_message",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "symbols_sources.rs should not own concrete test body: {test_name}"
        );
    }
    assert!(
        symbol_presence.contains("fn resolver_symbol_presence_validation_formats_messages")
            && symbol_presence.contains("fn resolver_symbol_presence_validation_pushes_diagnostic"),
        "symbol_presence.rs should cover resolver symbol presence helpers"
    );
    assert!(
        source_absence.contains("fn source_absence_validation_builds_source_validation")
            && source_absence.contains("fn source_absence_validation_uses_value_resolver_code"),
        "source_absence.rs should cover source-absence helper codes"
    );
    assert!(
        source_validation.contains("fn source_validation_formats_message")
            && source_validation.contains("fn source_validation_uses_resolver_codes"),
        "source_validation.rs should cover source validation formatting and codes"
    );
    assert!(
        absent_metadata.contains("fn absent_metadata_entry_formats_message")
            && absent_metadata
                .contains("fn resolver_named_list_display_formats_known_and_missing_items"),
        "absent_metadata.rs should cover absent metadata and named-list display helpers"
    );
}
