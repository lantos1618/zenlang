#[test]
fn emit_command_build_zen_rejects_graph_without_executable_targets() {
    let (tmp, output) = super::super::run_emit_build_zen(
        &[r#"    b.add(Test { name: "unit", root: "unit.zen" })"#],
        &[("unit.zen", super::MAIN_ZERO)],
    );

    super::assert_emit_rejected_without_outputs(
        &tmp,
        &output,
        "build graph C emission supports exactly one target, found 0",
        "graph has no executable targets",
    );
}
