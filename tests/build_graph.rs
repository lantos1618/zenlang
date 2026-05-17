use zen::build_graph::{
    BuildGraph, BuildGraphInput, BuildTargetInput, BuildTargetKind, HostEffect,
};
use zen::lexer;
use zen::parser;

#[path = "build_graph/dependencies.rs"]
mod dependencies;
#[path = "build_graph/host_effects.rs"]
mod host_effects;
#[path = "build_graph/target_metadata.rs"]
mod target_metadata;

fn parse_program(src: &str) -> zen::ast::Program {
    let tokens = lexer::tokenize(src, 0).expect("lex build script");
    parser::parse(tokens, 0).expect("parse build script")
}

fn executable_target(name: &str, sources: &[&str]) -> BuildTargetInput {
    BuildTargetInput {
        name: name.to_string(),
        kind: BuildTargetKind::Executable {
            root_source_file: "src/main.zen".to_string(),
            out_dir: "build".to_string(),
        },
        sources: sources.iter().map(|source| source.to_string()).collect(),
        dependencies: Vec::new(),
        features: vec!["release".to_string(), "lto".to_string()],
    }
}

#[test]
fn build_target_kind_owns_diagnostic_spelling() {
    assert_eq!(
        BuildTargetKind::Executable {
            root_source_file: "src/main.zen".to_string(),
            out_dir: "build".to_string(),
        }
        .to_string(),
        "executable"
    );
    assert_eq!(
        BuildTargetKind::Test {
            root_source_file: "tests/main.zen".to_string(),
        }
        .to_string(),
        "test"
    );
    assert_eq!(
        BuildTargetKind::Library {
            exports: vec!["src/lib.zen".to_string()],
        }
        .to_string(),
        "library"
    );
}

#[test]
fn deterministic_build_graph_creates_one_executable_target() {
    let first = BuildGraph::from_input(BuildGraphInput {
        targets: vec![executable_target(
            "app",
            &["src/main.zen", "src/math.zen", "src/main.zen"],
        )],
        declared_host_effects: vec![HostEffect::ReadEnv("ZEN_STD".to_string())],
        used_host_effects: vec![HostEffect::ReadEnv("ZEN_STD".to_string())],
    })
    .expect("build graph");

    let second = BuildGraph::from_input(BuildGraphInput {
        targets: vec![executable_target(
            "app",
            &["src/main.zen", "src/main.zen", "src/math.zen"],
        )],
        declared_host_effects: vec![HostEffect::ReadEnv("ZEN_STD".to_string())],
        used_host_effects: vec![HostEffect::ReadEnv("ZEN_STD".to_string())],
    })
    .expect("build graph");

    assert_eq!(first.targets().len(), 1);
    assert!(first.targets()[0].is_executable());
    assert_eq!(
        first.canonical_json().unwrap(),
        second.canonical_json().unwrap()
    );
}

#[test]
fn build_graph_rejects_undeclared_host_effects() {
    let err = BuildGraph::from_input(BuildGraphInput {
        targets: vec![executable_target("app", &["src/main.zen"])],
        declared_host_effects: Vec::new(),
        used_host_effects: vec![HostEffect::ReadEnv("ZEN_STD".to_string())],
    })
    .expect_err("undeclared host effects should be rejected");

    assert_eq!(
        err.to_string(),
        "undeclared host effect: read env `ZEN_STD`"
    );
}

#[test]
fn parsed_project_build_zen_lowers_to_executable_and_test_graph() {
    let program = parse_program(include_str!("../examples/project/build.zen"));
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
