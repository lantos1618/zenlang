use std::path::Path;
use std::process::Command;

fn fixture(path: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(path)
}

fn assert_gate_diagnostics_golden(
    zen_filename: &str,
    source: &str,
    failure_context: &str,
    single_diagnostic_context: &str,
    fixture_path: &str,
) {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let zen_path = tmp.path().join(zen_filename);
    std::fs::write(&zen_path, source).unwrap_or_else(|err| {
        panic!(
            "write {failure_context} source to {}: {err}",
            zen_path.display()
        )
    });

    let output = Command::new(env!("CARGO_BIN_EXE_zen"))
        .args(["emit-json", "diagnostics", zen_path.to_str().unwrap()])
        .output()
        .expect("run zen emit-json diagnostics");

    assert!(
        !output.status.success(),
        "zen emit-json diagnostics should fail on {failure_context}: stdout={}, stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = String::from_utf8(output.stdout).expect("diagnostics stdout is UTF-8");
    let json: serde_json::Value =
        serde_json::from_str(&actual).expect("diagnostics stdout is JSON");
    assert_eq!(
        json["diagnostics"]
            .as_array()
            .expect("diagnostics array")
            .len(),
        1,
        "{single_diagnostic_context}: {json}"
    );

    let normalized = actual.replace(tmp.path().to_str().expect("tmp path is UTF-8"), "$TMP");
    let expected_path = fixture(fixture_path);
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    assert_eq!(normalized.trim(), expected.trim());
}

#[test]
fn emit_json_diagnostics_typed_allocator_effect_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "typed_allocator_effect_gate.zen",
        r#"
main = (allocator: Allocator<i32, Sync>) void { }
"#,
        "typed allocator effect gate",
        "typed allocator/effect gate should emit one feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_typed_allocator_effect_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_async_intrinsic_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "async_intrinsic_gate.zen",
        r#"
main = () void {
    @builtin.async_enqueue(1)
}
"#,
        "async intrinsic gate",
        "async intrinsic gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_async_intrinsic_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_raw_allocate_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "raw_allocate_gate.zen",
        r#"
main = () void {
    @builtin.raw_allocate(8)
}
"#,
        "raw allocation gate",
        "raw allocation gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_raw_allocate_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_byte_memory_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "byte_memory_gate.zen",
        r#"
main = () void {
    @builtin.memcpy(0, 0, 8)
}
"#,
        "byte memory gate",
        "byte memory gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_byte_memory_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_raw_pointer_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "raw_pointer_gate.zen",
        r#"
main = () void {
    @builtin.gep(0, 1)
}
"#,
        "raw pointer gate",
        "raw pointer gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_raw_pointer_gate.golden.json",
    );
}
