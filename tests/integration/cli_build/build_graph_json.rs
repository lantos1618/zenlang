use std::process::Command;

#[path = "build_graph_json/validation.rs"]
mod validation;

#[test]
fn emit_json_build_graph_outputs_project_build_graph() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "build-graph", "examples/project/build.zen"])
        .output()
        .expect("run zen emit-json build-graph");

    assert!(
        output.status.success(),
        "emit-json build-graph failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("build graph json");
    assert_eq!(json["format"], "zen.build_graph.v0");
    assert_eq!(json["semantic_status"], "deterministic");
    assert_eq!(json["targets"][0]["name"], "myapp");
    assert_eq!(json["targets"][0]["kind"]["root_source_file"], "main.zen");
    assert_eq!(json["targets"][0]["kind"]["out_dir"], "build/");
    assert_eq!(json["targets"][1]["name"], "test");
    assert_eq!(json["targets"][1]["kind"]["kind"], "test");
    assert_eq!(json["targets"][1]["kind"]["root_source_file"], "test.zen");
}

#[test]
fn emit_json_build_graph_outputs_library_target() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["src/math.zen", "src/strings.zen"] })
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
    assert_eq!(json["targets"][0]["name"], "core");
    assert_eq!(json["targets"][0]["kind"]["kind"], "library");
    assert_eq!(json["targets"][0]["kind"]["exports"][0], "src/math.zen");
    assert_eq!(json["targets"][0]["kind"]["exports"][1], "src/strings.zen");
}

#[test]
fn emit_json_build_graph_outputs_target_dependencies_and_features() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    b.add(Library { name: "core", exports: ["src/lib.zen"] })
    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
        features: ["lto", "release"],
    })
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
    assert_eq!(json["targets"][0]["name"], "app");
    assert_eq!(json["targets"][0]["dependencies"][0], "core");
    assert_eq!(json["targets"][0]["features"][0], "lto");
    assert_eq!(json["targets"][0]["features"][1], "release");
}
