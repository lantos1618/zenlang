use super::*;

#[test]
fn method_key_formats_type_qualified_method_name() {
    assert_eq!(TypeChecker::method_key("Point", "encode"), "Point.encode");
}

#[test]
fn resolver_behavior_ref_owner_prefers_exact_then_unique_fallbacks() {
    let tc = TypeChecker::new();
    let mut refs_by_type = HashMap::from([
        (
            "Point".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Json".to_string(),
                type_args: vec![AstType::I32],
            }]),
        ),
        (
            "Label".to_string(),
            VecDeque::from(vec![BehaviorRefMetadata {
                name: "Debug".to_string(),
                type_args: vec![],
            }]),
        ),
    ]);
    let missing_refs = HashSet::new();

    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Json", &[AstType::I32]),
        Some("Point".to_string())
    );
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        None
    );

    refs_by_type.remove("Label");
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Point".to_string())
    );

    refs_by_type.clear();
    let missing_refs = HashSet::from(["Recovered".to_string()]);
    assert_eq!(
        tc.resolver_behavior_ref_owner_for(&refs_by_type, &missing_refs, "Missing", &[]),
        Some("Recovered".to_string())
    );
}

#[test]
fn resolver_symbol_metadata_helper_requires_symbol_and_selected_metadata() {
    let program = parse_program(
        r#"
Point: { x: i32 }
"#,
    );
    let mut symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    assert_eq!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| {
            symbol.field_types.as_ref()
        })
        .map(|(_, fields)| fields[0].0.as_str()),
        Some("x")
    );
    symbols.set_field_types_for_test(Namespace::Type, "Point", None);
    assert!(
        TypeChecker::resolver_symbol_metadata(&symbols, Namespace::Type, "Point", |symbol| symbol
            .field_types
            .as_ref())
        .is_none()
    );
    assert!(TypeChecker::resolver_symbol_metadata(
        &symbols,
        Namespace::Type,
        "Missing",
        |symbol| symbol.field_types.as_ref()
    )
    .is_none());
}
