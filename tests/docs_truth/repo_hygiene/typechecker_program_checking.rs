use super::*;

#[test]
fn typechecker_module_graph_program_checking_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let program_checking = read("src/typechecker/program_checking.rs");
    let module_graph = read("src/typechecker/program_module_graph.rs");

    for helper in ["check_module_graph_entry", "check_module_graph_module"] {
        assert!(
            !program_checking.contains(&format!("fn {helper}")),
            "program_checking.rs should not own module graph checking helper: {helper}"
        );
        assert!(
            module_graph.contains(&format!("fn {helper}")),
            "module graph checking helper should live in focused helper: {helper}"
        );
    }

    assert!(
        root.contains("mod program_module_graph;"),
        "typechecker root should include focused module graph checking module"
    );
}
