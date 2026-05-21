use super::{parse_program, BuildGraph, BuildTargetKind};

#[test]
fn parsed_project_build_zen_lowers_to_executable_and_test_graph() {
    let program = parse_program(include_str!("../../examples/project/build.zen"));
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

#[test]
fn build_program_lowering_collects_static_block_targets() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    {
        b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    }
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 1);
    assert_eq!(graph.targets()[0].name(), "app");
}

#[test]
fn build_program_lowering_rejects_dynamic_target_adds() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    enabled = true
    enabled ?
        | true {
            b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
        }
        | false {
            b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
        }
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("dynamic build target additions should stay gated");

    assert_eq!(
        err.to_string(),
        "build targets must be added in the deterministic build graph body"
    );
}

#[test]
fn build_program_lowering_collects_target_dependencies_and_features() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["src/lib.zen"] })
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
        features: ["lto", "release"],
    })
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 2);
    let target = &graph.targets()[0];
    assert_eq!(target.name(), "app");
    assert_eq!(target.dependencies(), ["core"]);
    assert_eq!(target.features(), ["lto", "release"]);
}
