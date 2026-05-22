use super::*;

#[test]
fn check_program_with_symbols_validates_resolver_struct_and_enum_absent_kind_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    symbols.set_variant_names_for_test(Namespace::Type, "Point", Some(vec!["Some".to_string()]));
    symbols.set_variant_payload_type_name_for_test(
        Namespace::Type,
        "Point",
        Some("i32".to_string()),
    );
    symbols.set_variant_payload_type_for_test(Namespace::Type, "Point", Some(AstType::I32));
    symbols.set_field_count_for_test(Namespace::Type, "Option", Some(1));
    symbols.set_field_type_names_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), "i32".to_string())]),
    );
    symbols.set_field_types_for_test(
        Namespace::Type,
        "Option",
        Some(vec![("value".to_string(), AstType::I32)]),
    );
    let mut tc = TypeChecker::new();

    let err = tc
        .check_program_with_symbols(&program, &symbols)
        .expect_err("resolver struct/enum kind metadata should fail");

    for expected in [
        "resolver type symbol 'Point' has variant names metadata, expected none",
        "resolver type symbol 'Point' has variant payload type metadata, expected none",
        "resolver type symbol 'Point' has typed variant payload type metadata, expected none",
        "resolver type symbol 'Option' has field count metadata, expected none",
        "resolver type symbol 'Option' has field types metadata, expected none",
        "resolver type symbol 'Option' has typed field types metadata, expected none",
    ] {
        assert!(
            err.iter().any(|d| d.message.contains(expected)),
            "expected resolver struct/enum kind metadata diagnostic '{expected}', got {err:?}"
        );
    }
}
