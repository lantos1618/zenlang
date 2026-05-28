use super::assert_gate_diagnostics_golden;

#[test]
fn emit_json_diagnostics_control_gate_schemas_match_golden() {
    for (filename, source, description, count_context) in [
        (
            "range_gate.zen",
            r#"
main = () i32 {
    1..3
}
"#,
            "range gate",
            "range gate should emit one feature-gate diagnostic",
        ),
        (
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
        ),
        (
            "closure_gate.zen",
            r#"
main = () i32 {
    f = (input: i32) i32 {
        input
    }
    0
}
"#,
            "closure gate",
            "closure gate should emit one lowering feature-gate diagnostic",
        ),
    ] {
        assert_gate_diagnostics_golden(filename, source, description, count_context);
    }
}
