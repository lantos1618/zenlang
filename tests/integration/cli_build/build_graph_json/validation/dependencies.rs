#[test]
fn emit_json_build_graph_rejects_unknown_target_dependencies() {
    super::assert_emit_json_build_graph_error_contains(
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
fn emit_json_build_graph_rejects_self_target_dependencies() {
    super::assert_emit_json_build_graph_error_contains(
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

#[test]
fn emit_json_build_graph_rejects_cyclic_target_dependencies() {
    super::assert_emit_json_build_graph_error_contains(
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
