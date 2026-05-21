use super::*;

#[test]
fn resolver_struct_field_metadata_requires_field_types() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let fields = TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol"),
    )
    .expect("complete resolver fields");

    assert_eq!(fields, [("x".to_string(), AstType::I32)]);

    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(TypeChecker::resolver_struct_field_metadata(
        symbols
            .lookup(Namespace::Type, "Point")
            .expect("Point symbol")
    )
    .is_none());
}

#[test]
fn resolver_enum_variant_name_metadata_requires_variant_names() {
    let program = parse_program(
        r#"
Option: Some(i32), None
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let variants = TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol"),
    )
    .expect("complete resolver variants");

    assert_eq!(variants, ["Some", "None"]);

    symbols.set_variant_names_for_test(Namespace::Type, "Option", None);
    assert!(TypeChecker::resolver_enum_variant_name_metadata(
        symbols
            .lookup(Namespace::Type, "Option")
            .expect("Option symbol")
    )
    .is_none());
}
