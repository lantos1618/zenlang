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
        dependencies: Vec::new(),
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
fn build_graph_rejects_unknown_target_dependencies() {
    let mut target = executable_target("app", &["src/main.zen"]);
    target.dependencies = vec!["core".to_string()];

    let err = BuildGraph::from_input(BuildGraphInput {
        targets: vec![target],
        declared_host_effects: Vec::new(),
        used_host_effects: Vec::new(),
    })
    .expect_err("unknown target dependency should fail");

    assert_eq!(
        err.to_string(),
        "build target `app` depends on unknown target `core`"
    );
}

#[test]
fn build_graph_rejects_self_target_dependencies() {
    let mut target = executable_target("app", &["src/main.zen"]);
    target.dependencies = vec!["app".to_string()];

    let err = BuildGraph::from_input(BuildGraphInput {
        targets: vec![target],
        declared_host_effects: Vec::new(),
        used_host_effects: Vec::new(),
    })
    .expect_err("self target dependency should fail");

    assert_eq!(
        err.to_string(),
        "build target `app` cannot depend on itself"
    );
}

#[test]
fn build_graph_rejects_cyclic_target_dependencies() {
    let mut app = executable_target("app", &["src/app.zen"]);
    app.dependencies = vec!["tool".to_string()];
    let mut tool = executable_target("tool", &["src/tool.zen"]);
    tool.dependencies = vec!["app".to_string()];

    let err = BuildGraph::from_input(BuildGraphInput {
        targets: vec![app, tool],
        declared_host_effects: Vec::new(),
        used_host_effects: Vec::new(),
    })
    .expect_err("cyclic target dependencies should fail");

    assert_eq!(
        err.to_string(),
        "build target dependency cycle includes `app`"
    );
}

#[test]
fn build_graph_orders_targets_before_dependents() {
    let mut app = executable_target("app", &["src/app.zen"]);
    app.dependencies = vec!["tool".to_string()];
    let tool = executable_target("tool", &["src/tool.zen"]);

    let graph = BuildGraph::from_input(BuildGraphInput {
        targets: vec![app, tool],
        declared_host_effects: Vec::new(),
        used_host_effects: Vec::new(),
    })
    .expect("build graph");

    let ordered_names: Vec<_> = graph
        .targets_in_dependency_order()
        .expect("dependency order")
        .into_iter()
        .map(|target| target.name().to_string())
        .collect();
    assert_eq!(ordered_names, ["tool", "app"]);
}

#[test]
fn build_program_lowering_rejects_cyclic_target_dependencies() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["tool"],
    })
    b.add(Executable {
        name: "tool",
        main: "tool.zen",
        out_dir: "build/tool/",
        dependencies: ["app"],
    })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("cyclic build target dependencies should fail");

    assert_eq!(
        err.to_string(),
        "build target dependency cycle includes `app`"
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

#[test]
fn build_program_lowering_rejects_unknown_target_dependencies() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
    })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("unknown build target dependency should fail");

    assert_eq!(
        err.to_string(),
        "build target `app` depends on unknown target `core`"
    );
}

#[test]
fn build_program_lowering_rejects_unsupported_package_targets() {
    assert_build_program_lowering_rejects_unsupported_target_kind("Package");
}

#[test]
fn build_program_lowering_rejects_unsupported_link_targets() {
    assert_build_program_lowering_rejects_unsupported_target_kind("Link");
}

fn assert_build_program_lowering_rejects_unsupported_target_kind(target_kind: &str) {
    let program = parse_program(&format!(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})
    .Ok(b.config())
}}
"#,
    ));

    let err =
        BuildGraph::from_build_program(&program).expect_err("unsupported build target should fail");

    assert_eq!(
        err.to_string(),
        format!(
            "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
        )
    );
}

#[test]
fn build_program_lowering_rejects_self_target_dependencies() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["app"],
    })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("self build target dependency should fail");

    assert_eq!(
        err.to_string(),
        "build target `app` cannot depend on itself"
    );
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

#[test]
fn build_program_lowering_accepts_declared_file_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
        | .Err { "default" }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");
    let json = graph.canonical_json().expect("build graph json");

    assert!(
        json.contains(r#""kind":"read_file","value":"build.targets""#),
        "expected read-file host effect in graph json, json={json}"
    );
}

#[test]
fn build_program_lowering_rejects_undeclared_file_reads() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("undeclared build.zen file read should fail");

    assert_eq!(
        err.to_string(),
        "undeclared host effect: read file `build.targets`"
    );
}
