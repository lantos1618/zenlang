use super::*;

#[test]
fn check_module_graph_entry_uses_graph_import_bindings() {
    let graph = module_graph_from_sources(
        &[("math", "pub add = (a: i32, b: i32) i32 { a + b }\n")],
        "{ add } = math\n\nmain = () i32 { add(1, 2) }\n",
    );
    let entry = graph.module(graph.entry).expect("entry module");
    assert!(
        !entry
            .program
            .declarations
            .iter()
            .any(|decl| decl.name() == Some("add")),
        "graph entry should not merge imported declarations"
    );

    let typed = TypeChecker::new()
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
    check_module_graph_sources(
        &[(
            "callbacks",
            "pub apply = (callback: (i32) i32, value: i32) i32 { value }\n",
        )],
        r#"{ apply } = callbacks

main = () i32 {
    callback = (value: i32) i32 { value }
    apply(callback, 1)
}
"#,
    )
    .expect("graph import bindings should seed function-typed signatures");
}
