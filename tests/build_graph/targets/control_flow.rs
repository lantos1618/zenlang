use super::*;

#[test]
fn build_program_lowering_collects_static_block_targets() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    {
        b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
    }
    .Ok(b.config())
}
"#,
    );
    let graph = BuildGraph::from_build_program(&program).expect("lower build graph");

    assert_eq!(graph.targets().len(), 1);
    assert_eq!(graph.targets()[0].name(), "app");
}

#[test]
fn build_program_lowering_rejects_dynamic_target_adds() {
    let program = parse_program(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    enabled = true
    enabled ?
        | true {
            b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
        }
        | false {
            b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
        }
    .Ok(b.config())
}
"#,
    );

    let err = BuildGraph::from_build_program(&program)
        .expect_err("dynamic build target additions should stay gated");

    assert_eq!(
        err.to_string(),
        "build targets must be added in the deterministic build graph body"
    );
}
