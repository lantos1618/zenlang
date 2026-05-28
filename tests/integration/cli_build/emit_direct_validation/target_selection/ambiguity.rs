#[test]
fn emit_command_build_zen_rejects_multiple_executable_targets() {
    let (tmp, output) = super::super::run_emit_build_zen(
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
        ],
        &[
            ("app.zen", super::MAIN_ZERO),
            ("tool.zen", super::MAIN_ZERO),
        ],
    );

    super::assert_emit_rejected_without_outputs(
        &tmp,
        &output,
        "build graph C emission supports exactly one target, found 2",
        "graph emission is ambiguous",
    );
}

#[test]
fn emit_command_build_zen_reports_multi_target_ambiguity_before_missing_executable_source() {
    let (tmp, output) = super::super::run_emit_build_zen(
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Executable { name: "tool", main: "missing_tool.zen", out_dir: "build/tool/" })"#,
        ],
        &[("app.zen", super::MAIN_ZERO)],
    );

    super::assert_emit_rejected_without_outputs(
        &tmp,
        &output,
        "build graph C emission supports exactly one target, found 2",
        "graph emission is ambiguous",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("missing_tool.zen"),
        "emit should reject ambiguous executable graphs before per-target source validation, stderr={stderr}"
    );
}

#[test]
fn emit_command_build_zen_reports_multi_target_ambiguity_before_graph_only_library_typechecking() {
    let (tmp, output) = super::super::run_emit_build_zen(
        &[
            r#"    b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })"#,
            r#"    b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })"#,
            r#"    b.add(Library { name: "core", exports: ["lib.zen"] })"#,
        ],
        &[
            ("app.zen", super::MAIN_ZERO),
            ("tool.zen", super::MAIN_ZERO),
            ("lib.zen", super::MAIN_TRUE),
        ],
    );

    super::assert_emit_rejected_without_outputs(
        &tmp,
        &output,
        "build graph C emission supports exactly one target, found 2",
        "graph emission is ambiguous",
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("return type mismatch"),
        "emit should reject ambiguous executable graphs before graph-only library typechecking, stderr={stderr}"
    );
}
