use super::assert_gate_diagnostics_golden;

#[test]
fn emit_json_diagnostics_intrinsic_gate_schemas_match_golden() {
    for (filename, source, description) in [
        (
            "raw_allocate_gate.zen",
            r#"
main = () void {
    @builtin.raw_allocate(8)
}
"#,
            "raw allocation gate",
        ),
        (
            "byte_memory_gate.zen",
            r#"
main = () void {
    @builtin.memcpy(0, 0, 8)
}
"#,
            "byte memory gate",
        ),
        (
            "raw_pointer_gate.zen",
            r#"
main = () void {
    @builtin.gep(0, 1)
}
"#,
            "raw pointer gate",
        ),
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
        (
            "enum_payload_gate.zen",
            r#"
main = () void {
    @builtin.set_payload(0, 0)
}
"#,
            "enum payload gate",
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
