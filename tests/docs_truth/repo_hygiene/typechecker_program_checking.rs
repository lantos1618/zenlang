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

#[test]
fn typechecker_program_global_lowering_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let program_checking = read("src/typechecker/program_checking.rs");
    let globals = read("src/typechecker/program_globals.rs");

    assert!(
        !program_checking.contains("fn push_typed_global"),
        "program_checking.rs should not own top-level global lowering"
    );
    assert!(
        globals.contains("fn push_typed_global"),
        "program global lowering should live in focused helper"
    );
    assert!(
        program_checking.lines().count() < 260,
        "program_checking.rs should stay focused on program declaration checking"
    );
    assert!(
        root.contains("mod program_globals;"),
        "typechecker root should include focused program global helper"
    );
}

#[test]
fn typechecker_program_type_def_lowering_lives_in_focused_helper() {
    let root = read("src/typechecker/mod.rs");
    let program_checking = read("src/typechecker/program_checking.rs");
    let type_defs = read("src/typechecker/program_type_defs.rs");

    for helper in ["fn typed_struct_def(", "fn typed_enum_def("] {
        assert!(
            !program_checking.contains(helper),
            "program_checking.rs should not own typed type-definition lowering helper: {helper}"
        );
        assert!(
            type_defs.contains(helper),
            "typed type-definition lowering helper should live in focused module: {helper}"
        );
    }

    assert!(
        program_checking.lines().count() < 220,
        "program_checking.rs should stay focused on program declaration checking"
    );
    assert!(
        root.contains("mod program_type_defs;"),
        "typechecker root should include focused program type-definition helper"
    );
}
