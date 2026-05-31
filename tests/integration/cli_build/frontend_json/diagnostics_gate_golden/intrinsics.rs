use super::assert_gate_diagnostics_golden;

#[test]
fn emit_json_diagnostics_intrinsic_gate_schemas_match_golden() {
    // Memory/pointer intrinsics are ungated (the compiler owns them; stdlib
    // builds allocators/collections on top). `syscall*`, `atomic_*`, and `fence`
    // are now also ungated — settled OS/hardware hooks with full C lowering that
    // the stdlib builds sys/io/concurrency on (via the std.compiler facade).
    // Still-gated below: the async runtime hooks (mid-build, see ASYNC_PLAN.md)
    // and comptime `type_match` (semantics not settled).
    for (filename, source, description) in [
        (
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
        ),
        (
            "async_enqueue_gate.zen",
            r#"
main = () void {
    @builtin.async_enqueue(0)
}
"#,
            "async enqueue gate",
        ),
        (
            "async_yield_gate.zen",
            r#"
main = () void {
    @builtin.async_yield()
}
"#,
            "async yield gate",
        ),
    ] {
        assert_gate_diagnostics_golden(
            filename,
            source,
            description,
            &format!("{description} should emit one gated intrinsic diagnostic"),
        );
    }
}
