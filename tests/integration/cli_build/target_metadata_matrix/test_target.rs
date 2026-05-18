#[test]
fn build_zen_commands_reject_duplicate_test_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test {
        name: "unit",
        name: "integration",
        root: "test.zen",
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Test` build target",
    );
}

#[test]
fn build_zen_commands_reject_missing_required_test_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit" })
    .Ok(b.config())
}
"#,
        "missing required field `root` or `root_source_file` in `Test` build target",
    );
}

#[test]
fn build_zen_commands_reject_invalid_test_target_field_types() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: 42 })
    .Ok(b.config())
}
"#,
        "field `root` in `Test` build target must be a string",
    );
}

#[test]
fn build_zen_commands_reject_unknown_test_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "test.zen", out_dir: "build/tests/" })
    .Ok(b.config())
}
"#,
        "unknown field `out_dir` in `Test` build target",
    );
}
