use super::assert_gate_diagnostics_golden;

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
fn emit_json_diagnostics_allocator_import_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "allocator_import_gate.zen",
        r#"
{ Allocator } = @std.memory.allocator

main = () void { }
"#,
        "allocator framework import gate",
        "allocator framework import gate should emit one std allocator feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_allocator_import_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_async_runtime_import_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "async_runtime_import_gate.zen",
        r#"
{ Scheduler } = @std.concurrency.async.scheduler

main = () void { }
"#,
        "async runtime import gate",
        "async runtime import gate should emit one std async feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_async_runtime_import_gate.golden.json",
    );
}

#[test]
fn emit_json_diagnostics_sync_runtime_import_gate_schema_matches_golden() {
    assert_gate_diagnostics_golden(
        "sync_runtime_import_gate.zen",
        r#"
{ Channel } = @std.concurrency.sync.channel

main = () void { }
"#,
        "sync runtime import gate",
        "sync runtime import gate should emit one std sync feature-gate diagnostic",
        "tests/fixtures/ir_json/diagnostics_sync_runtime_import_gate.golden.json",
    );
}
