use super::*;

#[test]
fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
    check_module_graph_sources(
        &[(
            "geometry",
            r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    self.x
}
"#,
        )],
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("imported public type should seed its public methods");
}

#[test]
fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
    let err = check_module_graph_sources(
        &[(
            "geometry",
            r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    self.x
}
"#,
        )],
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect_err("private imported methods should not be seeded");

    assert!(
        err.iter()
            .any(|d| d.message.contains("type `Point` has no method `value`")),
        "expected private imported method diagnostic, got {err:?}"
    );
}

#[test]
fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
    let typed = check_module_graph_sources(
        &[(
            "geometry",
            r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    value
}
"#,
        )],
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.keep<i32>(1)
}
"#,
    )
    .expect("imported public type should seed public generic method templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "Point.keep_i32"));
}
