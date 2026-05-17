use super::parse_program;
use zen::build_graph::BuildGraph;

#[test]
fn build_program_lowering_rejects_duplicate_target_fields() {
    let program = parse_program(
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
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("duplicate build target fields should fail");

    assert_eq!(
        err.to_string(),
        "duplicate field `name` in `Executable` build target"
    );
}

#[test]
fn build_program_lowering_rejects_unknown_target_fields() {
    let program = parse_program(
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
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("unknown build target fields should fail");

    assert_eq!(
        err.to_string(),
        "unknown field `output_dir` in `Executable` build target"
    );
}

#[test]
fn build_program_lowering_rejects_missing_required_target_fields() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
    })
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("missing required build target fields should fail");

    assert_eq!(
        err.to_string(),
        "missing required field `out_dir` in `Executable` build target"
    );
}

#[test]
fn build_program_lowering_rejects_invalid_target_field_types() {
    let program = parse_program(
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
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("invalid build target field type should fail");

    assert_eq!(
        err.to_string(),
        "field `out_dir` in `Executable` build target must be a string"
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
    let program = parse_program(&format!(
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
    ));

    let err =
        BuildGraph::from_build_program(&program).expect_err("gated build target field should fail");

    assert_eq!(
        err.to_string(),
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
