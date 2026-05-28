use super::{
    assert_build_program_error, executable_target, BuildGraph, BuildGraphInput, BuildTargetInput,
};

fn assert_input_error(targets: Vec<BuildTargetInput>, expected: &str) {
    let err = BuildGraph::from_input(BuildGraphInput {
        targets,
        declared_host_effects: Vec::new(),
        used_host_effects: Vec::new(),
    })
    .expect_err("build graph should fail");
    assert_eq!(err.to_string(), expected);
}

#[test]
fn build_graph_rejects_unknown_target_dependencies() {
    let mut target = executable_target("app", &["src/main.zen"]);
    target.dependencies = vec!["core".to_string()];

    assert_input_error(
        vec![target],
        "build target `app` depends on unknown target `core`",
    );
}

#[test]
fn build_graph_rejects_self_target_dependencies() {
    let mut target = executable_target("app", &["src/main.zen"]);
    target.dependencies = vec!["app".to_string()];

    assert_input_error(vec![target], "build target `app` cannot depend on itself");
}

#[test]
fn build_graph_rejects_cyclic_target_dependencies() {
    let mut app = executable_target("app", &["src/app.zen"]);
    app.dependencies = vec!["tool".to_string()];
    let mut tool = executable_target("tool", &["src/tool.zen"]);
    tool.dependencies = vec!["app".to_string()];

    assert_input_error(
        vec![app, tool],
        "build target dependency cycle includes `app`",
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
        .map(|target| target.name.to_string())
        .collect();
    assert_eq!(ordered_names, ["tool", "app"]);
}

#[test]
fn build_program_lowering_rejects_cyclic_target_dependencies() {
    assert_build_program_error(
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
        "build target dependency cycle includes `app`",
    );
}

#[test]
fn build_program_lowering_rejects_unknown_target_dependencies() {
    assert_build_program_error(
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
        "build target `app` depends on unknown target `core`",
    );
}

#[test]
fn build_program_lowering_rejects_self_target_dependencies() {
    assert_build_program_error(
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
        "build target `app` cannot depend on itself",
    );
}
