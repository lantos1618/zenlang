use super::*;

#[test]
fn resolver_type_reference_validation_tasks_collect_only_type_reference_work() {
    let program = parse_program(
        r#"
Point: { x: i32 }

main = (input: Point) Point {
    input
}
"#,
    );

    let tasks =
        TypeChecker::collect_resolver_type_reference_validation_tasks(&program.declarations);

    assert_eq!(tasks.len(), 2);
    assert!(matches!(
        tasks[0],
        ResolverTypeReferenceValidationTask::Struct { name: "Point", .. }
    ));
    assert!(matches!(
        tasks[1],
        ResolverTypeReferenceValidationTask::Function { name: "main", .. }
    ));
}

#[test]
fn resolver_declaration_metadata_tasks_collect_top_level_type_reference_tasks() {
    let program = parse_program(
        r#"
value := 1
"#,
    );

    let tasks = TypeChecker::collect_resolver_declaration_metadata_tasks(&program.declarations);

    assert!(
        tasks.type_references.iter().any(|task| matches!(
            task,
            ResolverTypeReferenceValidationTask::TopLevelExpr { .. }
        )),
        "top-level expression type references should stay in the shared resolver task collector"
    );
}
