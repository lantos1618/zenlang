use super::super::support::run_emit_json_build_graph_targets;
use super::{assert_emit_json_build_graph_failure_contains, assert_stdout_empty};

const FILE_READ_DIAGNOSTIC: &str = "undeclared host effect: read file `build.targets`";
const FILE_READ_FALLBACK_ARMS: &[&str] = &[
    r#"| .Err { "default" }"#,
    r#"| _ { "default" }"#,
    r#"| err { "default" }"#,
];

#[test]
fn emit_json_build_graph_outputs_declared_file_read_effects() {
    for fallback_arm in FILE_READ_FALLBACK_ARMS {
        assert_emit_json_build_graph_outputs_declared_file_read_effect(fallback_arm);
    }
}

fn assert_emit_json_build_graph_outputs_declared_file_read_effect(fallback_arm: &str) {
    let body = format!(
        r#"    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) {{ contents }}
        {fallback_arm}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})"#,
    );
    let output = run_emit_json_build_graph_targets(&[&body]);

    assert!(
        output.status.success(),
        "emit-json build-graph failed for fallback `{fallback_arm}`: stdout={}, stderr={}",
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
    assert_emit_json_build_graph_failure_contains(
        &[r#"    manifest = b.os.read_file("build.targets")
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#],
        FILE_READ_DIAGNOSTIC,
    );
}

#[test]
fn emit_json_build_graph_rejects_file_read_without_fallback() {
    let output = assert_emit_json_build_graph_failure_contains(
        &[r#"    manifest = b.os.read_file("build.targets") ?
        | .Ok(contents) { contents }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#],
        FILE_READ_DIAGNOSTIC,
    );
    assert_stdout_empty(
        &output,
        "emit-json build-graph should not emit graph JSON after validation fails",
    );
}
