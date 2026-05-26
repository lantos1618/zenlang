use super::*;

#[test]
fn check_module_graph_entry_specializes_imported_generic_functions() {
    let typed = check_module_graph_sources(
        &[("identity", "pub id<T> = (value: T) T { value }\n")],
        "{ id } = identity\n\nmain = () i32 { id<i32>(1) }\n",
    )
    .expect("graph import bindings should seed generic templates");

    assert!(typed
        .functions
        .iter()
        .any(|function| function.name == "id_i32"));
}

#[test]
fn check_module_graph_entry_specializes_imported_generic_enums() {
    let typed = check_module_graph_sources(
        &[(
            "option",
            r#"pub Option<T>:
    None,
    Some(T)

pub Result<T, E>:
    Ok(T),
    Err(E)
"#,
        )],
        r#"{ Option, Result } = option

main = () i32 {
    maybe = Option<i32>.Some(7)
    result = Result<i32, StaticString>.Ok(9)
    0
}
"#,
    )
    .expect("graph import bindings should seed generic enum templates");

    assert!(typed.types.iter().any(|ty| ty.name == "Option_i32"));
    assert!(typed
        .types
        .iter()
        .any(|ty| ty.name == "Result_i32_StaticString"));
}
