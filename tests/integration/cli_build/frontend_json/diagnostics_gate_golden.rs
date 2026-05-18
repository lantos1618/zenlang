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
fn emit_json_diagnostics_sync_effect_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "sync_effect_gate.zen",
        r#"
main = (mode: Sync) void { }
"#,
        "sync effect gate",
        "sync effect gate should emit one Sync/Async feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_sync_effect_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_async_effect_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "async_effect_gate.zen",
        r#"
main = (mode: Async) void { }
"#,
        "async effect gate",
        "async effect gate should emit one Sync/Async feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_async_effect_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_dynamic_string_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "dynamic_string_gate.zen",
        r#"
main = (value: String) void { }
"#,
        "dynamic string gate",
        "dynamic string gate should emit one allocator-backed text diagnostic",
        "tests/fixtures/ir_json/diagnostics_dynamic_string_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_range_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "range_gate.zen",
        r#"
main = () i32 {
    1..3
}
"#,
        "range gate",
        "range gate should emit one feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_range_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_raise_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "raise_gate.zen",
        r#"
Result<T, E>:
    Ok(T),
    Err(E)

main = () i32 {
    value = Result<i32, StaticString>.Ok(1)
    value.raise()
}
"#,
        "raise gate",
        "raise gate should emit one propagation feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_raise_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_await_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "await_gate.zen",
        r#"
Task<T>: {
    value: T
}

main = () i32 {
    task = Task<i32> { value: 1 }
    task.await()
}
"#,
        "await gate",
        "await gate should emit one Sync/Async feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_await_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_actor_type_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "actor_type_gate.zen",
        r#"
main = (actor: Actor<i32>) void { }
"#,
        "actor framework type gate",
        "actor framework type gate should emit one actor feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_actor_type_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_bare_actor_type_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "bare_actor_type_gate.zen",
        r#"
main = (actor: Actor) void { }
"#,
        "bare actor framework type gate",
        "bare actor framework type gate should emit one actor feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_bare_actor_type_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_actor_import_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "actor_import_gate.zen",
        r#"
{ Actor } = @std.concurrency.actor.actor

main = () void { }
"#,
        "actor framework import gate",
        "actor framework import gate should emit one std actor feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_actor_import_gate.golden.json",
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

#[test]
fn emit_json_diagnostics_atomic_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "atomic_gate.zen",
        r#"
main = () void {
    @builtin.atomic_load(0)
}
"#,
        "atomic gate",
        "atomic gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_atomic_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_syscall_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "syscall_gate.zen",
        r#"
main = () void {
    @builtin.syscall0(1)
}
"#,
        "syscall gate",
        "syscall gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_syscall_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_type_match_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "type_match_gate.zen",
        r#"
Point: {
    x: i32,
}

main = () void {
    @builtin.type_match<Point>()
}
"#,
        "type match gate",
        "type match gate should emit one gated intrinsic diagnostic",
        "tests/fixtures/ir_json/diagnostics_type_match_gate.golden.json",
    );
}
