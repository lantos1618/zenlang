use std::process::Command;

#[test]
fn emit_json_build_graph_outputs_declared_file_read_effects() {
    assert_emit_json_build_graph_outputs_declared_file_read_effect(
        r#"| .Err { "default" }"#,
        "emit_json_build_graph_outputs_declared_file_read_effects",
    );
}

#[test]
fn emit_json_build_graph_outputs_wildcard_fallback_declared_file_read_effects() {
    assert_emit_json_build_graph_outputs_declared_file_read_effect(
        r#"| _ { "default" }"#,
        "emit_json_build_graph_outputs_wildcard_fallback_declared_file_read_effects",
    );
}

#[test]
fn emit_json_build_graph_outputs_identifier_fallback_declared_file_read_effects() {
    assert_emit_json_build_graph_outputs_declared_file_read_effect(
        r#"| err { "default" }"#,
        "emit_json_build_graph_outputs_identifier_fallback_declared_file_read_effects",
    );
}

fn assert_emit_json_build_graph_outputs_declared_file_read_effect(
    fallback_arm: &str,
    case_name: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        format!(
            r#"
build = (b: Builder) Result<BuildConfig, BuildError> {{
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})
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
        output.status.success(),
        "{case_name}: emit-json build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("build graph json");
    assert_eq!(json["declared_host_effects"][0]["kind"], "read_file");
    assert_eq!(json["declared_host_effects"][0]["value"], "build.targets");
    assert_eq!(json["used_host_effects"][0]["kind"], "read_file");
    assert_eq!(json["used_host_effects"][0]["value"], "build.targets");
}

#[test]
fn emit_json_build_graph_rejects_undeclared_file_read_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
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
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file-read effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn emit_json_build_graph_rejects_file_read_without_fallback() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })
    .Ok(b.config())
}
"#,
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
        String::from_utf8_lossy(&output.stderr)
            .contains("undeclared host effect: read file `build.targets`"),
        "expected undeclared file-read effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "emit-json build-graph should not emit graph JSON after validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}
