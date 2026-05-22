use super::super::*;

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
