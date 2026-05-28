use super::assert_build_program_error;

#[test]
fn build_program_lowering_rejects_duplicate_target_fields() {
    assert_build_program_error(
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
fn build_program_lowering_rejects_unknown_target_fields() {
    assert_build_program_error(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        output_dir: "build/app/",
    })
    .Ok(b.config())
}
"#,
        "unknown field `output_dir` in `Executable` build target",
    );
}

#[test]
fn build_program_lowering_rejects_missing_required_target_fields() {
    assert_build_program_error(
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
fn build_program_lowering_rejects_invalid_target_field_types() {
    assert_build_program_error(
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

#[test]
fn build_program_lowering_rejects_gated_package_fields() {
    assert_build_program_lowering_rejects_gated_target_field("packages", r#"["std"]"#);
}

#[test]
fn build_program_lowering_rejects_gated_link_fields() {
    assert_build_program_lowering_rejects_gated_target_field("link", r#"["m"]"#);
}

fn assert_build_program_lowering_rejects_gated_target_field(field: &str, value: &str) {
    assert_build_program_error(
        &format!(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})
    .Ok(b.config())
}}
"#,
        ),
        format!("unsupported field `{field}` in `Executable` build target; package/link semantics are gated")
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
    assert_build_program_error(
        &format!(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})
    .Ok(b.config())
}}
"#,
        ),
        format!(
            "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
        )
    );
}
