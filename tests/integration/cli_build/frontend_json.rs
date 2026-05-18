use crate::support::*;
use std::process::Command;

#[path = "frontend_json/module_graph.rs"]
mod module_graph;

#[test]
fn emit_json_usage_lists_supported_and_gated_modes() {
    let missing_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .arg("emit-json")
        .output()
        .expect("run zen emit-json without mode");
    assert!(
        !missing_mode_output.status.success(),
        "zen emit-json without mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&missing_mode_output.stdout),
        String::from_utf8_lossy(&missing_mode_output.stderr)
    );

    let unknown_mode_output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "unknown",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json unknown mode");
    assert!(
        !unknown_mode_output.status.success(),
        "zen emit-json unknown mode should fail: stdout={}, stderr={}",
        String::from_utf8_lossy(&unknown_mode_output.stdout),
        String::from_utf8_lossy(&unknown_mode_output.stderr)
    );

    let expected_usage =
        "Usage: zen emit-json <ast|symbols|typed|diagnostics|build-graph|hir|mir|layout|target-yaml> <file.zen>";
    for output in [&missing_mode_output, &unknown_mode_output] {
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_usage),
            "emit-json usage should list supported and gated modes, stderr={stderr}"
        );
    }
}

#[test]
fn emit_json_typed_command_outputs_checked_program() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "typed",
            test_dir().join("generic_method.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json typed");

    assert!(
        output.status.success(),
        "zen emit-json typed failed: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("emit-json typed stdout is json");
    assert_eq!(json["format"], "zen.typed.v0");
    assert_eq!(json["semantic_status"], "checked");

    let functions = json["program"]["functions"]
        .as_array()
        .expect("typed functions array");
    assert!(
        functions
            .iter()
            .any(|function| function["name"] == "Box.get_i32"),
        "typed JSON should contain specialized generic method: {json}"
    );
    assert!(
        functions.iter().any(|function| function["name"] == "main"),
        "typed JSON should contain main function: {json}"
    );

    let types = json["program"]["types"]
        .as_array()
        .expect("typed types array");
    assert!(
        types.iter().any(|ty| ty["name"] == "Box_i32"),
        "typed JSON should contain specialized generic type: {json}"
    );

    let serialized = String::from_utf8(output.stdout).expect("typed JSON is utf-8");
    assert!(!serialized.contains("Box_T"));
    assert!(!serialized.contains("T Box_get"));
}

#[test]
fn emit_json_diagnostics_command_outputs_machine_readable_errors() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("bad_type.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    true
}
"#,
    )
    .expect("write bad source");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on errors: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("diagnostics stdout is json");
    assert_eq!(json["format"], "zen.diagnostics.v0");
    assert_eq!(json["semantic_status"], "diagnostic");
    assert_eq!(json["files"].as_array().expect("files array").len(), 1);

    let diagnostic = &json["diagnostics"][0];
    assert_eq!(diagnostic["severity"], "error");
    assert_eq!(diagnostic["code"], "E3030");
    assert!(
        diagnostic["message"]
            .as_str()
            .expect("diagnostic message")
            .contains("return type mismatch: expected `i32`, found `bool`"),
        "unexpected diagnostic payload: {diagnostic}"
    );

    let span = &diagnostic["span"];
    assert!(span["path"]
        .as_str()
        .expect("span path")
        .ends_with("bad_type.zen"));
    assert_eq!(span["line"], 3);
    assert_eq!(span["column"], 5);
}

#[test]
fn emit_json_mir_command_is_explicitly_gated() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "mir",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json mir");

    assert!(
        !output.status.success(),
        "zen emit-json mir should be gated: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("MIR JSON emission is gated until schema and golden tests exist"),
        "expected MIR gate diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_mir_rejects_program_before_mir_json() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("mir_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
main = () i32 {
    value = 40 + 2
    value
}
"#,
    )
    .expect("write MIR subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "mir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json mir on program input");

    assert!(
        !output.status.success(),
        "zen emit-json mir should be gated before MIR emission: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated MIR should not emit MIR JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("MIR JSON emission is gated until schema and golden tests exist"),
        "expected MIR gate diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "MIR should reject through the schema/golden-test gate, not command/path handling, stderr={stderr}"
    );
}

#[test]
fn emit_json_hir_command_is_explicitly_gated() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "hir",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json hir");

    assert!(
        !output.status.success(),
        "zen emit-json hir should be gated: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("HIR JSON emission is gated until schema and golden tests exist"),
        "expected HIR gate diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_hir_rejects_program_before_hir_json() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("hir_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Box<T>: {
    value: T
}

main = () i32 {
    box = Box<i32> { value: 7 }
    box.value
}
"#,
    )
    .expect("write HIR subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "hir", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json hir on program input");

    assert!(
        !output.status.success(),
        "zen emit-json hir should be gated before HIR emission: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated HIR should not emit HIR JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("HIR JSON emission is gated until schema and golden tests exist"),
        "expected HIR gate diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "HIR should reject through the schema/golden-test gate, not command/path handling, stderr={stderr}"
    );
}

#[test]
fn emit_json_target_yaml_command_is_explicitly_gated() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "target-yaml",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json target-yaml");

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should be gated: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(
            "target YAML validation is gated until schemas and negative validation tests exist"
        ),
        "expected target YAML gate diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_target_yaml_rejects_hand_authored_yaml_before_validation() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let yaml_path = tmp.path().join("target.yaml");
    std::fs::write(
        &yaml_path,
        r#"
triple: x86_64-unknown-linux-gnu
layout:
  pointer_width: 64
overrides:
  i32: i64
"#,
    )
    .expect("write target yaml");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "target-yaml", yaml_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json target-yaml on YAML input");

    assert!(
        !output.status.success(),
        "zen emit-json target-yaml should be gated before YAML validation: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated target-yaml should not emit schema or target JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains(
            "target YAML validation is gated until schemas and negative validation tests exist"
        ),
        "expected target YAML gate diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "target-yaml should reject through the IR-boundary gate, not command/path handling, stderr={stderr}"
    );
}

#[test]
fn emit_json_layout_command_is_explicitly_gated() {
    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args([
            "emit-json",
            "layout",
            test_dir().join("hello.zen").to_str().unwrap(),
        ])
        .output()
        .expect("run zen emit-json layout");

    assert!(
        !output.status.success(),
        "zen emit-json layout should be gated: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("type layout JSON emission is gated until ABI layout tests exist"),
        "expected layout gate diagnostic, stderr={stderr}"
    );
}

#[test]
fn emit_json_layout_rejects_program_before_layout_json() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join("layout_subject.zen");
    std::fs::write(
        &zen_path,
        r#"
Point: {
    x: i32,
    y: i32
}

Result<T, E>:
    Ok(T),
    Err(E)

main = () i32 {
    point = Point { x: 1, y: 2 }
    point.x
}
"#,
    )
    .expect("write layout subject");

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "layout", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json layout on program input");

    assert!(
        !output.status.success(),
        "zen emit-json layout should be gated before layout emission: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.trim().is_empty(),
        "gated layout should not emit type layout JSON, stdout={stdout}"
    );
    assert!(
        stderr.contains("type layout JSON emission is gated until ABI layout tests exist"),
        "expected layout gate diagnostic, stderr={stderr}"
    );
    assert!(
        !stderr.contains("unknown command") && !stderr.contains("No such file"),
        "layout should reject through the ABI-layout gate, not command/path handling, stderr={stderr}"
    );
}
