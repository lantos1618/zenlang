#[test]
fn emit_json_build_graph_rejects_dynamic_target_adds() {
    super::assert_emit_json_build_graph_error_contains(
        &[r#"    enabled = true
    enabled ?
        | true {
            b.add(Executable { name: "app", main: "app.zen", out_dir: "build/app/" })
        }
        | false {
            b.add(Executable { name: "tool", main: "tool.zen", out_dir: "build/tool/" })
        }"#],
        "build targets must be added in the deterministic build graph body",
    );
}
