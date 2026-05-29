use super::assert_gate_diagnostics_golden;

#[test]
fn emit_json_diagnostics_intrinsic_gate_schemas_match_golden() {
    // Memory/pointer intrinsics are ungated (the compiler owns them; stdlib
    // builds allocators/collections on top). Enum construction/matching is
    // lowered directly to struct `.tag`/`.data` access in codegen, so it needs
    // no intrinsics. Still-gated below are the primitives whose effect/ABI
    // semantics aren't settled yet.
    for (filename, source, description) in [
        (
            "atomic_gate.zen",
            r#"
main = () void {
    @builtin.atomic_load(0)
}
"#,
            "atomic gate",
        ),
        (
            "syscall_gate.zen",
            r#"
main = () void {
    @builtin.syscall0(1)
}
"#,
            "syscall gate",
        ),
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
    ] {
        assert_gate_diagnostics_golden(
            filename,
            source,
            description,
            &format!("{description} should emit one gated intrinsic diagnostic"),
        );
    }
}
