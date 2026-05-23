use super::*;

#[test]
fn build_program_lowering_collects_test_target() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { root: "tests/math.zen" })
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 1);
    let target = &graph.targets()[0];
    assert_eq!(target.name(), "math");
    assert_eq!(target.sources(), ["tests/math.zen"]);
    match target.kind() {
        BuildTargetKind::Test { root_source_file } => {
            assert_eq!(root_source_file, "tests/math.zen");
        }
        other => panic!("expected test target, got {other:?}"),
    }
}

#[test]
fn build_program_lowering_collects_library_target() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["src/math.zen", "src/strings.zen"] })
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 1);
    let target = &graph.targets()[0];
    assert_eq!(target.name(), "core");
    assert_eq!(target.sources(), ["src/math.zen", "src/strings.zen"]);
    match target.kind() {
        BuildTargetKind::Library { exports } => {
            assert_eq!(
                exports,
                &vec!["src/math.zen".to_string(), "src/strings.zen".to_string()]
            );
        }
        other => panic!("expected library target, got {other:?}"),
    }
}

#[test]
fn build_program_lowering_collects_multiple_executable_targets() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 2);
    assert_eq!(graph.targets()[0].name(), "app");
    assert_eq!(graph.targets()[0].sources(), ["app.zen"]);
    match graph.targets()[0].kind() {
        BuildTargetKind::Executable {
            root_source_file,
            out_dir,
        } => {
            assert_eq!(root_source_file, "app.zen");
            assert_eq!(out_dir, "build/app/");
        }
        other => panic!("expected executable target, got {other:?}"),
    }
    assert_eq!(graph.targets()[1].name(), "tool");
    assert_eq!(graph.targets()[1].sources(), ["tool.zen"]);
    match graph.targets()[1].kind() {
        BuildTargetKind::Executable {
            root_source_file,
            out_dir,
        } => {
            assert_eq!(root_source_file, "tool.zen");
            assert_eq!(out_dir, "build/tool/");
        }
        other => panic!("expected executable target, got {other:?}"),
    }
}
