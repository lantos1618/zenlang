use super::*;

#[test]
fn build_graph_command_rejects_duplicate_target_fields() {
    assert_build_graph_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        name: "tool",
        main: "app.zen",
        out_dir: "build/app/",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Executable` build target",
    );
}

#[test]
fn build_graph_command_rejects_missing_required_target_fields() {
    assert_build_graph_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
    })
    .Ok(b.config())
}
"#,
        "missing required field `out_dir` in `Executable` build target",
    );
}

#[test]
fn build_graph_command_rejects_invalid_target_field_types() {
    assert_build_graph_command_rejects_target_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: 42,
    })
    .Ok(b.config())
}
"#,
        "field `out_dir` in `Executable` build target must be a string",
    );
}
