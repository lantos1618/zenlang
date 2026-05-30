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
    assert_build_program_error(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        packages: ["std"],
    })
    .Ok(b.config())
}
"#,
        "unsupported field `packages` in `Executable` build target; package semantics are gated",
    );
}

#[test]
fn build_program_lowering_accepts_executable_link_libraries() {
    let program = super::parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        link: ["SDL3", "m"],
    })
    .Ok(b.config())
}
"#,
    );
    let graph = zen::build_graph::BuildGraph::from_build_program(&program)
        .expect("build program with link should lower");
    match &graph.targets[0].kind {
        zen::build_graph::BuildTargetKind::Executable { link, .. } => {
            assert_eq!(link.as_slice(), ["SDL3", "m"]);
        }
        other => panic!("expected executable target, got {other:?}"),
    }
}

#[test]
fn build_program_lowering_accepts_executable_headers() {
    let program = super::parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        link: ["SDL3"],
        headers: ["SDL3/SDL.h"],
    })
    .Ok(b.config())
}
"#,
    );
    let graph = zen::build_graph::BuildGraph::from_build_program(&program)
        .expect("build program with headers should lower");
    match &graph.targets[0].kind {
        zen::build_graph::BuildTargetKind::Executable { headers, .. } => {
            assert_eq!(headers.as_slice(), ["SDL3/SDL.h"]);
        }
        other => panic!("expected executable target, got {other:?}"),
    }
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
