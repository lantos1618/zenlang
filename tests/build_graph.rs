use zen::build_graph::{
    BuildGraph, BuildGraphInput, BuildTargetInput, BuildTargetKind, HostEffect,
};
use zen::lexer;
use zen::parser;

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
        dependencies: vec!["std".to_string(), "math".to_string()],
        features: vec!["release".to_string(), "lto".to_string()],
    }
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
fn parsed_project_build_zen_lowers_to_executable_graph() {
    let program = parse_program(include_str!("../examples/project/build.zen"));
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 1);
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
    }
}

#[test]
fn build_program_lowering_rejects_undeclared_env_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("undeclared build.zen env read should fail");

    assert_eq!(
        err.to_string(),
        "undeclared host effect: read env `ZEN_STD`"
    );
}
