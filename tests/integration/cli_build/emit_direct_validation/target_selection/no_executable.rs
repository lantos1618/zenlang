#[test]
fn emit_command_build_zen_rejects_graph_without_executable_targets() {
    let (tmp, output) = super::run_emit_build_zen(
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Test { name: "unit", root: "unit.zen" })
    .Ok(b.config())
}
"#,
        &[("unit.zen", super::main_source("0"))],
    );

    super::assert_emit_rejected_without_outputs(
        &tmp,
        &output,
        "build graph C emission supports exactly one target, found 0",
        "graph has no executable targets",
    );
}
