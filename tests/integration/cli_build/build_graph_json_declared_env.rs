use std::process::Command;

#[test]
fn emit_json_build_graph_outputs_declared_env_read_effects() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
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
        output.status.success(),
        "emit-json build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("build graph json");
    assert_eq!(json["declared_host_effects"][0]["kind"], "read_env");
    assert_eq!(json["declared_host_effects"][0]["value"], "ZEN_STD");
    assert_eq!(json["used_host_effects"][0]["kind"], "read_env");
    assert_eq!(json["used_host_effects"][0]["value"], "ZEN_STD");
}

#[test]
fn emit_json_build_graph_rejects_env_read_without_fallback() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
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
            .contains("undeclared host effect: read env `ZEN_STD`"),
        "expected undeclared env-read effect diagnostic, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stdout.is_empty(),
        "emit-json build-graph should not emit graph JSON after validation fails, stdout={}",
        String::from_utf8_lossy(&output.stdout)
    );
}
