use super::support::{
    assert_stdout_empty, assert_zen_failure_contains, assert_zen_success,
    run_emit_json_build_graph_targets,
};

const ENV_READ_DIAGNOSTIC: &str = "undeclared host effect: read env `ZEN_STD`";
const BUILD_GRAPH_JSON_ARGS: &[&str] = &["emit-json", "build-graph", "build.zen"];
const DECLARED_ENV_READ_FALLBACK_ARMS: &[&str] = &[
    r#"| .Err { "default" }"#,
    r#"| _ { "default" }"#,
    r#"| err { "default" }"#,
];

#[test]
fn emit_json_build_graph_outputs_declared_env_read_effects() {
    for fallback_arm in DECLARED_ENV_READ_FALLBACK_ARMS {
        assert_emit_json_build_graph_outputs_declared_env_read_effect(fallback_arm);
    }
}

fn assert_emit_json_build_graph_outputs_declared_env_read_effect(fallback_arm: &str) {
    let body = format!(
        r#"    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) {{ value }}
        {fallback_arm}
    b.add(Executable {{ name: "myapp", main: "main.zen", out_dir: "build/" }})"#,
    );
    let output = run_emit_json_build_graph_targets(&[&body]);

    assert_zen_success(BUILD_GRAPH_JSON_ARGS, &output);
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("build graph json");
    assert_eq!(json["declared_host_effects"][0]["kind"], "read_env");
    assert_eq!(json["declared_host_effects"][0]["value"], "ZEN_STD");
    assert_eq!(json["used_host_effects"][0]["kind"], "read_env");
    assert_eq!(json["used_host_effects"][0]["value"], "ZEN_STD");
}

#[test]
fn emit_json_build_graph_rejects_env_read_without_fallback() {
    let output = assert_emit_json_build_graph_failure_contains(
        &[r#"    std_path = b.os.env("ZEN_STD") ?
        | .Ok(value) { value }
    b.add(Executable { name: "myapp", main: "main.zen", out_dir: "build/" })"#],
        ENV_READ_DIAGNOSTIC,
    );
    assert_stdout_empty(
        &output,
        "emit-json build-graph should not emit graph JSON after validation fails",
    );
}

fn assert_emit_json_build_graph_failure_contains(
    targets: &[&str],
    expected_diagnostic: &str,
) -> std::process::Output {
    let output = run_emit_json_build_graph_targets(targets);
    assert_zen_failure_contains(BUILD_GRAPH_JSON_ARGS, &output, expected_diagnostic);
    output
}
