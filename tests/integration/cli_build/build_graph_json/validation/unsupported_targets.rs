#[test]
fn emit_json_build_graph_rejects_unsupported_target_kinds() {
    for target_kind in ["Package", "Link"] {
        assert_emit_json_build_graph_rejects_unsupported_target_kind(target_kind);
    }
}

#[test]
fn emit_json_build_graph_rejects_gated_target_fields() {
    // `packages` stays gated until the package driver exists; `link` is now
    // supported (see build_program_lowering_accepts_executable_link_libraries).
    assert_emit_json_build_graph_rejects_gated_target_field("packages", r#"["std"]"#);
}

fn assert_emit_json_build_graph_rejects_unsupported_target_kind(target_kind: &str) {
    let target_add = format!(r#"    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})"#);
    super::assert_emit_json_build_graph_error_contains(
        &[&target_add],
        &format!(
            "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
        ),
    );
}

fn assert_emit_json_build_graph_rejects_gated_target_field(field: &str, value: &str) {
    let target_add = format!(
        r#"    b.add(Executable {{
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        {field}: {value},
    }})"#,
    );
    super::assert_emit_json_build_graph_error_contains(
        &[&target_add],
        &format!(
            "unsupported field `{field}` in `Executable` build target; package semantics are gated"
        ),
    );
}
