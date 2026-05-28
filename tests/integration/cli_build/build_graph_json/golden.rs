use std::path::{Path, PathBuf};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

#[test]
fn emit_json_build_graph_project_schema_matches_golden() {
    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_project.golden.json",
    ))
    .expect("read build graph golden fixture");
    let actual =
        super::emit_build_graph_json(&["emit-json", "build-graph", "examples/project/build.zen"]);

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_build_graph_host_effect_schema_matches_golden() {
    let actual = super::emit_temp_build_graph_json(&[r#"    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
        | .Err { "default" }
    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
        | .Err { "fallback" }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#]);
    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_host_effects.golden.json",
    ))
    .expect("read build graph host effects golden fixture");

    assert_eq!(actual.trim(), expected.trim());
}

#[test]
fn emit_json_build_graph_target_metadata_schema_matches_golden() {
    let actual = super::emit_temp_build_graph_json(&[
        r#"    b.add(Library { name: "core", exports: ["src/lib.zen"] })"#,
        r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
        features: ["release", "lto"],
    })"#,
    ]);
    let expected = std::fs::read_to_string(fixture(
        "tests/fixtures/ir_json/build_graph_target_metadata.golden.json",
    ))
    .expect("read build graph target metadata golden fixture");

    assert_eq!(actual.trim(), expected.trim());
}
