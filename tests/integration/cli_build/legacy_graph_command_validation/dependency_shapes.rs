use super::*;

#[test]
fn build_graph_command_rejects_unknown_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
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
    )
    .expect("write build.zen");
    assert_build_graph_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` depends on unknown target `core`",
    );
}

#[test]
fn build_graph_command_rejects_self_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
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
    )
    .expect("write build.zen");
    assert_build_graph_command_rejects_dependency_shape(
        tmp.path(),
        "build target `app` cannot depend on itself",
    );
}

#[test]
fn build_graph_command_rejects_cyclic_target_dependencies() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    std::fs::write(
        tmp.path().join("build.zen"),
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
    )
    .expect("write build.zen");
    assert_build_graph_command_rejects_dependency_shape(
        tmp.path(),
        "build target dependency cycle includes `app`",
    );
}
