use super::*;

#[test]
fn check_module_graph_entry_uses_graph_import_bindings() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let math_path = tmp.path().join("math.zen");
    std::fs::write(&math_path, "pub add = (a: i32, b: i32) i32 { a + b }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");
    let entry = graph.module(graph.entry).expect("entry module");
    assert!(
        !entry
            .program
            .declarations
            .iter()
            .any(|decl| decl.name() == Some("add")),
        "graph entry should not merge imported declarations"
    );

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed imported signatures");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "main"));
    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "add"));
}

#[test]
fn check_module_graph_entry_seeds_imported_function_type_signatures() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let callbacks_path = tmp.path().join("callbacks.zen");
    std::fs::write(
        &callbacks_path,
        "pub apply = (callback: (i32) i32, value: i32) i32 { value }\n",
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { value }
    apply(callback, 1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("graph import bindings should seed function-typed signatures");
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_functions() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let identity_path = tmp.path().join("identity.zen");
    std::fs::write(&identity_path, "pub id<T> = (value: T) T { value }\n")
        .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        "{ id } = identity\n\nmain = () i32 { id<i32>(1) }\n",
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "id_i32"));
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_enums() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let option_path = tmp.path().join("option.zen");
    std::fs::write(
        &option_path,
        r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, StaticString>.Ok(9)
    0
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("graph import bindings should seed generic enum templates");

    assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
    assert!(typed
        .types
        .iter()
        .any(|ty| ty.name == "Result_i32_StaticString"));
}

#[test]
fn check_module_graph_entry_seeds_public_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    tc.check_module_graph_entry(&graph)
        .expect("imported public type should seed its public methods");
}

#[test]
fn check_module_graph_entry_does_not_seed_private_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

Point.value = (self: Point) i32 {
    self.x
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.value()
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let err = TypeChecker::new()
        .check_module_graph_entry(&graph)
        .expect_err("private imported methods should not be seeded");

    assert!(
        err.iter()
            .any(|d| d.message.contains("type `Point` has no method `value`")),
        "expected private imported method diagnostic, got {err:?}"
    );
}

#[test]
fn check_module_graph_entry_specializes_public_generic_methods_for_imported_types() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let geometry_path = tmp.path().join("geometry.zen");
    std::fs::write(
        &geometry_path,
        r#"pub Point: { x: i32 }

pub Point.keep<T> = (self: Point, value: T) T {
    value
}
"#,
    )
    .expect("write imported module");

    let main_path = tmp.path().join("main.zen");
    std::fs::write(
        &main_path,
        r#"{ Point } = geometry

main = () i32 {
    point = Point { x: 7 }
    point.keep<i32>(1)
}
"#,
    )
    .expect("write entry module");

    let mut files = crate::error::FileTable::new();
    let mut modules = crate::module_system::ModuleSystem::new();
    let graph = modules
        .load_module_graph(&main_path, &mut files)
        .expect("module graph");

    let mut tc = TypeChecker::new();
    let typed = tc
        .check_module_graph_entry(&graph)
        .expect("imported public type should seed public generic method templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "Point.keep_i32"));
}
