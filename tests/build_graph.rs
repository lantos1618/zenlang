use zen::build_graph::{
    BuildGraph, BuildGraphInput, BuildTargetInput, BuildTargetKind, HostEffect,
};

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
