#[test]
fn build_zen_commands_reject_duplicate_library_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library {
        name: "core",
        name: "utils",
        exports: ["lib.zen"],
    })
    .Ok(b.config())
}
"#,
        "duplicate field `name` in `Library` build target",
    );
}

#[test]
fn build_zen_commands_reject_missing_library_exports() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core" })
    .Ok(b.config())
}
"#,
        "missing required field `exports` in `Library` build target",
    );
}

#[test]
fn build_zen_commands_reject_invalid_library_exports_type() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: "lib.zen" })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must be an array of strings",
    );
}

#[test]
fn build_zen_commands_reject_empty_library_exports() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: [] })
    .Ok(b.config())
}
"#,
        "field `exports` in `Library` build target must contain at least one source",
    );
}

#[test]
fn build_zen_commands_reject_unknown_library_target_fields() {
    super::assert_build_zen_commands_reject_build_graph_metadata(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["lib.zen"], output_dir: "build/lib/" })
    .Ok(b.config())
}
"#,
        "unknown field `output_dir` in `Library` build target",
    );
}
