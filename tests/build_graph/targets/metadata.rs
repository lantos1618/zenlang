use super::*;

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
