use super::assert_gate_diagnostics_golden;

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
fn emit_json_diagnostics_generic_dynamic_string_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "generic_dynamic_string_gate.zen",
        r#"
Box<T>: {
    value: T
}

main = (box: Box<String>) void { }
"#,
        "generic dynamic string gate",
        "generic dynamic string gate should emit one allocator-backed text diagnostic",
        "tests/fixtures/ir_json/diagnostics_generic_dynamic_string_gate.golden.json",
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
