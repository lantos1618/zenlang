use std::process::Output;

use super::support::{assert_zen_success, run_emit_json_build_graph_targets, run_zen};
mod golden;
mod validation;

#[test]
fn emit_json_build_graph_outputs_project_build_graph() {
    let json = build_graph_json(&emit_build_graph_json(&[
        "emit-json",
        "build-graph",
        "examples/project/build.zen",
    ]));
    assert_eq!(json["format"], "zen.build_graph.v0");
    assert_eq!(json["schema_version"], 0);
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
    let json = build_graph_json(&emit_temp_build_graph_json(&[
        r#"    b.add(Library { name: "core", exports: ["src/math.zen", "src/strings.zen"] })"#,
    ]));

    assert_eq!(json["targets"][0]["name"], "core");
    assert_eq!(json["targets"][0]["kind"]["kind"], "library");
    assert_eq!(json["targets"][0]["kind"]["exports"][0], "src/math.zen");
    assert_eq!(json["targets"][0]["kind"]["exports"][1], "src/strings.zen");
}

#[test]
fn emit_json_build_graph_outputs_target_dependencies_and_features() {
    let json = build_graph_json(&emit_temp_build_graph_json(&[
        r#"    b.add(Library { name: "core", exports: ["src/lib.zen"] })"#,
        r#"    b.add(Executable {
        name: "app",
        main: "app.zen",
        out_dir: "build/app/",
        dependencies: ["core"],
        features: ["lto", "release"],
    })"#,
    ]));

    assert_eq!(json["targets"][0]["name"], "app");
    assert_eq!(json["targets"][0]["dependencies"][0], "core");
    assert_eq!(json["targets"][0]["features"][0], "lto");
    assert_eq!(json["targets"][0]["features"][1], "release");
}

fn emit_build_graph_json(args: &[&str]) -> String {
    build_graph_json_output(run_zen(args), args)
}

fn emit_temp_build_graph_json(targets: &[&str]) -> String {
    let args = ["emit-json", "build-graph", "build.zen"];
    build_graph_json_output(run_emit_json_build_graph_targets(targets), &args)
}

fn build_graph_json_output(output: Output, args: &[&str]) -> String {
    assert_zen_success(args, &output);
    serde_json::from_slice::<serde_json::Value>(&output.stdout).expect("build graph json");
    String::from_utf8(output.stdout).expect("utf8 build graph json")
}

fn build_graph_json(source: &str) -> serde_json::Value {
    serde_json::from_str(source).expect("build graph json")
}
