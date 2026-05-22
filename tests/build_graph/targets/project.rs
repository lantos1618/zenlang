use super::*;

#[test]
fn parsed_project_build_zen_lowers_to_executable_and_test_graph() {
    let program = parse_program(include_str!("../../../examples/project/build.zen"));
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 2);
    let target = &graph.targets()[0];
    assert_eq!(target.name(), "myapp");
    assert_eq!(target.sources(), ["main.zen"]);
    match target.kind() {
        BuildTargetKind::Executable {
            root_source_file,
            out_dir,
        } => {
            assert_eq!(root_source_file, "main.zen");
            assert_eq!(out_dir, "build/");
        }
        other => panic!("expected executable target, got {other:?}"),
    }
    let target = &graph.targets()[1];
    assert_eq!(target.name(), "test");
    assert_eq!(target.sources(), ["test.zen"]);
    match target.kind() {
        BuildTargetKind::Test { root_source_file } => {
            assert_eq!(root_source_file, "test.zen");
        }
        other => panic!("expected test target, got {other:?}"),
    }
}
