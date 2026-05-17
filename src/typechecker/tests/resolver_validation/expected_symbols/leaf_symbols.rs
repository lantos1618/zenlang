use super::*;

#[test]
fn expected_import_symbol_builds_source_and_visibility_together() {
    let symbol = ExpectedImportSymbol::new("std.io");

    assert_eq!(symbol.source, "std.io");
    assert!(!symbol.is_public);
}

#[test]
fn expected_module_symbol_builds_name_source_and_visibility_together() {
    let symbol = ExpectedModuleSymbol::new("std.io");

    assert_eq!(symbol.name, "std.io");
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_local_symbol_builds_scope_mutability_source_and_visibility_together() {
    let symbol = ExpectedLocalSymbol::new(true, 42);

    assert_eq!(symbol.scope_id, 42);
    assert!(symbol.is_mutable);
    assert_eq!(symbol.source, None);
    assert!(!symbol.is_public);
}

#[test]
fn expected_behavior_edge_builds_display_and_metadata_together() {
    let edge = ExpectedBehaviorEdge::new("Json", &[AstType::I32]);

    assert_eq!(edge.display, "Json<i32>");
    assert_eq!(edge.metadata.name, "Json");
    assert_eq!(edge.metadata.type_args, vec![AstType::I32]);
}

#[test]
fn expected_behavior_associations_build_impl_and_required_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Point.implements(Json<str>) {
    encode = (value: Point) str { "point" }
}

Point.requires(Json<str>)
"#,
    );

    let expected = ExpectedBehaviorAssociations::new(&program);
    let impl_edge = &expected.impls.edges_for("Point")[0];
    let required_edge = &expected.required.edges_for("Point")[0];

    assert_eq!(impl_edge.display, "Json<str>");
    assert_eq!(impl_edge.metadata.name, "Json");
    assert_eq!(impl_edge.metadata.type_args, vec![AstType::Str]);
    assert_eq!(required_edge.display, "Json<str>");
    assert_eq!(required_edge.metadata.name, "Json");
    assert_eq!(required_edge.metadata.type_args, vec![AstType::Str]);
}
