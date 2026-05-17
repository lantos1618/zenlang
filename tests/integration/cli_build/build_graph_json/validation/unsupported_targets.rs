use std::process::Command;

#[test]
fn emit_json_build_graph_rejects_unsupported_package_targets() {
    assert_emit_json_build_graph_rejects_unsupported_target_kind("Package");
}

#[test]
fn emit_json_build_graph_rejects_unsupported_link_targets() {
    assert_emit_json_build_graph_rejects_unsupported_target_kind("Link");
}

#[test]
fn emit_json_build_graph_rejects_gated_package_fields() {
    assert_emit_json_build_graph_rejects_gated_target_field("packages", r#"["std"]"#);
}

#[test]
fn emit_json_build_graph_rejects_gated_link_fields() {
    assert_emit_json_build_graph_rejects_gated_target_field("link", r#"["m"]"#);
}

fn assert_emit_json_build_graph_rejects_unsupported_target_kind(target_kind: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    b.add({target_kind} {{ name: "core", root: "src/lib.zen" }})
    .Ok(b.config())
}}
"#,
        ),
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", build_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        !output.status.success(),
        "emit-json build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(
            &format!(
                "unsupported build target kind `{target_kind}`; supported target kinds are `Executable`, `Test`, and `Library`"
            )
        ),
        "expected unsupported target diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_emit_json_build_graph_rejects_gated_target_field(field: &str, value: &str) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        format!(
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
    )
    .expect("write build.zen");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", build_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        !output.status.success(),
        "emit-json build-graph unexpectedly succeeded: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(&format!(
            "unsupported field `{field}` in `Executable` build target; package/link semantics are gated"
        )),
        "expected gated field diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
