use super::*;

#[test]
fn resolver_declaration_metadata_tasks_collect_impl_blocks_with_declarations() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Option: Some(i32), None

Json: behavior {
    encode: (Self) StaticString
}

Point.impl = {
    x_value = (value: Point) i32 { value.x }
}

Point.implements(Json) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json)

main = () i32 { 0 }
"#,
    );

    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    assert_eq!(tasks.types.len(), 2);
    assert_eq!(tasks.behaviors.len(), 1);
    assert_eq!(tasks.behaviors[0].name, "Json");
    assert_eq!(tasks.callable.len(), 2);
    assert!(tasks.behavior_associations.extends.is_empty());
    assert_eq!(tasks.behavior_associations.impls.len(), 1);
    let behavior_impl = &tasks.behavior_associations.impls[0];
    assert_eq!(behavior_impl.ast_type_name, "Point");
    assert_eq!(behavior_impl.behavior, "Json");
    assert_eq!(behavior_impl.methods.len(), 1);
    assert_eq!(tasks.behavior_associations.requires.len(), 1);
    let requires = &tasks.behavior_associations.requires[0];
    assert_eq!(requires.type_name, "Point");
    assert_eq!(requires.behavior, "Json");
    assert_eq!(tasks.type_references.len(), 6);

    let tc = TypeChecker::new();
    let refresh_tasks = tc.resolver_type_behavior_refresh_tasks(&tasks, &symbols);
    assert_eq!(refresh_tasks.len(), 2);
    assert_eq!(refresh_tasks[0].restored_name, "Point");
    assert_eq!(refresh_tasks[1].restored_name, "Option");
}

#[test]
fn expected_behavior_edges_build_parent_edges_from_extends_together() {
    let program = parse_program(
        r#"
Json: behavior {
    encode: (Self) StaticString
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json)
"#,
    );

    let expected = ExpectedBehaviorEdges::parents_from(&program);
    let edge = &expected.edges_for("PrettyJson")[0];

    assert_eq!(edge.display, "Json");
    assert_eq!(edge.metadata.name, "Json");
    assert_eq!(edge.metadata.type_args, Vec::<AstType>::new());
}

#[test]
fn behavior_ref_role_validation_emits_selected_contains_diagnostics() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

Point.implements(PrettyJson) {
    pretty = (value: Point) StaticString { "pretty" }
}
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");
    let ty = symbols
        .lookup(Namespace::Type, "Point")
        .expect("type symbol");
    let mut tc = TypeChecker::new();

    tc.validate_resolver_behavior_ref_contains_for_role(
        BehaviorRefRole::Impl,
        ty,
        "Point",
        expected_behavior_edge("Json", &[AstType::Str]),
        Span::dummy(),
    );

    assert!(tc.diagnostics.iter().any(|d| d.code == "E0236" && d.message.contains(
            "resolver type symbol 'Point' has behavior impls 'PrettyJson', expected to include 'Json<StaticString>'"
        )));
    assert!(tc.diagnostics.iter().any(|d| d.code == "E0247" && d.message.contains(
            "resolver type symbol 'Point' has behavior impl refs 'PrettyJson', expected to include 'Json<StaticString>'"
        )));
}
