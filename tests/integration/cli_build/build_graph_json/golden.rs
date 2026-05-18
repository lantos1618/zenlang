use std::path::{Path, PathBuf};
use std::process::Command;

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_build_graph_project_schema_matches_golden() {
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

    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("build graph json");

    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_project.golden.json",
    ))
    .expect("read build graph golden fixture");
    let actual = String::from_utf8(output.stdout).expect("utf8 build graph json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_build_graph_host_effect_schema_matches_golden() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let build_path = tmp.path().join("build.zen");
    std::fs::write(
        &build_path,
        r#"
build = (b: Builder) Result<BuildConfig, BuildError> {
    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
        | .Err { "fallback" }
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

    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("build graph json");

    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_host_effects.golden.json",
    ))
    .expect("read build graph host effects golden fixture");
    let actual = String::from_utf8(output.stdout).expect("utf8 build graph json");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_build_graph_target_metadata_schema_matches_golden() {
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
        features: ["release", "lto"],
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

    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("build graph json");

    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_target_metadata.golden.json",
    ))
    .expect("read build graph target metadata golden fixture");
    let actual = String::from_utf8(output.stdout).expect("utf8 build graph json");

    assert_eq!(actual.trim(), expected.trim());
}
