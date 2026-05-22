use super::*;

#[test]
fn resolver_behavior_association_list_tasks_collect_type_and_parent_edges_together() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

PrettyJson: behavior {
    pretty: (Self) StaticString
}

PrettyJson.extends(Json<StaticString>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let tasks = TypeChecker::collect_resolver_behavior_association_list_tasks(&program, &symbols);

    assert_eq!(tasks.type_associations.len(), 1);
    let type_task = &tasks.type_associations[0];
    assert_eq!(type_task.name, "Point");
    assert_eq!(type_task.impl_edges[0].display, "Json<StaticString>");
    assert_eq!(type_task.required_edges[0].display, "Json<StaticString>");

    assert_eq!(tasks.behavior_parents.len(), 2);
    let pretty_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "PrettyJson")
        .expect("PrettyJson parent task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<StaticString>");
    let json_task = tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "Json")
        .expect("Json empty parent task");
    assert!(json_task.parent_edges.is_empty());
}

#[test]
fn resolver_behavior_association_list_tasks_select_from_declaration_bundle() {
    let program = parse_program(
        r#"
Point: { x: i32 }

Json<T>: behavior {
    encode: (Self) T
}

Pretty<T>: behavior {
    pretty: (Self) StaticString
}

Pretty.extends(Json<T>)

Point.implements(Json<StaticString>) {
    encode = (value: Point) StaticString { "point" }
}

Point.requires(Json<StaticString>)
"#,
    );
    let symbols = crate::resolver::Resolver::new()
        .resolve_program(&program)
        .expect("resolver succeeds");

    let declaration_tasks =
        TypeChecker::collect_resolver_validation_replay_declaration_tasks(&program, &symbols);
    let association_tasks =
        TypeChecker::collect_resolver_behavior_association_list_tasks_from_declaration_tasks(
            &declaration_tasks,
        );

    assert_eq!(association_tasks.type_associations.len(), 1);
    assert_eq!(association_tasks.type_associations[0].name, "Point");
    assert_eq!(
        association_tasks.type_associations[0].impl_edges[0].display,
        "Json<StaticString>"
    );
    assert_eq!(
        association_tasks.type_associations[0].required_edges[0].display,
        "Json<StaticString>"
    );
    assert_eq!(association_tasks.behavior_parents.len(), 2);
    let pretty_task = association_tasks
        .behavior_parents
        .iter()
        .find(|task| task.name == "Pretty")
        .expect("Pretty parent association task");
    assert_eq!(pretty_task.parent_edges[0].display, "Json<T>");
}
