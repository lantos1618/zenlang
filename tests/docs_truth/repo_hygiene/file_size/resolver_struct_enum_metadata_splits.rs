use super::super::*;

#[test]
fn resolver_enum_function_payload_tests_live_in_focused_helper() {
    let root = read("src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata.rs");
    let function_payloads = read(
        "src/typechecker/tests/resolver_struct_enum_metadata/enum_metadata/function_payloads.rs",
    );

    for test_name in [
        "check_program_with_symbols_validates_resolver_enum_function_type_payloads",
        "check_program_with_symbols_validates_resolver_enum_typed_payload_metadata",
        "check_program_with_symbols_validates_resolver_generic_enum_function_type_payloads",
    ] {
        assert!(
            !root.contains(&format!("fn {test_name}")),
            "enum_metadata.rs should not own function-type enum payload test: {test_name}"
        );
        assert!(
            function_payloads.contains(&format!("fn {test_name}")),
            "function-type enum payload tests should live in focused helper: {test_name}"
        );
    }

    assert!(
        root.lines().count() < 170,
        "enum_metadata.rs should stay focused on enum variant metadata counts, visibility, names, and owners"
    );
    assert!(
        root.contains("mod function_payloads;"),
        "enum_metadata.rs should include the focused function_payloads module"
    );
}
