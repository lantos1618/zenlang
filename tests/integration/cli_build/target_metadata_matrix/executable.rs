#[test]
fn build_zen_commands_reject_duplicate_executable_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
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
fn build_zen_commands_reject_missing_required_executable_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen" })
    .Ok(b.config())
}
"#,
        "missing required field `out_dir` in `Executable` build target",
    );
}

#[test]
fn build_zen_commands_reject_invalid_executable_target_field_types() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable { name: "app", main: "app.zen", out_dir: 42 })
    .Ok(b.config())
}
"#,
        "field `out_dir` in `Executable` build target must be a string",
    );
}
