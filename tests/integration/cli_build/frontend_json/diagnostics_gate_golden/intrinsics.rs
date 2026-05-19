use super::assert_gate_diagnostics_golden;

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
